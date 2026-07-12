//! Root-conversation branch orchestration (ARCH §2.3, §2.5, §2.6, §2.7,
//! §2.8, §2.10).
//!
//! [`run_exchange`] executes a single root conversation off `main`:
//!
//! 1. Spawn branch `<conv-id>` off `main` with a sibling worktree (§2.2).
//! 2. **Step 1 dispatch commit:** write `goal.md` + `soul.md` and commit —
//!    that commit's tree *is* step 1's read state (§2.10).
//! 3. **Step loop (§2.5).** At each boundary the executor drains the
//!    agent's inbox ([`drain`], §2.11), records the branch-tip sha, writes
//!    the step record under `<conv-repo>/steps/<conv-id>/<NNN>/`, and
//!    drives the model call through the retry loop ([`model_call`], §4.4).
//!    Each step re-assembles its history from the read-state commit's tree
//!    ([`assembler`], §2.3, §5); a settled `tool_use` loops, no `tool_use`
//!    is terminal.
//! 4. On a normal terminal event, dispatch the terminal compactor off the
//!    tip (§2.7) — the compaction merge is the one merge left (§2.6).
//!    Merge-back is gone: the root branch persists on its own ref (§2.4).
//!    Every terminal event ([`result_deposit`]) deposits a result message
//!    into the parent's inbox (§2.6, §2.3 step 5) — a no-op for a root.

mod assembler;
mod drain;
mod model_call;
mod result_deposit;
mod staging;
mod step_commit;
mod tool_step;
mod tools;
mod transcript;
mod transfer;

pub use model_call::{RealSleeper, Sleeper};

use super::budget;
use super::inbox::{self, Epitaph};
use super::step::{RESPONSE_FILE, STAGING_FILE, StepMeta, step_dir_rel};
use super::{Deps, Error};
use crate::config::{Budgets, Model, RetryConfig};
use crate::template::ROOT_WORKTREE;
use assembler::assemble;
use brazen::Content;
use model_call::ModelCall;
use std::ffi::OsString;
use std::path::Path;
use step_commit::{
    commit_dispatch, read_branch_tip, write_dispatch_files, write_meta, write_request,
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
    let worktree_path = repo.join(&conv_id);
    let primary_worktree = repo.join(ROOT_WORKTREE);

    // Executor lock (§2.11): acquire the branch's inbox lease before any
    // work, held for the whole loop and kernel-released on exit. Losing
    // the acquire means another driver owns this branch — clean no-op
    // (Writer/driver totality); a fresh root always wins (unique conv-id).
    let inbox = inbox::inbox_dir(repo, &conv_id);
    let _executor_lock = match inbox::try_acquire(&inbox).map_err(|source| Error::ExecutorLock {
        path: inbox.clone(),
        source,
    })? {
        Some(guard) => guard,
        None => return Ok(branch_name),
    };

    spawn_branch(&primary_worktree, &worktree_path, &branch_name, deps)?;

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

        // §6 budget check at the model-call boundary: tokens/wall/depth
        // derived live over the tree (no stored counter, PRINCIPLES SSOT).
        // Exhaustion writes `refs/lernie/budget-exhausted/<branch>` and
        // ceases the loop — an ordinary terminal state.
        if let Some(ex) = budget::check(repo, &branch_name, &resolved.budgets) {
            eprintln!("lernie: budget {ex} on {branch_name}; stopping (§6)");
            budget::mark_exhausted(&worktree_path, &branch_name, deps.git).map_err(|source| {
                Error::Git {
                    op: "budget-exhausted update-ref",
                    source,
                }
            })?;
            // Terminal event (§2.3 step 5, §6): deposit a `budget-exhausted`
            // result message. The agent did not speak this step, so no body.
            result_deposit::deposit_terminal(
                repo,
                &conv_id,
                &worktree_path,
                Epitaph::BudgetExhausted,
                None,
                deps,
            )?;
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
        model_call::run(&call, &request_bytes, &response_path)?;
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

        // A step with no `tool_use` block is terminal (§2.5): deposit a
        // `final-response` result message carrying the terminal response
        // iff the agent spoke (§2.3 step 5, §2.6). No-op for a root.
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

        // §2.5 pairing: run each tool_use, committing its tool_result as
        // a transcript entry (§2.3); the next step re-assembles from the tree.
        run_tool_calls(
            repo,
            &worktree_path,
            &conv_id,
            &step_dir_rel_str,
            &assistant_content,
            deps,
        )?;
        step_seq += 1;
    }

    // An exhausted conversation is terminal-by-exhaustion (§6, §2.9): it
    // persists behind the budget-exhausted ref with no compaction.
    // Otherwise the normal terminal path dispatches the terminal
    // compactor off the tip (§2.7), whose return lands the compaction
    // merge — the one merge left in the system (§2.6) — into this branch.
    // There is no merge-back: the root branch persists on its own ref
    // (§2.4), and any child's return already rode the result-message
    // channel above (§2.6).
    if !exhausted {
        deps.dispatcher
            .dispatch("compactor", repo, &branch_name, None)
            .map_err(|source| Error::DispatchFailed {
                role: "compactor",
                source,
            })?;
    }

    Ok(branch_name)
}

/// `git worktree add -b <branch> <worktree_path> main`, run inside the
/// primary worktree where `.git` lives (§2.2).
fn spawn_branch(
    primary_worktree: &Path,
    worktree_path: &Path,
    branch_name: &str,
    deps: &Deps<'_>,
) -> Result<(), Error> {
    let wt_str = worktree_path.to_string_lossy().to_string();
    deps.git
        .run(
            primary_worktree,
            &[
                "worktree",
                "add",
                "-b",
                branch_name,
                wt_str.as_str(),
                "main",
            ],
        )
        .map_err(|source| Error::Git {
            op: "worktree add",
            source,
        })
}

/// Prepend the branch's goal to the role's soul so it sits at the head of
/// assembled context (ARCH §2.8); manifest-driven assembly replaces the
/// inline `<goal>` framing later.
fn prepend_goal(goal: &str, soul: &str) -> String {
    format!("<goal>\n{goal}\n</goal>\n\n{soul}")
}
