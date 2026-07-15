//! The compaction merge (ARCH §2.6, §2.7, §5.5) — the one merge left in
//! the system now that merge-back is gone (§2.6).
//!
//! A compactor forks off a **checkpoint commit** `C` (the dispatching
//! branch's tip at dispatch) and rewrites only what existed at `C`:
//! deleting superseded transcript entries, landing a new summary, and
//! nominating superseded work products for deletion. The live agent keeps
//! stepping past `C`, and its commits since `C` only *append* new
//! sequence filenames (transcript immutability, §2.3) — so the two write
//! sets are disjoint and the merge is conflict-free by construction. The
//! agent's own executor lands it `--no-ff` at a step boundary; the merge
//! commit *is* the context rebuild point (§5.5).
//!
//! **The one theoretical overlap — live-branch-wins.** A compactor may
//! nominate a *work product* the live agent has rewritten since `C`. That
//! is the sole conflict class: transcript entries never collide (the live
//! branch only appends new filenames) and a fresh `summary/<NNN>.md`
//! never collides (its seq is past every prior summary, §2.7). The
//! overlap surfaces as a git modify/delete conflict, and the executor
//! resolves it **live-branch-wins**: it stages the working-tree state
//! (git leaves the live agent's version in the worktree on a
//! modify/delete), which drops the compactor's deletion. A dropped
//! deletion is lost compaction, never lost work — the same worst case the
//! deletion-only toolset already guarantees (§2.7).
//!
//! `git add -A` after a `--no-ff --no-commit` merge realizes this in one
//! move: a clean merge is already fully staged (the add is a no-op), and
//! a modify/delete conflict leaves the live version in the worktree,
//! which the add stages — resolving the conflict by keeping ours.

use super::Error;
use crate::template::GitRunner;
use crate::workspace;
use std::path::Path;

/// Outcome of a compaction merge attempt against the dispatching branch.
#[derive(Debug, PartialEq, Eq)]
pub enum MergeOutcome {
    /// The compactor's branch landed as a `--no-ff` merge commit — the
    /// summary and every non-overlapping deletion applied, any
    /// work-product-deletion overlap resolved live-branch-wins (§2.6).
    Merged,
    /// The dispatching branch was already up to date with the compactor
    /// ref — nothing to land. The general path with an empty diff, not a
    /// bootstrap special case (`docs/PRINCIPLES.md`).
    NoOp,
}

/// Land the compactor branch `compactor_id` into the dispatching branch
/// checked out at `parent_worktree` (ARCH §2.6). The checkout's `HEAD`
/// *is* the dispatching branch (§2.3), so the merge base derives from
/// ancestry and no branch name is passed. `--no-ff --no-commit` sets the
/// merge up, `git add -A` resolves any work-product modify/delete overlap
/// live-branch-wins (module docs), and the commit lands the two-parent
/// merge — the §5.5 rebuild point.
///
/// A compactor whose ref is already an ancestor of `HEAD` (nothing to
/// land) leaves no `MERGE_HEAD`; that is [`MergeOutcome::NoOp`], not an
/// error. A merge that fails to even begin (a bad ref) surfaces loudly.
pub fn merge(
    parent_worktree: &Path,
    compactor_id: &str,
    git: &dyn GitRunner,
) -> Result<MergeOutcome, Error> {
    let compactor_ref = workspace::agent_ref(compactor_id);
    let subject = format!("compaction merge [{compactor_id}]");

    // `--no-ff --no-commit`: a conflicting merge exits non-zero but still
    // establishes MERGE_HEAD; an up-to-date merge exits zero and sets
    // none. So the exit code is not the signal — the presence of
    // MERGE_HEAD is (checked next).
    let merge_res = git.run(
        parent_worktree,
        &["merge", "--no-ff", "--no-commit", "--no-edit", &compactor_ref],
    );

    if !merge_in_progress(parent_worktree, git)? {
        // No MERGE_HEAD: either already up to date (the merge was a
        // no-op) or the merge could not begin (a bad ref). The first is
        // `Ok(NoOp)`; the second is the error `merge_res` carries.
        merge_res.map_err(|source| Error::Git {
            op: "compaction merge",
            source,
        })?;
        return Ok(MergeOutcome::NoOp);
    }

    // Live-branch-wins (§2.6): stage the working-tree state. A clean merge
    // is already staged (no-op); a work-product modify/delete overlap
    // leaves the live version in the worktree, which this stages — the
    // compactor's deletion of a rewritten work product is thereby dropped.
    git.run(parent_worktree, &["add", "-A"])
        .map_err(|source| Error::Git {
            op: "compaction merge add",
            source,
        })?;
    git.run(parent_worktree, &["commit", "--no-edit", "-m", &subject])
        .map_err(|source| Error::Git {
            op: "compaction merge commit",
            source,
        })?;
    Ok(MergeOutcome::Merged)
}

/// True iff a merge is in progress in `parent_worktree` — i.e. `MERGE_HEAD`
/// resolves. Derived from git state, never a stored flag: `git rev-parse
/// --verify -q MERGE_HEAD` prints the sha and exits zero when a merge is
/// underway, and exits non-zero (captured as `Err`) when none is.
fn merge_in_progress(parent_worktree: &Path, git: &dyn GitRunner) -> Result<bool, Error> {
    match git.run_capture(parent_worktree, &["rev-parse", "--verify", "-q", "MERGE_HEAD"]) {
        Ok(out) => Ok(!out.trim().is_empty()),
        Err(_) => Ok(false),
    }
}

#[cfg(test)]
mod tests;
