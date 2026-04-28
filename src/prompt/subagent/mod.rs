//! Shared subagent-dispatch primitive (ARCH §2.3 step 2, §2.5).
//!
//! Every dispatched subagent — compactor (§2.7), worker (§2.5) — starts
//! with the same on-disk shape: a branch off the parent's tip, a sibling
//! worktree at `<conv-repo>/<full-descent>/` (§2.2), `goal.md` (and, for
//! roles with a per-call soul, `soul.md`) at the worktree root, all
//! committed as the dispatch commit. ARCH §2.5 calls dispatch "the
//! primitive"; this module is its in-process realization, shared between
//! the role-specific entry points.
//!
//! The function is `pub(crate)` because the only legitimate callers are
//! sibling modules within `prompt::` — the CLI surface for
//! procedure-to-procedure invocation is `lernie dispatch <role>` per
//! §3.4, never a direct library call.

use super::Error;
use crate::template::GitRunner;
use std::path::Path;

/// Worktree-relative goal artifact (ARCH §2.8). At the worktree root
/// so manifest pinning (§5.2) sees it.
pub(crate) const GOAL_FILE: &str = "goal.md";
/// Worktree-relative soul artifact (ARCH §2.3 step 2 / §4.3). At the
/// worktree root for the same reason `goal.md` is.
pub(crate) const SOUL_FILE: &str = "soul.md";

/// Inputs to a subagent dispatch's spawn step. Held as a struct so the
/// compactor and worker call sites pass identically-shaped requests.
pub(crate) struct SpawnRequest<'a> {
    /// Parent worktree — the dispatching branch's working tree. Owns
    /// the `.git` dir's view of `parent_branch`'s tip; this is where
    /// `git worktree add` runs (ARCH §2.2 — the conv-repo root itself
    /// is not a checkout in v0.3).
    pub(crate) parent_worktree: &'a Path,
    /// Dispatching branch name (full hyphenated descent of the
    /// parent's conv-id chain — ARCH §2.3).
    pub(crate) parent_branch: &'a str,
    /// New subagent branch name. By convention `<parent>-<sub-id>`,
    /// where `<sub-id>` is `<ts>-<short-id>` (§2.2 hyphenated descent).
    pub(crate) sub_branch: &'a str,
    /// Sibling worktree where the new branch is checked out. Same
    /// name as `sub_branch` so on-disk layout and ref namespace are
    /// isomorphic (§2.2).
    pub(crate) sub_worktree: &'a Path,
    /// Goal text written to `<sub_worktree>/goal.md` and committed.
    pub(crate) goal_text: &'a str,
    /// Soul text written to `<sub_worktree>/soul.md` when supplied.
    /// `None` for roles whose dispatch has no per-call soul (e.g. the
    /// v0.3 compactor stub: no model call, so no soul to compose).
    pub(crate) soul_text: Option<&'a str>,
    /// Commit message subject for the dispatch commit. Each role
    /// keeps its own phrasing so `git log --oneline` legibly
    /// distinguishes the role at a glance — compactor uses
    /// `compaction: dispatch [...]`; worker uses `dispatch: worker [...]`.
    pub(crate) commit_subject: &'a str,
}

/// Spawn the subagent branch and write the dispatch commit. Steps,
/// in order:
///
/// 1. `git worktree add -b <sub_branch> <sub_worktree> <parent_branch>`
///    in the parent worktree (which shares `.git` with the conv-repo
///    root, §2.2).
/// 2. Write `goal.md` to the new worktree.
/// 3. If a soul was supplied, write `soul.md` to the new worktree.
/// 4. `git add` the artifacts.
/// 5. `git commit -m <commit_subject>` — the dispatch commit (§2.3
///    step 2 / §2.10). Step 1 of the subagent's own step loop, when
///    one runs, takes no further pre-call commit; the dispatch commit
///    *is* its read state.
///
/// Disk and git ordering match the v0.3 compactor stub so callers that
/// audit the run sequence (e.g. `compactor::tests`) keep working
/// verbatim through the refactor.
pub(crate) fn spawn_subagent_branch(
    req: &SpawnRequest<'_>,
    git: &dyn GitRunner,
) -> Result<(), Error> {
    let wt_str = req.sub_worktree.to_string_lossy().to_string();
    git.run(
        req.parent_worktree,
        &[
            "worktree",
            "add",
            "-b",
            req.sub_branch,
            wt_str.as_str(),
            req.parent_branch,
        ],
    )
    .map_err(|source| Error::Git {
        op: "worktree add",
        source,
    })?;

    // `git worktree add` creates the directory in production; the
    // explicit `create_dir_all` is here for stub-git tests (and is a
    // harmless no-op in production since the directory already exists).
    std::fs::create_dir_all(req.sub_worktree)?;
    std::fs::write(req.sub_worktree.join(GOAL_FILE), req.goal_text)?;
    if let Some(soul) = req.soul_text {
        std::fs::write(req.sub_worktree.join(SOUL_FILE), soul)?;
    }

    let mut add_args: Vec<&str> = vec!["add", GOAL_FILE];
    if req.soul_text.is_some() {
        add_args.push(SOUL_FILE);
    }
    git.run(req.sub_worktree, &add_args)
        .map_err(|source| Error::Git { op: "add", source })?;

    git.run(req.sub_worktree, &["commit", "-m", req.commit_subject])
        .map_err(|source| Error::Git {
            op: "commit",
            source,
        })?;

    Ok(())
}

#[cfg(test)]
mod tests;
