//! Per-step on-disk landings for the conversation branch.
//!
//! These helpers materialize the §2.3 / §2.10 invariants: the snapshot
//! commit lands the model-call input *before* the call is issued, and
//! the response follow-up lands the parsed output as a separate
//! commit (so the snapshot's tree continues to reflect pre-model-call
//! state for replay). Step 1 of a conversation additionally lays
//! `goal.md` and `soul.md` at the worktree root, the dispatch-commit
//! shape from §2.3 step 2; later steps add only their `request.json`.
//!
//! Living in a sibling module keeps the loop body in `super`'s
//! `run_exchange` under the repo's 300-line code-file cap without
//! splitting the orchestration logic itself.

use crate::prompt::Deps;
use crate::prompt::Error;
use crate::prompt::step::{REQUEST_FILE, RESPONSE_FILE, StepResponse};
use serde_json::Value;
use std::path::Path;

/// Worktree-relative path where the conversation's goal is committed
/// at dispatch time (ARCH §2.8). Lives at the worktree root so the
/// manifest's `pinned: [goal.md]` rule (§5.2) sees it.
pub(super) const GOAL_FILE: &str = "goal.md";
/// Worktree-relative path where the role's system prompt is committed
/// at dispatch time (ARCH §4.3 / §2.8). Lives at the worktree root for
/// the same reason `goal.md` does.
pub(super) const SOUL_FILE: &str = "soul.md";

/// Step 1 lays down `goal.md` + `soul.md` + `request.json`; subsequent
/// steps lay down only `request.json` (the goal/soul are inherited
/// across the branch's history and merge=ours-disciplined per §2.6).
pub(super) fn write_snapshot(
    worktree_path: &Path,
    goal_text: &str,
    soul_text: &str,
    step_dir_rel_str: &str,
    request_value: &Value,
    step_seq: u32,
) -> Result<(), Error> {
    std::fs::create_dir_all(worktree_path)?;
    if step_seq == 1 {
        std::fs::write(worktree_path.join(GOAL_FILE), goal_text)?;
        std::fs::write(worktree_path.join(SOUL_FILE), soul_text)?;
    }

    let step_dir_abs = worktree_path.join(step_dir_rel_str);
    std::fs::create_dir_all(&step_dir_abs)?;
    let request_bytes =
        serde_json::to_vec_pretty(request_value).expect("Value is always serializable");
    std::fs::write(step_dir_abs.join(REQUEST_FILE), request_bytes)?;
    Ok(())
}

/// `git add` the snapshot files then `git commit`. Step 1 stages
/// goal/soul/request together as the "dispatch" commit (§2.3); steps
/// >1 stage only the new request.json as a "request" commit.
pub(super) fn commit_snapshot(
    worktree_path: &Path,
    step_dir_rel_str: &str,
    conv_id: &str,
    step_seq: u32,
    deps: &Deps<'_>,
) -> Result<(), Error> {
    let request_rel_str = format!("{step_dir_rel_str}/{REQUEST_FILE}");
    let add_args: Vec<&str> = if step_seq == 1 {
        vec!["add", GOAL_FILE, SOUL_FILE, request_rel_str.as_str()]
    } else {
        vec!["add", request_rel_str.as_str()]
    };
    deps.git
        .run(worktree_path, &add_args)
        .map_err(|source| Error::Git { op: "add", source })?;
    let role = if step_seq == 1 { "dispatch" } else { "request" };
    let msg = format!("step {step_seq:03}: {role} [{conv_id}]");
    deps.git
        .run(worktree_path, &["commit", "-m", msg.as_str()])
        .map_err(|source| Error::Git {
            op: "commit",
            source,
        })
}

/// Write the parsed response to `response.json` on the branch's tree.
pub(super) fn write_response(
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
pub(super) fn commit_response(
    worktree_path: &Path,
    step_dir_rel_str: &str,
    conv_id: &str,
    step_seq: u32,
    deps: &Deps<'_>,
) -> Result<(), Error> {
    let response_rel_str = format!("{step_dir_rel_str}/{RESPONSE_FILE}");
    deps.git
        .run(worktree_path, &["add", response_rel_str.as_str()])
        .map_err(|source| Error::Git { op: "add", source })?;
    let msg = format!("step {step_seq:03}: response [{conv_id}]");
    deps.git
        .run(worktree_path, &["commit", "-m", msg.as_str()])
        .map_err(|source| Error::Git {
            op: "commit",
            source,
        })
}
