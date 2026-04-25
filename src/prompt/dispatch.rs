//! Root-conversation branch orchestration (ARCH §2.3, §2.6, §2.7, §2.8, §2.10).
//!
//! [`run_exchange`] executes a single root conversation off `main`:
//!
//! 1. Spawn branch `<conv-id>` (the bare hyphenated id — no `ex/`
//!    prefix per §2.3 v0.3) off `main` and allocate a sibling worktree
//!    at `<conv-repo>/<conv-id>/` (§2.2 — sibling of `root/`, never
//!    nested).
//! 2. Write `goal.md` (the user message for v0.3 per §2.8) and
//!    `soul.md` (the role's system prompt per §4.3) at the worktree
//!    root, plus `steps/<conv-id>/001/request.json`.
//! 3. Commit the snapshot — §2.10's "commit before model call" — so the
//!    commit's tree is the exact state the model call reads from.
//! 4. Invoke `describe` then `complete` on the provider adapter
//!    (§4.4). Describe is harness-wide adapter setup and happens before
//!    branch work, so an adapter fault does not leave a stray branch.
//! 5. Write `steps/<conv-id>/001/response.json` and land it as a
//!    follow-up commit on the same branch. Follow-up rather than amend
//!    keeps the snapshot's tree intact for replay.
//! 6. Dispatch the terminal compactor off the branch tip (§2.7) — the
//!    compactor is a subagent running on its own branch (a hyphenated
//!    descent of the parent's id) and merging back via the normal
//!    protocol (§2.6).
//! 7. Rebase the conversation branch onto the current `main` tip and
//!    `--no-ff` merge it into `main` (§2.6). The merge runs inside the
//!    primary worktree at `<conv-repo>/root/` since that is where
//!    `main` is checked out (§2.2). Remove the conversation worktree;
//!    the branch ref stays for the retention window (§2.3).
//!
//! Unmerged branches are enumerable via `git branch --list '*-*'
//! --no-merged main` (§8) — no sidecar state, per PRINCIPLES.md's
//! "Single source of truth".

use super::merge::rebase_and_merge;
use super::step::{REQUEST_FILE, RESPONSE_FILE, StepResponse, Usage, step_dir_rel};
use super::{Deps, Error, parse_adapter_stdout, parse_endpoint_env};
use crate::config::Model;
use crate::config::Provider as ProviderConfig;
use crate::template::ROOT_WORKTREE;
use std::ffi::OsString;
use std::path::Path;

/// The trunk branch every root conversation eventually merges into
/// (ARCH §2.3). Named as a constant so the one-path merge protocol
/// does not depend on a literal threaded through multiple call sites.
const TRUNK_BRANCH: &str = "main";

/// Per-request `max_tokens` cap. v0.3 is not opinionated about budget
/// yet — this matches v0.2's default and moves to config when the
/// manifest surface lands (v0.6).
const DEFAULT_MAX_TOKENS: u32 = 4096;
/// v0.3 has one step per root conversation; 1-indexed matches how the
/// step is spoken about in commit messages ("step 001").
const FIRST_STEP_SEQ: u32 = 1;
/// Worktree-relative path where the role's system prompt is committed
/// at dispatch time (ARCH §2.8). Lives at the worktree root so the
/// manifest's `pinned: [soul.md]` rule (§5.2) sees it.
const SOUL_FILE: &str = "soul.md";
/// Worktree-relative path where the conversation's goal is committed
/// at dispatch time (ARCH §2.8). Lives at the worktree root for the
/// same reason `soul.md` does.
const GOAL_FILE: &str = "goal.md";

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
    let binary: OsString = format!("lernie-provider-{}", resolved.provider_name).into();

    // Describe runs before any branch work so an adapter fault fails
    // fast and leaves no stray branch behind.
    let describe_bytes = deps
        .adapter
        .run(&binary, &["describe"], &[], &[])
        .map_err(Error::AdapterSpawn)?;
    let endpoint_env_names = parse_endpoint_env(&describe_bytes)?;

    let ts = deps.clock.now_compact();
    let short_id = deps.id_gen.short();
    let conv_id = format!("{ts}-{short_id}");
    let branch_name = conv_id.clone();
    let worktree_path = repo.join(&conv_id);
    let primary_worktree = repo.join(ROOT_WORKTREE);

    spawn_branch(&primary_worktree, &worktree_path, &branch_name, deps)?;

    let goal_text = user_message;
    let system_with_goal = prepend_goal(goal_text, &resolved.soul);
    let request_value = serde_json::json!({
        "model": resolved.model.model_id,
        "max_tokens": DEFAULT_MAX_TOKENS,
        "system": system_with_goal,
        "messages": [{"role": "user", "content": user_message}],
    });

    let step_dir_rel_str = step_dir_rel(&conv_id, FIRST_STEP_SEQ);
    write_snapshot(
        &worktree_path,
        goal_text,
        &resolved.soul,
        &step_dir_rel_str,
        &request_value,
    )?;
    commit_snapshot(&worktree_path, &step_dir_rel_str, &conv_id, deps)?;

    let endpoint_envs: Vec<(&str, &str)> = endpoint_env_names
        .iter()
        .map(|name| (name.as_str(), resolved.provider.endpoint.as_str()))
        .collect();
    let request_bytes = serde_json::to_vec(&request_value).expect("Value is always serializable");

    let started_at = deps.clock.now_iso8601();
    let complete_stdout = deps
        .adapter
        .run(&binary, &["complete"], &endpoint_envs, &request_bytes)
        .map_err(Error::AdapterSpawn)?;
    let ended_at = deps.clock.now_iso8601();

    let response = parse_adapter_stdout(&complete_stdout)?;
    let step_response = StepResponse {
        assistant_response: response.text(),
        model_id: resolved.model.model_id.clone(),
        provider: resolved.provider_name.to_string(),
        usage: Usage {
            input_tokens: response.usage.input_tokens,
            output_tokens: response.usage.output_tokens,
        },
        stop_reason: response.stop_reason.clone(),
        started_at,
        ended_at,
    };
    write_response(&worktree_path, &step_dir_rel_str, &step_response)?;
    commit_response(&worktree_path, &step_dir_rel_str, &conv_id, deps)?;

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

