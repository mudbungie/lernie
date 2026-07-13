//! Root-conversation branch orchestration (ARCH §2.3, §2.5, §2.6, §2.7,
//! §2.8, §2.9, §2.10).
//!
//! [`run_exchange`] executes a single root conversation:
//!
//! 1. Spawn branch `agents/<conv-id>` off the default config branch's
//!    head, with a worktree under `<workspace>/agents/` (§2.2–§2.3).
//! 2. **Step 1 dispatch commit:** write `goal.md` + `soul.md`, remove
//!    the config commit's control files from the tree (§2.2), and
//!    commit — that commit's tree *is* step 1's read state (§2.10).
//! 3. **Step loop (§2.5).** At each boundary the executor drains the
//!    agent's inbox ([`drain`], §2.11), records the branch-tip sha, writes
//!    the step record under `<conv-repo>/steps/<conv-id>/<NNN>/`, and
//!    drives the model call through the retry loop ([`model_call`], §4.4).
//!    Each step re-assembles its history from the read-state commit's tree
//!    ([`assembler`], §2.3, §5); a settled `tool_use` loops, no `tool_use`
//!    is terminal.
//! 4. Every terminal event ([`result_deposit`]) deposits a result message
//!    into the parent's inbox (§2.6, §2.3 step 5) — a no-op for a root; a
//!    stop deposits `stopped` on its way out (§2.9, [`terminal`]). A normal
//!    end dispatches the terminal compactor (§2.6, §2.7); stops/exhaustion skip it (§2.4).
//! 5. **Exit protocol (§2.11):** deposit → release own lock → spawn a
//!    driver at own agent, fire-and-forget → exit. The launch is decided
//!    by epitaph value ([`terminal::exit_launch`]): a final response
//!    launches; `stopped` and `budget-exhausted` never do. The launched
//!    driver's own-branch entry — acquire-or-exit, deliver or silently
//!    no-op — is [`driver`].

pub mod advance;
mod assembler;
mod drain;
pub mod driver;
mod model_call;
mod result_deposit;
mod staging;
mod step_commit;
pub mod stop_signal;
mod terminal;
mod tool_step;
mod tools;
mod transcript;
mod transfer;

pub use model_call::{RealSleeper, Sleeper};
pub(crate) use step_commit::remove_control_files;
pub use stop_signal::{flag as stop_flag, install as install_stop_handler};

use super::inbox::{self, Epitaph};
use super::step::{RESPONSE_FILE, STAGING_FILE, StepMeta, step_dir_rel};
use super::{Deps, Error};
use crate::config::{Budgets, Model, RetryConfig};
use assembler::assemble;
use brazen::Content;
use model_call::ModelCall;
use std::ffi::OsString;
use std::path::Path;
use step_commit::{
    commit_dispatch, prepend_goal, read_branch_tip, spawn_branch, write_dispatch_files, write_meta,
    write_request,
};
use tool_step::run_tool_calls;

/// Per-request `max_tokens` output cap — one model call's output ceiling,
/// distinct from the §6 cumulative spend budgets ([`Budgets`]). Moves to
/// manifest config when that surface lands.
const DEFAULT_MAX_TOKENS: u32 = 4096;

/// Inputs resolved by [`super::run`] before branch work starts.
pub(super) struct Resolved<'a> {
    pub(super) model: &'a Model,
    /// brazen provider-row name passed as `bz --provider <row>` (§4.4).
    pub(super) provider_row: &'a str,
    /// The role's declared tool names (§4.3 `tools:`). Composed against
    /// the branch's `descriptions/tools/*.json` schemas into the typed
    /// request's `tools` array (§3.3) so the model is told its toolset.
    pub(super) tools: &'a [String],
    pub(super) soul: String,
    /// The adapter binary (`bz` or the `adapter:` override, §4.2).
    pub(super) binary: OsString,
    /// Harness-owned retry policy from `workflow.yaml` (§2.10, §6).
    pub(super) retry: RetryConfig,
    /// Per-conversation spend limits from `workflow.yaml` (§6). Checked
    /// at every model-call boundary before the adapter is invoked.
    pub(super) budgets: Budgets,
    /// True under an `adapter:` override — the MessageStart.v handshake
    /// governs in place of the version guard (§4.4).
    pub(super) expect_handshake: bool,
}

