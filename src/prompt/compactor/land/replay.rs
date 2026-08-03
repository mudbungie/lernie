//! The **replay** half of the compaction landing (ARCH §2.6): every
//! commit after the compaction point rebases onto the freshly minted
//! base, and the branch moves to the replayed tip.
//!
//! `git rebase --empty=keep --onto <base> <P> <branch>` is the whole
//! move: it re-lands each commit in `P..tip` in order (keeping ones the
//! squash made empty — a delete/delete agreement is still a commit the
//! checkpoint clock counts), then points the branch at the result.
//! Transcript entries are one immutable file each with monotonic names
//! (§2.3), so the replay is conflict-free by construction; where git
//! stops anyway, the index stages say which of the two legal exception
//! classes this is (the same stage-reading discipline as the retired
//! merge's decline, bl-a9eb):
//!
//! - **stages 1+3 only** — a modify/delete: the replayed commit rewrote a
//!   work product the compaction deleted. Git leaves the live content in
//!   the worktree; staging it (`git add`) resolves **live-branch-wins**,
//!   dropping the compaction's deletion. Lost compaction, never lost
//!   work.
//! - **anything else** — stage 2 present (both sides carry content — git
//!   wrote `<<<<<<<` markers) or a shape the construction does not admit:
//!   the landing is **declined loudly**. `git rebase --abort` restores
//!   the branch bit-for-bit, `refs/lernie/conflicted/<compactor-id>` is
//!   marked at the compactor's tip, and nothing lands (§2.6 decline —
//!   the same escape hatch as the work-product transfer).

use super::super::Error;
use super::LandOutcome;
use crate::template::GitRunner;
use crate::workspace::CONFLICTED_REF_PREFIX;
use std::collections::BTreeMap;
use std::path::Path;

/// Rebase the branch's commits after `point` onto `base` and move
/// `parent_id`'s branch to the replayed tip (module docs). The loop is
/// bounded by the number of commits being replayed: each continue settles
/// at least one, so more stops than commits means git is not making
/// progress and the landing aborts rather than spins.
pub(super) fn run(
    parent_worktree: &Path,
    parent_id: &str,
    compactor_id: &str,
    compactor_ref: &str,
    point: &str,
    base: &str,
    git: &dyn GitRunner,
) -> Result<LandOutcome, Error> {
    let branch = crate::workspace::agent_ref(parent_id);
    let range = format!("{point}..HEAD");
    let stops = git
        .run_capture(parent_worktree, &["rev-list", "--count", &range])
        .map_err(|source| Error::Git {
            op: "compaction land replay count",
            source,
        })?
        .trim()
        .parse::<u32>()
        .unwrap_or(0);

    let mut result = git.run(
        parent_worktree,
        &["rebase", "--empty=keep", "--onto", base, point, &branch],
    );
    let mut budget = stops;
    while let Err(source) = result {
        let unmerged = unmerged_stages(parent_worktree, git)?;
        let keep: Vec<&String> = unmerged
            .iter()
            .filter(|(_, s)| **s == (true, false, true))
            .map(|(path, _)| path)
            .collect();
        let marked: Vec<String> = unmerged
            .keys()
            .filter(|path| !keep.contains(path))
            .map(String::clone)
            .collect();
        if unmerged.is_empty() || budget == 0 {
            // Not a conflict stop (a dirty tree, a bad ref) — or git is
            // not making progress. Restore the branch and surface the
            // rebase's own failure.
            let _ = git.run(parent_worktree, &["rebase", "--abort"]);
            return Err(Error::Git {
                op: "compaction land rebase",
                source,
            });
        }
        if !marked.is_empty() {
            return decline(parent_worktree, compactor_id, compactor_ref, marked, git);
        }
        budget -= 1;
        let mut add = vec!["add", "--"];
        add.extend(keep.iter().map(|s| s.as_str()));
        git.run(parent_worktree, &add)
            .map_err(|source| Error::Git {
                op: "compaction land live-branch-wins add",
                source,
            })?;
        result = git.run(
            parent_worktree,
            &["-c", "core.editor=true", "rebase", "--continue"],
        );
    }
    Ok(LandOutcome::Landed)
}

/// Unmerged paths and which index stages each populates — `(base, ours,
/// theirs)`, i.e. stages 1/2/3 of `git ls-files -u`. In a rebase stop,
/// "ours" is the base side being rebased onto and "theirs" is the live
/// commit being replayed.
fn unmerged_stages(
    parent_worktree: &Path,
    git: &dyn GitRunner,
) -> Result<BTreeMap<String, (bool, bool, bool)>, Error> {
    let out = git
        .run_capture(parent_worktree, &["ls-files", "-u"])
        .map_err(|source| Error::Git {
            op: "compaction land unmerged",
            source,
        })?;
    let mut stages: BTreeMap<String, (bool, bool, bool)> = BTreeMap::new();
    for line in out.lines() {
        // `<mode> <sha> <stage>\t<path>`
        let Some((meta, path)) = line.split_once('\t') else {
            continue;
        };
        let entry = stages.entry(path.to_string()).or_default();
        match meta.rsplit(' ').next() {
            Some("1") => entry.0 = true,
            Some("2") => entry.1 = true,
            Some("3") => entry.2 = true,
            _ => {}
        }
    }
    Ok(stages)
}

/// Refuse the landing loudly (§2.6 decline): abort the rebase so the
/// branch and its worktree are exactly as they were, mark
/// `refs/lernie/conflicted/<compactor-id>` at the compactor's tip —
/// every byte of its work preserved for the operator — and land nothing.
/// The branch continues uncompacted, the §2.7 outcome for any compaction
/// that does not land.
fn decline(
    parent_worktree: &Path,
    compactor_id: &str,
    compactor_ref: &str,
    paths: Vec<String>,
    git: &dyn GitRunner,
) -> Result<LandOutcome, Error> {
    git.run(parent_worktree, &["rebase", "--abort"])
        .map_err(|source| Error::Git {
            op: "compaction land abort",
            source,
        })?;
    let conflicted_ref = format!("{CONFLICTED_REF_PREFIX}{compactor_id}");
    git.run(
        parent_worktree,
        &["update-ref", conflicted_ref.as_str(), compactor_ref],
    )
    .map_err(|source| Error::Git {
        op: "compaction land decline update-ref",
        source,
    })?;
    Ok(LandOutcome::Conflicted(paths))
}
