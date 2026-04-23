//! Exchange-branch orchestration (ARCH §2.3, §2.8, §2.10).
//!
//! [`run_exchange`] executes a single exchange off `main`:
//!
//! 1. Spawn branch `ex/<ts>-<short-id>` off `main` and allocate a
//!    worktree at `<repo>/.lernie/worktrees/ex/<ts>-<short-id>/`.
//! 2. Write `.agent/goal.md` (goal = the user message for v0.2, per
//!    §2.8) and `exchanges/<id>/steps/001/request.json`.
//! 3. Commit the snapshot — §2.10's "commit before model call" — so the
//!    commit's tree is the exact state the model call reads from.
//! 4. Invoke `describe` then `complete` on the provider adapter
//!    (§4.4). Describe is harness-wide adapter setup and happens before
//!    branch work, so an adapter fault does not leave a stray branch.
//! 5. Write `exchanges/<id>/steps/001/response.json` and land it as a
//!    follow-up commit on the same branch. Follow-up rather than amend
//!    keeps the snapshot's tree intact for replay.
//!
//! The exchange branch is left open at the end — merge-back is §2.6
//! and lives in a separate task. Unmerged branches are enumerable via
//! `git branch --list ex/*` (see PRINCIPLES.md "Single source of
//! truth" — no mirror file, git's ref database IS the tracking).

use super::step::{REQUEST_FILE, RESPONSE_FILE, StepResponse, Usage, step_dir_rel};
use super::{AGENT_DIR, Deps, Error, parse_adapter_stdout, parse_endpoint_env};
use crate::config::Model;
use crate::config::Provider as ProviderConfig;
use std::ffi::OsString;
use std::path::{Path, PathBuf};

/// Per-request `max_tokens` cap. v0.2 is not opinionated about budget
/// yet — this matches v0.1's default and moves to config when the
/// manifest surface lands (v0.6).
const DEFAULT_MAX_TOKENS: u32 = 4096;
/// Top-level directory inside the conversation repo that holds
/// exchange-branch worktrees. Gitignored in the template so `main`
/// never sees it.
const WORKTREES_DIR: &str = ".lernie/worktrees";
/// Branch-name prefix for exchange branches (ARCH §2.3). A separate
/// prefix (`inv/`) covers invocation branches when v0.4 lands.
const EXCHANGE_BRANCH_PREFIX: &str = "ex";
/// v0.2 has one step per exchange; 1-indexed matches how the step is
/// spoken about in commit messages ("step 001").
const FIRST_STEP_SEQ: u32 = 1;

/// Inputs resolved by [`super::run`] before branch work starts.
pub(super) struct Resolved<'a> {
    pub(super) model: &'a Model,
    pub(super) provider_name: &'a str,
    pub(super) provider: &'a ProviderConfig,
    pub(super) system_prompt: String,
}

/// Drive one exchange against an already-resolved config. Returns the
/// branch name so the caller can surface it on stdout.
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
    let exchange_id = format!("{ts}-{short_id}");
    let branch_name = format!("{EXCHANGE_BRANCH_PREFIX}/{exchange_id}");
    let worktree_rel = format!("{WORKTREES_DIR}/{EXCHANGE_BRANCH_PREFIX}/{exchange_id}");
    let worktree_path = repo.join(worktree_rel);

    spawn_branch(repo, &worktree_path, &branch_name, deps)?;

    let goal_text = user_message;
    let system_with_goal = prepend_goal(goal_text, &resolved.system_prompt);
    let request_value = serde_json::json!({
        "model": resolved.model.model_id,
        "max_tokens": DEFAULT_MAX_TOKENS,
        "system": system_with_goal,
        "messages": [{"role": "user", "content": user_message}],
    });

    let step_dir_rel_str = step_dir_rel(&exchange_id, FIRST_STEP_SEQ);
    write_snapshot(&worktree_path, goal_text, &step_dir_rel_str, &request_value)?;
    commit_snapshot(&worktree_path, &step_dir_rel_str, &exchange_id, deps)?;

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
    commit_response(&worktree_path, &step_dir_rel_str, &exchange_id, deps)?;

    Ok(branch_name)
}

/// `git worktree add -b <branch> <worktree_path> main` — creates the
/// branch ref and checks it out at `worktree_path`.
fn spawn_branch(
    repo: &Path,
    worktree_path: &Path,
    branch_name: &str,
    deps: &Deps<'_>,
) -> Result<(), Error> {
    let wt_str = worktree_path.to_string_lossy().to_string();
    deps.git
        .run(
            repo,
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

/// Prepend the branch's goal to the role's system prompt. ARCH §2.8
/// pins the goal at the head of the assembled context; v0.2's
/// "minimal" context assembly realizes that by inlining it into
/// `system` as an explicit `<goal>` block. v0.6 replaces this with
/// manifest.yaml-driven assembly.
fn prepend_goal(goal: &str, system_prompt: &str) -> String {
    format!("<goal>\n{goal}\n</goal>\n\n{system_prompt}")
}

/// Write `.agent/goal.md` and the step's `request.json` into the
/// exchange branch's worktree. Called inside the worktree so the
/// writes land on the branch's tree, not `main`'s.
fn write_snapshot(
    worktree_path: &Path,
    goal_text: &str,
    step_dir_rel_str: &str,
    request_value: &serde_json::Value,
) -> Result<(), Error> {
    let agent_dir = worktree_path.join(AGENT_DIR);
    std::fs::create_dir_all(&agent_dir)?;
    std::fs::write(agent_dir.join("goal.md"), goal_text)?;

    let step_dir_abs = worktree_path.join(step_dir_rel_str);
    std::fs::create_dir_all(&step_dir_abs)?;
    let request_bytes =
        serde_json::to_vec_pretty(request_value).expect("Value is always serializable");
    std::fs::write(step_dir_abs.join(REQUEST_FILE), request_bytes)?;
    Ok(())
}

/// `git add` the goal + request then `git commit` the snapshot. The
/// commit message names the step and exchange so history is readable
/// without a separate branches.json lookup.
fn commit_snapshot(
    worktree_path: &Path,
    step_dir_rel_str: &str,
    exchange_id: &str,
    deps: &Deps<'_>,
) -> Result<(), Error> {
    let goal_rel = PathBuf::from(AGENT_DIR).join("goal.md");
    let goal_rel_str = goal_rel.to_string_lossy().to_string();
    let request_rel_str = format!("{step_dir_rel_str}/{REQUEST_FILE}");
    deps.git
        .run(
            worktree_path,
            &["add", goal_rel_str.as_str(), request_rel_str.as_str()],
        )
        .map_err(|source| Error::Git { op: "add", source })?;
    let msg = format!("step {FIRST_STEP_SEQ:03}: dispatch [ex {exchange_id}]");
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
    exchange_id: &str,
    deps: &Deps<'_>,
) -> Result<(), Error> {
    let response_rel_str = format!("{step_dir_rel_str}/{RESPONSE_FILE}");
    deps.git
        .run(worktree_path, &["add", response_rel_str.as_str()])
        .map_err(|source| Error::Git { op: "add", source })?;
    let msg = format!("step {FIRST_STEP_SEQ:03}: response [ex {exchange_id}]");
    deps.git
        .run(worktree_path, &["commit", "-m", msg.as_str()])
        .map_err(|source| Error::Git {
            op: "commit",
            source,
        })
}