/// Drive one root conversation against an already-resolved config.
/// Returns the branch name so the caller can surface it on stdout.
pub(super) fn run_exchange(
    repo: &Path,
    user_message: &str,
    resolved: &Resolved<'_>,
    deps: &Deps<'_>,
) -> Result<String, Error> {
    let ts = deps.clock.now_compact();
    let short_id = deps.id_gen.short();
    let conv_id = format!("{ts}-{short_id}");
    let branch_name = conv_id.clone();
    let worktree_path = crate::workspace::agent_worktree(repo, &conv_id);

    // Executor lock (§2.11): acquire the branch's inbox lease before any
    // work, held for the whole loop and kernel-released on exit. Losing
    // the acquire means another driver owns this branch — clean no-op
    // (Writer/driver totality); a fresh root always wins (unique conv-id).
    let inbox = inbox::inbox_dir(repo, &conv_id);
    let executor_lock = match inbox::try_acquire(&inbox).map_err(|source| Error::ExecutorLock {
        path: inbox.clone(),
        source,
    })? {
        Some(guard) => guard,
        None => return Ok(branch_name),
    };

    spawn_branch(repo, &worktree_path, &conv_id, deps)?;

    // The initial user message enters through the front door (§2.4, §2.11):
    // deposited into this agent's own inbox and delivered by the step-1
    // drain — the same path any reprompt takes, no bespoke delivery.
    inbox::deposit(repo, &conv_id, inbox::USER_SENDER, user_message, deps.clock)?;

    let system_with_goal = prepend_goal(user_message, &resolved.soul);

    let call = ModelCall {
        adapter: deps.adapter,
        sleeper: deps.sleeper,
        binary: &resolved.binary,
        provider_row: resolved.provider_row,
        retry: resolved.retry,
        expect_handshake: resolved.expect_handshake,
    };

    let mut step_seq: u32 = 1;
    let mut exhausted = false;
    // §2.9 step 3: set when a check point sees the SIGTERM handler flag.
    let mut stopped = false;
    // §3.3/§4.3: the role's declared tools composed against the schemas
    // committed under `descriptions/tools/` (§2.10), once at step 1 and
    // cloned into every step's request (git-inherited, stable mid-branch).
    let mut tools: Vec<brazen::Tool> = Vec::new();
    loop {
        if step_seq == 1 {
            write_dispatch_files(&worktree_path, user_message, &resolved.soul)?;
            commit_dispatch(&worktree_path, &conv_id, deps)?;
            tools = tools::compose(&worktree_path, resolved.tools)?;
        }

        // Step-boundary drain (§2.11 *Delivery*): move each pending inbox
        // message into the transcript ahead of this step's read-state
        // capture, so it is part of the commit the model call assembles
        // from — and after the prior step's tool entries, so a message
        // never wedges between paired tool blocks (§2.3).
        drain::drain(&worktree_path, &inbox, &conv_id, deps.git)?;

        let commit_sha = read_branch_tip(&worktree_path, deps)?;

        // §2.9 step 3 check point: a stop between steps (or during a prior
        // step's tool work) is caught here, before the next model call.
        if stop_signal::stopped(deps) {
            stopped = true;
            break;
        }

        // §6 budget check (deposits + marks the ref on exhaustion, §2.9).
        if terminal::budget_exhausted(
            repo,
            &conv_id,
            &branch_name,
            &worktree_path,
            &resolved.budgets,
            deps,
        )? {
            exhausted = true;
            break;
        }

        // §2.3 / §5: assemble the model-facing history from the read-state
        // commit's tree — one code path for running, retry, and replay.
        let messages = assemble(&worktree_path)?;
        let request = model_call::build_request(
            &resolved.model.model_id,
            &system_with_goal,
            messages,
            tools.clone(),
            DEFAULT_MAX_TOKENS,
        );
        let request_value =
            serde_json::to_value(&request).expect("CanonicalRequest is always serializable");
        let step_dir_rel_str = step_dir_rel(&conv_id, step_seq);
        write_request(repo, &step_dir_rel_str, &request_value)?;

        let request_bytes =
            serde_json::to_vec(&request).expect("CanonicalRequest is always serializable");
        let started_at = deps.clock.now_iso8601();
        let response_path = repo.join(&step_dir_rel_str).join(RESPONSE_FILE);
        let call_outcome = model_call::run(&call, &request_bytes, &response_path);
        // §2.9 step 3 check point: a stop delivered during the call killed
        // `bz`, leaving `response.json` without a trailing `end` (the stop
        // signature, untouched here) — surfacing as `AdapterHalfStream`.
        // With the flag set it is a stop, not a failure: swallow and exit.
        if stop_signal::stopped(deps) {
            stopped = true;
            break;
        }
        call_outcome?;
        let ended_at = deps.clock.now_iso8601();

        write_meta(
            repo,
            &step_dir_rel_str,
            &StepMeta {
                commit: commit_sha,
                started_at,
                ended_at,
            },
        )?;

        // Transcript writer (§2.3): seal-and-rename the staging entry to
        // `messages/NNN-<model-id>.json` (origin = authoring model) + commit.
        let staging_path = repo.join(&step_dir_rel_str).join(STAGING_FILE);
        let assistant_content = transcript::commit_assistant(
            &worktree_path,
            &conv_id,
            &resolved.model.model_id,
            &staging_path,
            deps.git,
        )?;

        // No `tool_use` block is terminal (§2.5): deposit a `final-response`
        // result, response body iff the agent spoke (§2.6). No-op for a root.
        if !assistant_content
            .iter()
            .any(|b| matches!(b, Content::ToolUse { .. }))
        {
            let response = result_deposit::terminal_text(&assistant_content);
            result_deposit::deposit_terminal(
                repo,
                &conv_id,
                &worktree_path,
                Epitaph::FinalResponse,
                response.as_deref(),
                deps,
            )?;
            break;
        }

        // §2.5 pairing: run each tool_use, committing its tool_result as a
        // transcript entry (§2.3); the next step re-assembles from the tree.
        // §2.9 step 3 check point: a stop landing in this tool-execution
        // window (the group SIGTERM felled the running tool) breaks here for
        // the same stopped-deposit exit as the model-call window.
        if run_tool_calls(
            repo,
            &worktree_path,
            &conv_id,
            &step_dir_rel_str,
            &assistant_content,
            deps,
        )? {
            stopped = true;
            break;
        }
        step_seq += 1;
    }

    // Terminal handling (§2.7, §2.9, §6): stopped deposits and skips
    // compaction, exhausted skips it, the ordinary path dispatches the
    // terminal compactor — the one merge left (§2.6). No merge-back.
    let epitaph = match (stopped, exhausted) {
        (true, _) => Epitaph::Stopped,
        (false, true) => Epitaph::BudgetExhausted,
        (false, false) => Epitaph::FinalResponse,
    };
    terminal::finish(repo, &conv_id, &branch_name, &worktree_path, epitaph, deps)?;

    // Exit protocol (§2.11): the result deposit landed at the terminal
    // event above; now release own lock, then spawn a driver at own
    // agent, fire-and-forget, and exit. After release this process has
    // no authority — spawn and exit are its only remaining acts.
    drop(executor_lock);
    terminal::exit_launch(repo, &conv_id, epitaph, deps);

    Ok(branch_name)
}