/// Write `goal.md`, `soul.md`, and the step's `request.json` into the
/// conversation branch's worktree. Called inside the worktree so the
/// writes land on the branch's tree, not `main`'s.
fn write_snapshot(
    worktree_path: &Path,
    goal_text: &str,
    soul_text: &str,
    step_dir_rel_str: &str,
    request_value: &serde_json::Value,
) -> Result<(), Error> {
    std::fs::create_dir_all(worktree_path)?;
    std::fs::write(worktree_path.join(GOAL_FILE), goal_text)?;
    std::fs::write(worktree_path.join(SOUL_FILE), soul_text)?;

    let step_dir_abs = worktree_path.join(step_dir_rel_str);
    std::fs::create_dir_all(&step_dir_abs)?;
    let request_bytes =
        serde_json::to_vec_pretty(request_value).expect("Value is always serializable");
    std::fs::write(step_dir_abs.join(REQUEST_FILE), request_bytes)?;
    Ok(())
}

/// `git add` the goal + soul + request then `git commit` the snapshot.
/// The commit message names the step and conversation so history stays
/// legible to `git log` alone.
fn commit_snapshot(
    worktree_path: &Path,
    step_dir_rel_str: &str,
    conv_id: &str,
    deps: &Deps<'_>,
) -> Result<(), Error> {
    let request_rel_str = format!("{step_dir_rel_str}/{REQUEST_FILE}");
    deps.git
        .run(
            worktree_path,
            &["add", GOAL_FILE, SOUL_FILE, request_rel_str.as_str()],
        )
        .map_err(|source| Error::Git { op: "add", source })?;
    let msg = format!("step {FIRST_STEP_SEQ:03}: dispatch [{conv_id}]");
    deps.git
        .run(worktree_path, &["commit", "-m", msg.as_str()])
        .map_err(|source| Error::Git {
            op: "commit",
            source,
        })
}

/// Write the parsed response to `response.json` on the branch's tree.
fn write_response(
    worktree_path: &Path,
    step_dir_rel_str: &str,
    step_response: &StepResponse,
) -> Result<(), Error> {
    let step_dir_abs = worktree_path.join(step_dir_rel_str);
    let response_bytes =
        serde_json::to_vec_pretty(step_response).expect("StepResponse is always serializable");
    std::fs::write(step_dir_abs.join(RESPONSE_FILE), response_bytes)?;
    Ok(())
}

/// Follow-up commit: `git add` the response file then commit. Does
/// not amend the snapshot so the snapshot's tree keeps reflecting
/// pre-model-call state (§2.10 replay).
fn commit_response(
    worktree_path: &Path,
    step_dir_rel_str: &str,
    conv_id: &str,
    deps: &Deps<'_>,
) -> Result<(), Error> {
    let response_rel_str = format!("{step_dir_rel_str}/{RESPONSE_FILE}");
    deps.git
        .run(worktree_path, &["add", response_rel_str.as_str()])
        .map_err(|source| Error::Git { op: "add", source })?;
    let msg = format!("step {FIRST_STEP_SEQ:03}: response [{conv_id}]");
    deps.git
        .run(worktree_path, &["commit", "-m", msg.as_str()])
        .map_err(|source| Error::Git {
            op: "commit",
            source,
        })
}
