//! Per-step on-disk landings (ARCH §2.3 / §2.10).
//!
//! Step records live at `<conv-repo>/steps/<conv-id>/<NNN>/`,
//! outside every worktree (§2.2). The harness writes them as
//! diagnostic / audit artifacts and does not read them back at
//! runtime (§2.3 Diagnostic-only contract).
//!
//! Step 1's dispatch commit lays `goal.md` and `soul.md` at the
//! worktree root and commits — that single commit's tree is the
//! model-read state for step 1 (§2.10). Step ≥2 takes no pre-call
//! commit; the branch tip already represents what the model reads.
//! The `commit` field on each step's `meta.json` records that tip
//! sha so replay can re-run context assembly against the right
//! tree (§2.10) without consulting `request.json`.
//!
//! `request.json`, `response.json`, and `meta.json` land outside
//! the worktree and are not git-tracked (§2.3 — "Step records are
//! not committed to git").

use crate::prompt::Deps;
use crate::prompt::Error;
use crate::prompt::step::{META_FILE, REQUEST_FILE, StepMeta};
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

/// `git worktree add -b agents/<id> <worktree_path> <config-ref>`, run
/// against the workspace's bare `repo.git` (§2.2): fork the fresh root
/// agent off the default config branch's head — the fork is the freeze
/// (§2.2). Root id uniqueness per workspace is structural: the `-b`
/// creation fails if the ref already exists.
pub(super) fn spawn_branch(
    workspace: &Path,
    worktree_path: &Path,
    agent_id: &str,
    deps: &Deps<'_>,
) -> Result<(), Error> {
    let wt_str = worktree_path.to_string_lossy().to_string();
    let branch_ref = crate::workspace::agent_ref(agent_id);
    deps.git
        .run(
            &crate::workspace::repo_git(workspace),
            &[
                "worktree",
                "add",
                "-b",
                branch_ref.as_str(),
                wt_str.as_str(),
                crate::workspace::DEFAULT_CONFIG_REF,
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
pub(super) fn prepend_goal(goal: &str, soul: &str) -> String {
    format!("<goal>\n{goal}\n</goal>\n\n{soul}")
}

/// Step 1: write `goal.md` + `soul.md` to the worktree root. Step
/// ≥2 has no dispatch artifact (the branch tip already reflects the
/// model-read state per §2.10).
pub(super) fn write_dispatch_files(
    worktree_path: &Path,
    goal_text: &str,
    soul_text: &str,
) -> Result<(), Error> {
    std::fs::create_dir_all(worktree_path)?;
    std::fs::write(worktree_path.join(GOAL_FILE), goal_text)?;
    std::fs::write(worktree_path.join(SOUL_FILE), soul_text)?;
    Ok(())
}

/// Step 1's dispatch commit (§2.3 step 2): remove the harness-facing
/// control files from the agent's tree (§2.2 — control is read from the
/// governing config commit; the worktree holds only context), `git add
/// goal.md soul.md`, then commit on the agent branch. The removal is
/// total, not conditional: `--ignore-unmatch` makes it a no-op when the
/// fork point was not a config commit (a child forked off a parent's
/// tip, whose tree already lost them). This is the only commit the
/// harness emits for a step; §2.10 keeps step ≥2 commit-free, so the
/// branch tip after a dispatch commit *is* step 1's read state.
pub(super) fn commit_dispatch(
    worktree_path: &Path,
    conv_id: &str,
    deps: &Deps<'_>,
) -> Result<(), Error> {
    remove_control_files(worktree_path, deps.git)?;
    deps.git
        .run(worktree_path, &["add", GOAL_FILE, SOUL_FILE])
        .map_err(|source| Error::Git { op: "add", source })?;
    let msg = format!("step 001: dispatch [{conv_id}]");
    deps.git
        .run(worktree_path, &["commit", "-m", msg.as_str()])
        .map_err(|source| Error::Git {
            op: "commit",
            source,
        })
}

/// Stage the removal of the config commit's control files from the
/// agent's tree (§2.2): `git rm -r -q --ignore-unmatch -- <paths>`.
/// `descriptions/**` is deliberately not among them — it *is* context
/// (§3.3) and stays inherited.
pub(crate) fn remove_control_files(
    worktree_path: &Path,
    git: &dyn crate::template::GitRunner,
) -> Result<(), Error> {
    let mut args: Vec<&str> = vec!["rm", "-r", "-q", "--ignore-unmatch", "--"];
    args.extend_from_slice(crate::workspace::CONTROL_PATHS);
    git.run(worktree_path, &args).map_err(|source| Error::Git {
        op: "rm control files",
        source,
    })
}

/// Resolve the branch tip's sha at step-start. Recorded in
/// `meta.json` so replay can re-run context assembly against the
/// right tree without reading `request.json` (§2.10 Diagnostic-only
/// contract).
pub(super) fn read_branch_tip(worktree_path: &Path, deps: &Deps<'_>) -> Result<String, Error> {
    deps.git
        .run_capture(worktree_path, &["rev-parse", "HEAD"])
        .map_err(|source| Error::Git {
            op: "rev-parse",
            source,
        })
}

/// Land `request.json` under `<conv-repo>/steps/<conv-id>/<NNN>/`.
/// Outside every worktree (§2.2) so context assembly cannot pick it
/// up; not git-tracked (§2.3).
pub(super) fn write_request(
    conv_repo: &Path,
    step_dir_rel_str: &str,
    request_value: &Value,
) -> Result<(), Error> {
    let step_dir_abs = conv_repo.join(step_dir_rel_str);
    std::fs::create_dir_all(&step_dir_abs)?;
    let bytes = serde_json::to_vec_pretty(request_value).expect("Value is always serializable");
    std::fs::write(step_dir_abs.join(REQUEST_FILE), bytes)?;
    Ok(())
}

/// Land `meta.json` under the conv-repo step dir. The `commit` field
/// is the load-bearing piece (§2.10 — replay reproduces the wire
/// input by re-running context assembly against this sha).
pub(super) fn write_meta(
    conv_repo: &Path,
    step_dir_rel_str: &str,
    meta: &StepMeta,
) -> Result<(), Error> {
    let step_dir_abs = conv_repo.join(step_dir_rel_str);
    std::fs::create_dir_all(&step_dir_abs)?;
    let bytes = serde_json::to_vec_pretty(meta).expect("StepMeta is always serializable");
    std::fs::write(step_dir_abs.join(META_FILE), bytes)?;
    Ok(())
}
