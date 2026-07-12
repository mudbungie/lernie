//! Root-conversation branch orchestration (ARCH §2.3, §2.5, §2.6, §2.7,
//! §2.8, §2.10).
//!
//! [`run_exchange`] executes a single root conversation off `main`:
//!
//! 1. Spawn branch `<conv-id>` off `main` and allocate a sibling
//!    worktree at `<conv-repo>/<conv-id>/` (§2.2).
//! 2. **Step 1 dispatch commit:** write `goal.md` + `soul.md` and
//!    commit — that commit's tree *is* step 1's read state (§2.10).
//!    Step ≥2 takes no pre-call commit.
//! 3. **Step loop (§2.5).** At each boundary the executor drains the
//!    agent's inbox ([`drain`], §2.11) — pending messages land as delivery
//!    commits ahead of the read state — then records the branch-tip sha,
//!    writes `request.json` + `meta.json` under
//!    `<conv-repo>/steps/<conv-id>/<NNN>/` (§2.2 / §2.3), and drives the
//!    model call through the retry loop ([`model_call`], §4.4). Each step
//!    re-assembles its history from the read-state commit's tree
//!    ([`assembler`], §2.3, §5); a settled `tool_use` block runs through
//!    [`crate::prompt::ToolExecutor`], committing each `tool_result` as a
//!    transcript entry, then loops; no `tool_use` is terminal.
//! 4. Dispatch the terminal compactor off the tip (§2.7), then rebase
//!    onto `main` and `--no-ff` merge (§2.6).

mod assembler;
mod drain;
mod model_call;
mod staging;
mod step_commit;
mod tool_step;
mod tools;
mod transcript;

pub use model_call::{RealSleeper, Sleeper};

use super::budget;
use super::inbox;
use super::merge::rebase_and_merge;
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

/// The trunk branch every root conversation eventually merges into
/// (ARCH §2.3).
const TRUNK_BRANCH: &str = "main";

/// Per-request `max_tokens` output cap — one model call's output
/// ceiling. Distinct from the §6 spend budgets ([`Budgets`]), which
/// bound *cumulative* tokens across the conversation tree; this is a
/// per-call limit. Moves to manifest config when that surface lands; the
/// row's brazen default still applies if a provider requires one.
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
    // work. Held for the whole step loop and released by the kernel on
    // exit. Losing the acquire means another driver already owns this
    // branch — exit as a clean no-op (Writer/driver totality). A fresh
    // root prompt always wins (its conv-id is unique), so the None arm is
    // the re-entry/concurrency guard, not the common path.
    let inbox = inbox::inbox_dir(repo, &conv_id);
    let _executor_lock = match inbox::try_acquire(&inbox).map_err(|source| Error::ExecutorLock {
        path: inbox.clone(),
        source,
    })? {
        Some(guard) => guard,
        None => return Ok(branch_name),
    };

    spawn_branch(&primary_worktree, &worktree_path, &branch_name, deps)?;

    // The initial user message enters through the front door (§2.4,
    // §2.11): deposit it into this agent's own inbox and let the step-1
    // boundary drain (below) deliver it — the same path any reprompt takes,
    // so there is no bespoke initial-message delivery beside the drain.
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
    // committed under `descriptions/tools/` in the read-state tree (§2.10).
    // Composed once at step 1 — after the dispatch commit establishes that
    // tree — and cloned into every step's request (schemas are git-
    // inherited and do not change mid-branch).
    let mut tools: Vec<brazen::Tool> = Vec::new();
    loop {
        if step_seq == 1 {
            write_dispatch_files(&worktree_path, user_message, &resolved.soul)?;
            commit_dispatch(&worktree_path, &conv_id, deps)?;
            tools = tools::compose(&worktree_path, resolved.tools)?;
        }

        // Step-boundary drain (§2.11 *Delivery*): move each pending inbox
        // message into the transcript and commit it, ahead of this step's
        // read-state capture so a delivered message is part of the commit
        // the model call assembles from (§2.3, §2.10). Ordered after the
        // prior step's tool entries (loop tail) and before the rev-parse
        // (§2.3), so a message never wedges between paired tool blocks.
        drain::drain(&worktree_path, &inbox, &conv_id, deps.git)?;

        let commit_sha = read_branch_tip(&worktree_path, deps)?;

        // §6 budget check at the model-call boundary, before invoking the
        // adapter. Tokens/wall are derived live over the conversation tree
        // and depth over this branch — no stored counter (PRINCIPLES SSOT;
        // `steps/` is shared per §2.2/§2.3). On exhaustion the harness
        // writes the git-native `refs/lernie/budget-exhausted/<branch>`
        // marker and ceases the loop: an ordinary terminal state (§6) that
        // deposits a result message like any other terminal event (§2.11).
        if let Some(ex) = budget::check(repo, &branch_name, &resolved.budgets) {
            eprintln!("lernie: budget {ex} on {branch_name}; stopping (§6)");
            budget::mark_exhausted(&worktree_path, &branch_name, deps.git).map_err(|source| {
                Error::Git {
                    op: "budget-exhausted update-ref",
                    source,
                }
            })?;
            exhausted = true;
            break;
        }

        // §2.3 / §5: assemble the model-facing history from the
        // read-state commit's tree (the worktree checkout equals that
        // commit at step start). One code path for running, retry, and
        // replay — no in-memory history, no git-log walk.
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

        // Transcript writer (§2.3): the model call settled complete, so
        // seal-and-rename the staging entry into `messages/NNN-assistant.json`
        // and commit it — the entry that advances the branch tip for the
        // next step's read state (§2.10). The committed blocks come back
        // as this step's assistant content (their one home, §2.3).
        let staging_path = repo.join(&step_dir_rel_str).join(STAGING_FILE);
        let assistant_content =
            transcript::commit_assistant(&worktree_path, &conv_id, &staging_path, deps.git)?;

        // The step continues iff it emitted tool calls to resolve (§2.5);
        // a step with no `tool_use` block is terminal.
        let has_tool_use = assistant_content
            .iter()
            .any(|block| matches!(block, Content::ToolUse { .. }));
        if !has_tool_use {
            break;
        }

        // §2.5 pairing: run each tool_use, committing its tool_result as
        // a transcript entry (§2.3). The next step re-assembles the whole
        // history from the tree — nothing accumulates in memory.
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
    // persists unmerged behind the budget-exhausted ref. Otherwise:
    // terminal compaction (§2.7) + merge-back (§2.6), via the CLI (§3.4).
    if !exhausted {
        deps.dispatcher
            .dispatch("compactor", repo, &branch_name, None)
            .map_err(|source| Error::DispatchFailed {
                role: "compactor",
                source,
            })?;

        rebase_and_merge(
            &primary_worktree,
            TRUNK_BRANCH,
            &primary_worktree,
            &worktree_path,
            &branch_name,
            deps.git,
        )?;
    }

    Ok(branch_name)
}

/// `git worktree add -b <branch> <worktree_path> main` — run inside the
/// primary worktree (`<conv-repo>/root/`) where `.git` lives (§2.2).
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

/// Prepend the branch's goal to the role's soul so it sits at the head
/// of assembled context (ARCH §2.8). v0.6 keeps the inline `<goal>`
/// framing; manifest-driven assembly replaces it later.
fn prepend_goal(goal: &str, soul: &str) -> String {
    format!("<goal>\n{goal}\n</goal>\n\n{soul}")
}
