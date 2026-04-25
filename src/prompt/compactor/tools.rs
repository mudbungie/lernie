//! Compactor tool surface (ARCH §2.7).
//!
//! These are the two tools a v1 compactor agent is allowed to call:
//! `write_summary` and `mark_for_deletion`. The toolset is
//! deliberately small so "deletion-only" is a structural property —
//! the compactor has no general filesystem write surface, which makes
//! the worst failure mode lost information rather than corrupted
//! information.
//!
//! v0.3 exposes them here so the call sites in the stub (`super::run`)
//! land with the final shape. The `mark_for_deletion` semantics are
//! a no-op in v0.3 per ARCH §12; v0.4+ wires the real `git rm` path.

use std::path::Path;

/// Branch-relative directory holding compaction summaries (ARCH §2.7).
/// Lives at the worktree root so the manifest's role-keyed `pinned:
/// [summary/**]` rule (§5.2) sees it.
pub(crate) const SUMMARY_DIR: &str = "summary";
/// Width of the zero-padded summary-seq in summary filenames
/// (`001.md`, `002.md`). Matches the step-seq width (§2.3) so the two
/// on-disk layouts read uniformly.
const SUMMARY_SEQ_WIDTH: usize = 3;

/// Write `summary/<NNN>.md` on `worktree`, picking the next-available
/// seq by scanning the directory. Returns the branch-relative path of
/// the written file for the subsequent `git add`.
///
/// Seq is branch-global over the summary directory's contents:
/// intermediate compaction (§2.7 / v0.6) will write multiple summaries
/// per branch, and reading existing seqs here means the stub and the
/// future intermediate case share one numbering rule.
pub(crate) fn write_summary(worktree: &Path, content: &str) -> std::io::Result<String> {
    let dir_abs = worktree.join(SUMMARY_DIR);
    std::fs::create_dir_all(&dir_abs)?;
    let seq = next_seq(&dir_abs)?;
    let file_name = format!("{seq:0width$}.md", width = SUMMARY_SEQ_WIDTH);
    let path_abs = dir_abs.join(&file_name);
    std::fs::write(&path_abs, content)?;
    Ok(format!("{SUMMARY_DIR}/{file_name}"))
}

/// Nominate a file on the compactor branch for removal at commit
/// time. v0.3 leaves this a no-op: the stub does not prune the raw
/// step tree, so the merge commit carries the full step dirs
/// alongside the compaction summary. v0.4+ wires this to `git rm`
/// with the deletion-only write discipline (§2.7).
#[allow(
    dead_code,
    reason = "surface is part of the v1 compactor contract (ARCH §2.7) — the v0.3 \
        stub exposes it so call sites land with the final shape but the deletion \
        semantics come in v0.4"
)]
pub fn mark_for_deletion(_worktree: &Path, _path: &Path) -> std::io::Result<()> {
    Ok(())
}

/// Pick the next summary-seq: one more than the highest existing
/// `<NNN>.md` file in the directory. Non-`.md` files and files whose
/// stems don't parse as integers are skipped so an operator-dropped
/// note never fouls numbering.
fn next_seq(dir: &Path) -> std::io::Result<u32> {
    let mut max = 0u32;
    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let name = entry.file_name();
        let Some(stem) = Path::new(&name).file_stem().and_then(|s| s.to_str()) else {
            continue;
        };
        if Path::new(&name).extension().and_then(|e| e.to_str()) != Some("md") {
            continue;
        }
        if let Ok(n) = stem.parse::<u32>() {
            max = max.max(n);
        }
    }
    Ok(max + 1)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tmpdir() -> tempfile::TempDir {
        tempfile::TempDir::new().unwrap()
    }

    #[test]
    fn write_summary_picks_001_when_dir_is_empty() {
        let wt = tmpdir();
        let rel = write_summary(wt.path(), "body\n").unwrap();
        assert_eq!(rel, "summary/001.md");
        assert_eq!(
            std::fs::read_to_string(wt.path().join(&rel)).unwrap(),
            "body\n"
        );
    }

    #[test]
    fn write_summary_increments_past_existing_files() {
        let wt = tmpdir();
        let dir = wt.path().join("summary");
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("001.md"), "old").unwrap();
        std::fs::write(dir.join("007.md"), "also old").unwrap();
        let rel = write_summary(wt.path(), "new\n").unwrap();
        assert_eq!(rel, "summary/008.md");
    }

    #[test]
    fn write_summary_skips_non_md_and_unparseable_stems() {
        let wt = tmpdir();
        let dir = wt.path().join("summary");
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("README.txt"), "").unwrap();
        std::fs::write(dir.join("notes.md"), "").unwrap();
        std::fs::write(dir.join("002.md"), "").unwrap();
        let rel = write_summary(wt.path(), "x").unwrap();
        assert_eq!(rel, "summary/003.md");
    }

    #[test]
    fn mark_for_deletion_is_a_noop() {
        // The stub exists to hold the compactor tool surface (ARCH
        // §2.7). A no-op is what v0.3 ships; asserting the Ok makes
        // future reimplementations visible as test changes.
        let wt = tmpdir();
        mark_for_deletion(wt.path(), Path::new("anything")).unwrap();
    }
}
