//! The §2.6 **decline** half of the compaction merge: recognising a
//! conflict git had to write markers for, and refusing the merge loudly
//! rather than committing the markup. See [`super`]'s module docs for why
//! `git add -A` is a resolution only for the one expected overlap class.

use super::{Error, MergeOutcome};
use crate::template::GitRunner;
use crate::workspace::CONFLICTED_REF_PREFIX;
use std::collections::BTreeMap;
use std::path::Path;

/// Paths git could not merge on its own and wrote conflict markers into.
///
/// `git ls-files -u` lists every unmerged path once per populated index
/// stage — 1 base, 2 ours, 3 theirs. The distinction the merge turns on
/// is exactly which stages are populated, so it is read here rather than
/// guessed: **stages 2 and 3 both present** means both sides carried
/// content for the path, which is the case — and the only case — where
/// git writes `<<<<<<<` / `=======` / `>>>>>>>` into the working tree.
/// The expected overlap class, a work-product modify/delete, populates
/// one of the two and leaves the live version marker-free in the
/// worktree, so it is not listed here and stays live-branch-wins.
pub(super) fn content_conflicts(
    parent_worktree: &Path,
    git: &dyn GitRunner,
) -> Result<Vec<String>, Error> {
    let out = git
        .run_capture(parent_worktree, &["ls-files", "-u"])
        .map_err(|source| Error::Git {
            op: "compaction merge unmerged",
            source,
        })?;
    let mut both: BTreeMap<String, (bool, bool)> = BTreeMap::new();
    for line in out.lines() {
        // `<mode> <sha> <stage>\t<path>`
        let Some((meta, path)) = line.split_once('\t') else {
            continue;
        };
        let entry = both.entry(path.to_string()).or_default();
        match meta.rsplit(' ').next() {
            Some("2") => entry.0 = true,
            Some("3") => entry.1 = true,
            _ => {}
        }
    }
    Ok(both
        .into_iter()
        .filter(|(_, (ours, theirs))| *ours && *theirs)
        .map(|(path, _)| path)
        .collect())
}

/// Refuse the compaction merge loudly (§2.6 decline): abort the merge so
/// the dispatching branch's tree is left exactly as it was, mark
/// `refs/lernie/conflicted/<compactor-id>` at the compactor's ref — every
/// byte of its work preserved for the operator — and land nothing. The
/// branch continues uncompacted, which is the §2.7 outcome for any
/// compaction that does not land; lost compaction is the worst case the
/// deletion-only toolset already guarantees, and it is strictly better
/// than a summary of conflict markers being composed into the next model
/// call as if it were context (§5.2).
pub(super) fn decline(
    parent_worktree: &Path,
    compactor_id: &str,
    compactor_ref: &str,
    paths: Vec<String>,
    git: &dyn GitRunner,
) -> Result<MergeOutcome, Error> {
    git.run(parent_worktree, &["merge", "--abort"])
        .map_err(|source| Error::Git {
            op: "compaction merge abort",
            source,
        })?;
    let conflicted_ref = format!("{CONFLICTED_REF_PREFIX}{compactor_id}");
    git.run(
        parent_worktree,
        &["update-ref", conflicted_ref.as_str(), compactor_ref],
    )
    .map_err(|source| Error::Git {
        op: "compaction merge decline update-ref",
        source,
    })?;
    Ok(MergeOutcome::Conflicted(paths))
}
