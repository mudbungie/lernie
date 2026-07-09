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
//! 3. For each step: record the branch-tip sha, write `request.json` +
//!    `meta.json` under `<conv-repo>/steps/<conv-id>/<NNN>/` (outside
//!    every worktree, §2.2 / §2.3), and drive the model call through the
//!    harness-owned retry loop ([`model_call`]): `bz --json --provider
//!    <row>` per attempt, request on stdin, each attempt's stdout
//!    appended verbatim to `response.json` (§4.4). Closing the
//!    `response.json` fd at step resolution is the §3.5 IN_CLOSE_WRITE
//!    completion signal.
//! 4. **Step loop (§2.5).** If the completion's terminal reason is
//!    `Finish{ToolUse}`, run every emitted `tool_use` block through the
//!    [`crate::prompt::ToolExecutor`], then assemble a follow-up step
//!    whose user message carries one `tool_result` block per call. Any
//!    other terminal reason ends the loop.
//! 5. Dispatch the terminal compactor off the tip (§2.7).
//! 6. Rebase onto `main` and `--no-ff` merge (§2.6).

mod assembler;
mod model_call;
mod step_commit;
mod tool_step;
mod tools;

pub use model_call::{RealSleeper, Sleeper};

use super::budget;
use super::merge::rebase_and_merge;
use super::step::{RESPONSE_FILE, StepMeta, step_dir_rel};
use super::{Deps, Error};
use crate::config::{Budgets, Model, RetryConfig};
use crate::template::ROOT_WORKTREE;
use brazen::{Content, Message, Role};
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

    spawn_branch(&primary_worktree, &worktree_path, &branch_name, deps)?;

    let system_with_goal = prepend_goal(user_message, &resolved.soul);
    // The growing canonical-request `messages` list across loop
    // iterations (§2.5). Step 1's user message is bare text; tool-result
    // follow-ups append `ToolResult` blocks.
    let mut messages: Vec<Message> = vec![user_message_block(user_message)];

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
    // committed under `descriptions/tools/` in the branch's read-state
    // tree (§2.10). Composed once at step 1 — after the dispatch commit
    // establishes that tree — and cloned into every step's request, since
    // the schemas are git-inherited and do not change mid-branch.
    let mut tools: Vec<brazen::Tool> = Vec::new();
    loop {
        if step_seq == 1 {
            write_dispatch_files(&worktree_path, user_message, &resolved.soul)?;
            commit_dispatch(&worktree_path, &conv_id, deps)?;
            tools = tools::compose(&worktree_path, resolved.tools)?;
        }

        let commit_sha = read_branch_tip(&worktree_path, deps)?;

        // §6 budget check at the model-call boundary, before invoking the
        // adapter. Spend/wall/depth are derived from disk across the
        // branch and its descent — no stored counter (PRINCIPLES SSOT).
        // On exhaustion the harness writes the git-native
        // `refs/lernie/budget-exhausted/<branch>` marker and ceases the
        // loop: an ordinary terminal state (§6), classified by `await`
        // like any other. The root has no parent to clamp against, so its
        // declared budget is its effective budget.
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

        let request = model_call::build_request(
            &resolved.model.model_id,
            &system_with_goal,
            messages.clone(),
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
        let completion = model_call::run(&call, &request_bytes, &response_path)?;
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

        if !completion.is_tool_use() {
            break;
        }

        // §2.5 pairing: every tool_use gets a matching tool_result on
        // the next user message, in emission order.
        let assistant_content = completion.content;
        let tool_results = run_tool_calls(repo, &step_dir_rel_str, &assistant_content, deps)?;
        messages.push(Message {
            role: Role::Assistant,
            content: assistant_content,
        });
        messages.push(Message {
            role: Role::User,
            content: tool_results,
        });
        step_seq += 1;
    }

    // An exhausted conversation is terminal-by-exhaustion (§6, §2.9): it
    // does not compact or merge back — it persists unmerged behind the
    // budget-exhausted ref. Otherwise: terminal compaction (§2.7) +
    // merge-back (§2.6), both via the CLI control plane (§3.4).
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

/// A bare-text user message (§4.4 — a `Content::Text`).
fn user_message_block(text: &str) -> Message {
    Message {
        role: Role::User,
        content: vec![Content::Text(text.to_string())],
    }
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
