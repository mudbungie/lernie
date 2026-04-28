//! Root-conversation branch orchestration (ARCH §2.3, §2.5, §2.6, §2.7,
//! §2.8, §2.10).
//!
//! [`run_exchange`] executes a single root conversation off `main`:
//!
//! 1. Spawn branch `<conv-id>` (the bare hyphenated id — no `ex/`
//!    prefix per §2.3 v0.3) off `main` and allocate a sibling worktree
//!    at `<conv-repo>/<conv-id>/` (§2.2 — sibling of `root/`, never
//!    nested).
//! 2. **Step 1 dispatch commit:** write `goal.md` + `soul.md` at the
//!    worktree root and commit. That commit's tree *is* step 1's
//!    read state (§2.10). Step ≥2 takes no pre-call commit — the
//!    branch tip already reflects what the model reads.
//! 3. For each step: capture the branch-tip sha (the read state),
//!    write `request.json` + `meta.json` to
//!    `<conv-repo>/steps/<conv-id>/<NNN>/` (outside every worktree,
//!    §2.2 / §2.3), invoke `complete` with `stream: true` on the
//!    provider adapter (§4.4), and tail its stdout into
//!    `response.json` line-by-line as JSONL of §4.4 stream events.
//!    Closing the response.json fd at terminal `message_stop` /
//!    `error` is the §3.5 IN_CLOSE_WRITE completion signal. None of
//!    these artifacts is committed — they are diagnostic-only (§2.3).
//! 4. **Step loop (§2.5).** If `stop_reason == "tool_use"`, the
//!    harness runs every emitted `tool_use` block through
//!    [`crate::prompt::ToolExecutor`] (per-call records land at
//!    `<conv-repo>/steps/<conv-id>/<NNN>/tools/<tool-id>/`, also
//!    outside every worktree per §3.3), then assembles a follow-up
//!    step whose user message carries one `tool_result` block per
//!    emitted call. The loop terminates when `stop_reason` is
//!    anything else (`end_turn`, `max_tokens`, …).
//! 5. Dispatch the terminal compactor off the branch tip (§2.7).
//! 6. Rebase the conversation branch onto the current `main` tip and
//!    `--no-ff` merge it into `main` (§2.6). The merge runs inside the
//!    primary worktree at `<conv-repo>/root/` since that is where
//!    `main` is checked out (§2.2). Remove the conversation worktree;
//!    the branch ref stays for the retention window (§2.3).
//!
//! Unmerged branches are enumerable via `git branch --list '*-*'
//! --no-merged main` (§8) — no sidecar state, per PRINCIPLES.md's
//! "Single source of truth".

mod assembler;
mod step_commit;
mod stream;
mod tool_step;

use super::adapter;
use super::merge::rebase_and_merge;
use super::step::{RESPONSE_FILE, StepMeta, step_dir_rel};
use super::{Deps, Error, parse_endpoint_env};
use crate::config::Model;
use crate::config::Provider as ProviderConfig;
use crate::template::ROOT_WORKTREE;
use serde_json::{Value, json};
use std::path::Path;
use step_commit::{
    commit_dispatch, read_branch_tip, write_dispatch_files, write_meta, write_request,
};
use tool_step::run_tool_calls;

/// The trunk branch every root conversation eventually merges into
/// (ARCH §2.3). Named as a constant so the one-path merge protocol
/// does not depend on a literal threaded through multiple call sites.
const TRUNK_BRANCH: &str = "main";

/// Per-request `max_tokens` cap. v0.3 is not opinionated about budget
/// yet — this matches v0.2's default and moves to config when the
/// manifest surface lands (v0.6).
const DEFAULT_MAX_TOKENS: u32 = 4096;
/// Wire-level `stop_reason` value that drives another loop iteration
/// (§2.5). Any other value terminates the loop and lets the parent
/// compactor + merge-back run.
const STOP_REASON_TOOL_USE: &str = "tool_use";

/// Inputs resolved by [`super::run`] before branch work starts.
pub(super) struct Resolved<'a> {
    pub(super) model: &'a Model,
    pub(super) provider_name: &'a str,
    pub(super) provider: &'a ProviderConfig,
    pub(super) soul: String,
}

