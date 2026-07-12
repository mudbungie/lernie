//! Rebase-then-align-then-no-ff merge protocol (ARCH §2.6).
//!
//! One shared routine for every parent/child merge-back in v0.3. The
//! compactor merges back into its dispatching branch through it; the
//! dispatching root conversation then merges into `main` through it.
//! A single path mirrors "One obvious path" (`docs/PRINCIPLES.md`):
//! both merges share the same core operation — rebase child onto
//! parent tip, align the merge=ours-pinned paths to the parent's
//! versions, `--no-ff` merge child into parent, remove the child's
//! worktree — and differ only in which paths name parent and child.
//!
//! §2.6 step 6 says a rebase that conflicts "indicates a harness
//! defect — two branches were given overlapping write paths". v0.3
//! does not have the single-author-per-file machinery in place yet,
//! so in practice the conflict path is reached when concurrent root
//! conversations overlap on e.g. `goal.md` — which v0.3 does not
//! test. We surface conflicts as [`Error::Git`] with `op: "rebase"`,
//! abort the rebase so the worktree is left in a clean state, and
//! write a marker ref `refs/lernie/conflicted/<child_branch>` at the
//! child's pre-rebase tip — the declined-transfer mark (§2.6), read
//! back when the subagent's result is surfaced. Single source of
//! truth: the marker is a plain git ref, not a sidecar file.
//!
//! **The alignment step.** Vanilla `merge=ours` only resolves
//! both-modified conflicts; it is silent on "added on theirs only"
//! and "modified on theirs / unchanged on ours". ARCH §2.6 wants the
//! parent's pre-merge state to win for `goal.md`, `soul.md`, and
//! `summary/**` regardless of which side touched them, so the
//! harness enforces that itself: right after the rebase, on the
//! child's tip, every merge=ours-pinned path is replaced with the
//! parent's version (or removed, if the parent does not have it),
//! and any resulting delta is committed as a single alignment
//! commit. The subsequent `merge --no-ff` therefore has nothing to
//! contribute on those paths — the parent's tree carries through
//! verbatim. The `.gitattributes` driver remains as a backstop for
//! hand-run merges that bypass the harness.

use super::Error;
use crate::template::{GitRunner, MERGE_OURS_PATHS};
use std::path::Path;

/// Ref-namespace prefix for the merge protocol's conflicted marker
/// (ARCH §2.6 step 6) — the declined-transfer mark. Single source of
/// the spec's ref name; a plain git ref, not a sidecar file.
const CONFLICTED_REF_PREFIX: &str = "refs/lernie/conflicted/";

/// Rebase `child_branch` onto `parent_branch`'s tip (run inside
/// `child_worktree`), then merge `child_branch` into `parent_branch`
/// with `--no-ff` (run inside `parent_worktree`), then remove
/// `child_worktree` (run inside `repo`). The branch ref survives for
/// the retention window (§2.3); only the worktree checkout is cleaned
/// up here.
///
/// Rebase is unconditional — v0.3 does not try to detect when the
/// parent tip has not advanced. The one-path discipline costs a
/// no-op rebase in the common case and is what makes concurrent root
/// conversations (v0.5 UI scope) tractable without retrofitting state
/// tracking now.
pub fn rebase_and_merge(
    repo: &Path,
    parent_branch: &str,
    parent_worktree: &Path,
    child_worktree: &Path,
    child_branch: &str,
    git: &dyn GitRunner,
) -> Result<(), Error> {
    if let Err(source) = git.run(child_worktree, &["rebase", parent_branch]) {
        // Clean up mid-rebase state so the operator can retry without
        // hand-running `git rebase --abort`. We intentionally ignore
        // the abort's error — if it fails there is nothing more the
        // harness can do, and the rebase failure is the one the
        // operator needs to see. After abort, the child branch ref is
        // back at its pre-rebase tip; mark that tip with a
        // conflicted ref (the declined-transfer mark, §2.6) so the
        // subagent's result can be surfaced without sidecar state.
        let _ = git.run(child_worktree, &["rebase", "--abort"]);
        let conflicted_ref = format!("{CONFLICTED_REF_PREFIX}{child_branch}");
        let _ = git.run(
            child_worktree,
            &["update-ref", conflicted_ref.as_str(), child_branch],
        );
        return Err(Error::Git {
            op: "rebase",
            source,
        });
    }

    align_merge_ours(parent_branch, child_worktree, git)?;

    git.run(parent_worktree, &["merge", "--no-ff", child_branch])
        .map_err(|source| Error::Git {
            op: "merge",
            source,
        })?;

    let wt_str = child_worktree.to_string_lossy().to_string();
    git.run(repo, &["worktree", "remove", wt_str.as_str()])
        .map_err(|source| Error::Git {
            op: "worktree remove",
            source,
        })
}

/// Align the merge=ours-pinned paths on the child's tip with the
/// parent's tip (ARCH §2.6). Three steps:
///
/// 1. `git rm -r --ignore-unmatch -- <pinned paths>` clears whatever
///    the child has at those paths. `--ignore-unmatch` means absent
///    paths are not an error — the cheap blanket call covers every
///    pinned path uniformly.
/// 2. `git ls-tree -r --name-only <parent_branch> -- <pinned paths>`
///    reports which of those paths the parent has. The harness then
///    `git checkout <parent_branch> --` only that subset, because
///    `git checkout` errors out if any pathspec misses.
/// 3. If steps 1 and 2 produced any index delta against `HEAD`, the
///    delta is committed as a single "merge=ours alignment" commit.
///    A no-op alignment (nothing was different) is silently skipped
///    to keep history quiet.
fn align_merge_ours(
    parent_branch: &str,
    child_worktree: &Path,
    git: &dyn GitRunner,
) -> Result<(), Error> {
    let mut rm_args: Vec<&str> = vec!["rm", "-r", "--ignore-unmatch", "--"];
    rm_args.extend(MERGE_OURS_PATHS.iter().copied());
    git.run(child_worktree, &rm_args)
        .map_err(|source| Error::Git {
            op: "merge=ours rm",
            source,
        })?;

    let mut ls_args: Vec<&str> = vec!["ls-tree", "-r", "--name-only", parent_branch, "--"];
    ls_args.extend(MERGE_OURS_PATHS.iter().copied());
    let listing = git
        .run_capture(child_worktree, &ls_args)
        .map_err(|source| Error::Git {
            op: "merge=ours ls-tree",
            source,
        })?;
    let on_parent: Vec<&str> = MERGE_OURS_PATHS
        .iter()
        .copied()
        .filter(|path| {
            listing
                .lines()
                .any(|line| line == *path || line.starts_with(&format!("{path}/")))
        })
        .collect();
    if !on_parent.is_empty() {
        let mut co_args: Vec<&str> = vec!["checkout", parent_branch, "--"];
        co_args.extend(on_parent.iter().copied());
        git.run(child_worktree, &co_args)
            .map_err(|source| Error::Git {
                op: "merge=ours checkout",
                source,
            })?;
    }

    let staged = git
        .run_capture(child_worktree, &["diff", "--cached", "--name-only"])
        .map_err(|source| Error::Git {
            op: "merge=ours diff",
            source,
        })?;
    if !staged.is_empty() {
        let msg = format!("merge=ours alignment with {parent_branch}");
        git.run(child_worktree, &["commit", "-m", msg.as_str()])
            .map_err(|source| Error::Git {
                op: "merge=ours commit",
                source,
            })?;
    }
    Ok(())
}

#[cfg(test)]
mod tests;
