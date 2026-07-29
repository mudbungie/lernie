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
//!
//! **Anything else is refused, loudly.** `git add -A` is a *resolution*,
//! and it is only a correct one for the single overlap class above. When
//! git cannot merge a path on its own it writes `<<<<<<<` / `=======` /
//! `>>>>>>>` into the working tree, and an unqualified `add -A` would
//! stage that markup and commit it — into `summary/**`, which §5.2
//! composes into every subsequent model call on this branch. That is not
//! lost compaction, it is *corrupted context*, the one outcome §2.7
//! promises can never happen. So the merge asks git which paths it had to
//! mark ([`content_conflicts`], read from the index stages, not guessed)
//! and, for any of them, aborts the merge and marks
//! `refs/lernie/conflicted/<compactor-id>` instead — the §2.6 decline,
//! the same escape hatch the work-product transfer uses. Nothing lands;
//! the branch continues uncompacted. This is the third defect of
//! bl-a9eb (yog bl-ebbd), where three unresolved conflicts were committed
//! into a live `summary/001.md` at a root branch's tip.
//!
//! **Filtered to the compaction product** (§2.6, §2.7). A compactor is an
//! ordinary child, so its branch also grew its *own* context since `C`:
//! the `goal.md` and `soul.md` its dispatch commit rewrote, and the
//! transcript entries under `messages/**` its step loop appended. None of
//! that is the dispatching branch's context — it is the compactor's
//! private dialog, whose record is its own ref. What the merge is
//! specified to land is exactly what the two-tool toolset produces
//! (§2.7): the new `summary/<NNN>.md` and the nominated deletions. So the
//! staged merge is filtered before it commits — every path the merge
//! would *add or rewrite* outside `summary/` is restored to the
//! dispatching branch's own version, while deletions (the whole point of
//! compaction) pass untouched. This is the §2.6 work-product transfer's
//! principle in mirror image: that channel admits work products and
//! excludes branch-scoped context; this one admits the branch's own
//! context product and excludes the compactor's private dialog.

mod decline;

use super::Error;
use crate::template::GitRunner;
use crate::workspace;
use decline::{content_conflicts, decline};
use std::path::Path;

/// The compaction product's sole *addition* surface (ARCH §2.7): the
/// summary `write_summary` writes. A git exclude pathspec — the same
/// filtering mechanism the work-product transfer uses (§2.6).
const SUMMARY_EXCLUDE: &str = ":(exclude)summary";

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
    /// Git could not merge a path on its own and wrote conflict markers
    /// into it ([`content_conflicts`]) — the write-path guarantee this
    /// merge is built on was violated, so the merge is **aborted**,
    /// `refs/lernie/conflicted/<compactor-id>` is marked, and nothing
    /// lands. Carries the offending paths for the operator-facing line.
    Conflicted(Vec<String>),
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
/// A merge git had to write conflict markers into is
/// [`MergeOutcome::Conflicted`]: aborted, marked, nothing committed
/// ([`decline`]).
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
        &[
            "merge",
            "--no-ff",
            "--no-commit",
            "--no-edit",
            &compactor_ref,
        ],
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

    // Refuse before staging (module docs): `git add -A` is a resolution,
    // and it can only be *applied* to the one conflict class this merge
    // is specified to resolve. A path git had to write markers into is
    // not that class — staging it would commit the markers into the live
    // branch's own context.
    let conflicted = content_conflicts(parent_worktree, git)?;
    if !conflicted.is_empty() {
        return decline(
            parent_worktree,
            compactor_id,
            &compactor_ref,
            conflicted,
            git,
        );
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
    filter_to_product(parent_worktree, git)?;
    git.run(parent_worktree, &["commit", "--no-edit", "-m", &subject])
        .map_err(|source| Error::Git {
            op: "compaction merge commit",
            source,
        })?;
    Ok(MergeOutcome::Merged)
}

/// Drop the compactor's private dialog from the staged merge (module
/// docs, §2.6/§2.7). The comparison is the staged merge result against
/// `HEAD` — still the dispatching branch's own tip under `--no-commit` —
/// so its three classes name themselves: a path **added** outside
/// `summary/` is a compactor transcript entry, a path **rewritten**
/// outside `summary/` is its `goal.md`/`soul.md` (or a filename collision
/// git resolved into a conflict), and a path **deleted** is the
/// compaction the merge exists to land. Both non-deletion classes are
/// restored to the dispatching branch's version — an addition by removal,
/// a rewrite by checkout — leaving only the summary and the deletions.
fn filter_to_product(parent_worktree: &Path, git: &dyn GitRunner) -> Result<(), Error> {
    let added = staged_outside_summary(parent_worktree, "A", git)?;
    run_on_paths(parent_worktree, &["rm", "-q", "-f", "--"], &added, git)?;
    let rewritten = staged_outside_summary(parent_worktree, "M", git)?;
    run_on_paths(
        parent_worktree,
        &["checkout", "HEAD", "--"],
        &rewritten,
        git,
    )
}

/// Paths the staged merge would land outside `summary/` under diff class
/// `filter` (`A` added, `M` rewritten), relative to the dispatching
/// branch's tip. `--no-renames` keeps the classes exhaustive: an
/// add/delete pair must not collapse into an `R` that escapes both.
fn staged_outside_summary(
    parent_worktree: &Path,
    filter: &str,
    git: &dyn GitRunner,
) -> Result<Vec<String>, Error> {
    let filter_arg = format!("--diff-filter={filter}");
    let out = git
        .run_capture(
            parent_worktree,
            &[
                "diff",
                "--cached",
                "--name-only",
                "--no-renames",
                &filter_arg,
                "HEAD",
                "--",
                SUMMARY_EXCLUDE,
            ],
        )
        .map_err(|source| Error::Git {
            op: "compaction merge filter",
            source,
        })?;
    Ok(out
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty())
        .map(str::to_owned)
        .collect())
}

/// Run `args` extended with `paths`, or nothing at all when `paths` is
/// empty — the general path with empty inputs (`docs/PRINCIPLES.md`), not
/// a special case: git would read a pathspec-less `rm` as an error.
fn run_on_paths(
    parent_worktree: &Path,
    args: &[&str],
    paths: &[String],
    git: &dyn GitRunner,
) -> Result<(), Error> {
    if paths.is_empty() {
        return Ok(());
    }
    let mut argv: Vec<&str> = args.to_vec();
    argv.extend(paths.iter().map(String::as_str));
    git.run(parent_worktree, &argv)
        .map_err(|source| Error::Git {
            op: "compaction merge filter",
            source,
        })
}

/// True iff a merge is in progress in `parent_worktree` — i.e. `MERGE_HEAD`
/// resolves. Derived from git state, never a stored flag: `git rev-parse
/// --verify -q MERGE_HEAD` prints the sha and exits zero when a merge is
/// underway, and exits non-zero (captured as `Err`) when none is.
fn merge_in_progress(parent_worktree: &Path, git: &dyn GitRunner) -> Result<bool, Error> {
    match git.run_capture(
        parent_worktree,
        &["rev-parse", "--verify", "-q", "MERGE_HEAD"],
    ) {
        Ok(out) => Ok(!out.trim().is_empty()),
        Err(_) => Ok(false),
    }
}

#[cfg(test)]
mod tests;