/// Drive one root conversation against an already-resolved config.
/// Returns the branch name so the caller can surface it on stdout.
pub(super) fn run_exchange(
    repo: &Path,
    user_message: &str,
    resolved: &Resolved<'_>,
    deps: &Deps<'_>,
) -> Result<String, Error> {
    let binary = adapter::resolve_binary(deps.harness_root, resolved.provider_name);

    // Describe runs before any branch work so an adapter fault fails
    // fast and leaves no stray branch behind. `describe` writes one
    // JSON line to stdout — collect that line through the same
    // streaming runner the rest of the harness uses.
    let mut describe_bytes: Vec<u8> = Vec::new();
    deps.adapter
        .run(&binary, &["describe"], &[], &[], &mut |line| {
            if !describe_bytes.is_empty() {
                describe_bytes.push(b'\n');
            }
            describe_bytes.extend_from_slice(line);
            Ok(())
        })
        .map_err(Error::AdapterSpawn)?;
    let endpoint_env_names = parse_endpoint_env(&describe_bytes)?;

    let ts = deps.clock.now_compact();
    let short_id = deps.id_gen.short();
    let conv_id = format!("{ts}-{short_id}");
    let branch_name = conv_id.clone();
    let worktree_path = repo.join(&conv_id);
    let primary_worktree = repo.join(ROOT_WORKTREE);

    spawn_branch(&primary_worktree, &worktree_path, &branch_name, deps)?;

    let system_with_goal = prepend_goal(user_message, &resolved.soul);
    // The growing wire-shape `messages` list across loop iterations
    // (§2.5). Step 1's user message is a bare string; tool-result
    // follow-ups append blocks-shape content.
    let mut messages: Vec<Value> = vec![json!({"role": "user", "content": user_message})];
    let endpoint_envs: Vec<(&str, &str)> = endpoint_env_names
        .iter()
        .map(|name| (name.as_str(), resolved.provider.endpoint.as_str()))
        .collect();

    let mut step_seq: u32 = 1;
    loop {
        // Step 1 lands its dispatch commit (goal.md + soul.md) before
        // anything else; that commit's tree is step 1's read state
        // (§2.10). Step ≥2 takes no pre-call commit — the branch tip
        // already reflects what the model reads.
        if step_seq == 1 {
            write_dispatch_files(&worktree_path, user_message, &resolved.soul)?;
            commit_dispatch(&worktree_path, &conv_id, deps)?;
        }

        // The branch-tip sha at step-start is the model's read state
        // (§2.10). Recorded in meta.json so replay can re-run context
        // assembly against it without reading the diagnostic
        // request.json (§2.3).
        let commit_sha = read_branch_tip(&worktree_path, deps)?;

        let request_value = stream::build_request(
            &resolved.model.model_id,
            &system_with_goal,
            &messages,
            DEFAULT_MAX_TOKENS,
        );
        let step_dir_rel_str = step_dir_rel(&conv_id, step_seq);
        write_request(repo, &step_dir_rel_str, &request_value)?;

        let request_bytes =
            serde_json::to_vec(&request_value).expect("Value is always serializable");
        let started_at = deps.clock.now_iso8601();
        let response_path = repo.join(&step_dir_rel_str).join(RESPONSE_FILE);
        let completion = stream::run_complete(
            deps.adapter,
            &binary,
            &endpoint_envs,
            &request_bytes,
            &response_path,
        )?;
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

        if completion.stop_reason != STOP_REASON_TOOL_USE {
            break;
        }

        // §2.5 pairing: every tool_use gets a matching tool_result on
        // the next user message. Run them in emission order so the
        // tool record sequence matches `completion.content[]`.
        let assistant_blocks =
            serde_json::to_value(&completion.content).expect("ContentBlock serializes");
        let tool_results = run_tool_calls(repo, &step_dir_rel_str, &completion.content, deps)?;
        messages.push(json!({"role": "assistant", "content": assistant_blocks}));
        messages.push(json!({"role": "user", "content": tool_results}));
        step_seq += 1;
    }

    // Terminal compaction (§2.7) + merge-back to main (§2.6). The
    // compactor is a subagent dispatched off the branch tip via the
    // same primitive v0.4 will use generally — see ARCH §2.5 on why
    // one dispatch primitive serves both. §3.4 puts the dispatch on
    // the CLI: the harness re-enters `lernie dispatch compactor`
    // rather than calling the compactor module directly.
    deps.dispatcher
        .dispatch_compactor(repo, &branch_name)
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

    Ok(branch_name)
}

/// `git worktree add -b <branch> <worktree_path> main` — creates the
/// branch ref and checks it out at `worktree_path`. Run inside the
/// primary worktree (`<conv-repo>/root/`) since that is where the
/// `.git` directory and the `main` ref live (§2.2).
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

/// Prepend the branch's goal to the role's soul. ARCH §2.8 pins the
/// goal at the head of the assembled context; v0.3's "minimal" context
/// assembly realizes that by inlining it into `system` as an explicit
/// `<goal>` block. v0.6 replaces this with manifest.yaml-driven
/// assembly.
fn prepend_goal(goal: &str, soul: &str) -> String {
    format!("<goal>\n{goal}\n</goal>\n\n{soul}")
}
