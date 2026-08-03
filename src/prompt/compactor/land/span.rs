//! The compaction span (ARCH §2.6): where the returning compactor's pass
//! begins and ends on the dispatching branch, derived entirely from git.
//!
//! - The **compaction point** `P` is the parent of the compactor's own
//!   dispatch commit — the commit it forked off (§2.6), the same
//!   ancestry-derived fork point the work-product transfer uses.
//! - The **span's lower bound** is the dispatching branch's checkpoint
//!   origin *as of `P`* ([`super::super::checkpoint::origin`]): its
//!   founding commit or its last compaction base, whichever is newer —
//!   the same single derivation the checkpoint clock reads, so the span
//!   and the clock cannot disagree. When neither exists (a degenerate
//!   tree), the root commit is the general-path fallback.
//! - The pass is **superseded** (`None`) when another compaction landed
//!   since `P`: the point is no longer reachable from `HEAD` (the landing
//!   rebased it away), or a compaction base — or a retired-mechanism
//!   merge commit — sits inside `P..HEAD`, which a replay must never
//!   re-replay.

use super::super::{Error, checkpoint};
use crate::prompt::role;
use crate::template::GitRunner;
use std::path::Path;

/// A current (un-superseded) compaction span, ready to land.
pub(super) struct Span {
    /// The compactor's dispatch commit on its own branch — the boundary
    /// past which its committed deletions are nominations, not the
    /// harness's fork-time prunes ([`super::base::product`]).
    pub(super) dispatch: String,
    /// The compaction point `P` — the commit the compactor forked off.
    pub(super) point: String,
    /// The span's lower bound — the parent of the base commit the landing
    /// mints; everything between it and `P` is squashed.
    pub(super) bound: String,
}

/// Derive the span for `compactor_id`'s return, or `None` when the pass
/// is superseded (module docs). A compactor branch with no dispatch
/// commit is declined loudly — it is not a compactor branch.
pub(super) fn of(
    parent_worktree: &Path,
    parent_id: &str,
    compactor_id: &str,
    compactor_ref: &str,
    git: &dyn GitRunner,
) -> Result<Option<Span>, Error> {
    let dispatch = role::founding_sha(parent_worktree, compactor_ref, compactor_id, git)?.ok_or(
        Error::Git {
            op: "compaction land dispatch commit",
            source: std::io::Error::other(format!(
                "no dispatch commit names [{compactor_id}] on {compactor_ref}"
            )),
        },
    )?;
    let parent_rev = format!("{dispatch}^");
    let point = git
        .run_capture(parent_worktree, &["rev-parse", &parent_rev])
        .map_err(|source| Error::Git {
            op: "compaction land point",
            source,
        })?
        .trim()
        .to_string();

    if !reaches(parent_worktree, &point, git) || landed_since(parent_worktree, &point, git)? {
        return Ok(None);
    }

    let bound = match checkpoint::origin(parent_worktree, &point, parent_id, git)? {
        Some(sha) => sha,
        None => checkpoint::root_of(parent_worktree, &point, git)?,
    };
    Ok(Some(Span {
        dispatch,
        point,
        bound,
    }))
}

/// Whether `point` is still an ancestor of `HEAD`. A landing since the
/// compactor forked rebases the point out of the branch's history, and
/// `merge-base --is-ancestor` answers by exit code — a non-zero exit
/// (`Err` under [`GitRunner::run`]) reads as "no", the safe direction:
/// a pass that cannot prove its point lands nothing.
fn reaches(parent_worktree: &Path, point: &str, git: &dyn GitRunner) -> bool {
    git.run(
        parent_worktree,
        &["merge-base", "--is-ancestor", point, "HEAD"],
    )
    .is_ok()
}

/// Whether a compaction landed inside `point..HEAD`: a compaction base
/// (or a retired-mechanism merge commit — any merge, since the one merge
/// the system ever landed was compaction) inside the replay span means
/// this pass was overtaken.
fn landed_since(parent_worktree: &Path, point: &str, git: &dyn GitRunner) -> Result<bool, Error> {
    let range = format!("{point}..HEAD");
    let err = |source| Error::Git {
        op: "compaction land span rev-list",
        source,
    };
    let landed = checkpoint::landing_subject_pattern();
    let bases = git
        .run_capture(
            parent_worktree,
            &["rev-list", "-E", "--grep", landed.as_str(), &range],
        )
        .map_err(err)?;
    if !bases.trim().is_empty() {
        return Ok(true);
    }
    let merges = git
        .run_capture(parent_worktree, &["rev-list", "--merges", &range])
        .map_err(err)?;
    Ok(!merges.trim().is_empty())
}
