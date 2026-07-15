//! Compactor toolset (ARCH §2.7) — `write_summary` and
//! `mark_for_deletion`, and nothing else.
//!
//! These are the **two** tools a compactor agent may call, and they are
//! **built into the primitive, not declared in `providers.yaml`** (§2.7):
//! the compactor's toolset is this fixed pair, injected by the harness for
//! the compactor role alone, never assembled from a role's `tools:` list.
//! The narrowness is the point — giving the compactor no general
//! filesystem write surface makes "deletion-only" a **structural**
//! property rather than a disciplinary one: the worst failure mode is lost
//! information, never corrupted information (§2.7, §2.6 live-branch-wins).
//!
//! - [`write_summary`] writes `summary/<NNN>.md` on the compactor branch —
//!   the one location it may create, picked by scanning the directory.
//! - [`mark_for_deletion`] nominates a file for removal; the harness
//!   applies the deletion at commit time. "Applied at commit time" is
//!   realized by staging the removal (`git rm`) so the compactor step's
//!   own commit carries it (§2.3), and the compaction merge (§2.6) then
//!   lands it — subject to live-branch-wins on any work-product overlap.
//!
//! The deletions are **deletion-only structural**: `git rm` can remove but
//! never write content, so a compactor cannot corrupt a work product even
//! by defect. The compactor decides relevance against the dispatching
//! branch's goal (`goal.md`), which its inherited worktree carries (§2.7).

use super::Error;
use crate::template::GitRunner;
use std::path::Path;

/// Built-in tool name: write the next `summary/<NNN>.md` (ARCH §2.7).
pub(crate) const WRITE_SUMMARY: &str = "write_summary";
/// Built-in tool name: nominate a branch-relative path for removal
/// (deletion-only structural, ARCH §2.7).
pub(crate) const MARK_FOR_DELETION: &str = "mark_for_deletion";

/// Branch-relative directory holding compaction summaries (ARCH §2.7).
/// Lives at the worktree root so the manifest's role-keyed `pinned:
/// [summary/**]` rule (§5.2) sees it.
pub(crate) const SUMMARY_DIR: &str = "summary";
/// Width of the zero-padded summary-seq in summary filenames
/// (`001.md`, `002.md`). Matches the step-seq width (§2.3) so the two
/// on-disk layouts read uniformly.
const SUMMARY_SEQ_WIDTH: usize = 3;

/// Write `summary/<NNN>.md` on `worktree`, picking the next-available
/// seq by scanning the directory. Returns the branch-relative path of the
/// written file for the subsequent `git add`.
///
/// Seq is branch-global over the summary directory's contents: a branch
/// may compact several times (§2.7), and reading existing seqs here means
/// every checkpoint shares one numbering rule.
pub(crate) fn write_summary(worktree: &Path, content: &str) -> std::io::Result<String> {
    let dir_abs = worktree.join(SUMMARY_DIR);
    std::fs::create_dir_all(&dir_abs)?;
    let seq = next_seq(&dir_abs)?;
    let file_name = format!("{seq:0width$}.md", width = SUMMARY_SEQ_WIDTH);
    let path_abs = dir_abs.join(&file_name);
    std::fs::write(&path_abs, content)?;
    Ok(format!("{SUMMARY_DIR}/{file_name}"))
}

/// Nominate the branch-relative `path` for removal (ARCH §2.7). Realized
/// as `git rm -r -- <path>` inside the compactor `worktree`, staging the
/// deletion so the compactor step's commit carries it (§2.3) — the
/// "applied at commit time" contract. **Deletion-only structural**: this
/// can only remove, never write, so a compactor cannot corrupt content.
///
/// A path that does not exist on the branch is **declined loudly** rather
/// than silently ignored (`docs/PRINCIPLES.md` "Decline illegal
/// operations"): a compactor nominating a nonexistent file is a defect
/// worth surfacing, and `git rm` errors on it.
pub(crate) fn mark_for_deletion(
    worktree: &Path,
    path: &str,
    git: &dyn GitRunner,
) -> Result<(), Error> {
    git.run(worktree, &["rm", "-r", "-q", "--", path])
        .map_err(|source| Error::Git {
            op: "mark_for_deletion rm",
            source,
        })
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
    use crate::template::RealGit;

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

    /// A real repo on `agents/p1` with one tracked file, for the
    /// deletion-only `git rm` path.
    fn repo_with(rel: &str) -> tempfile::TempDir {
        let dir = tmpdir();
        let wt = dir.path();
        let g = RealGit::new();
        g.run(wt, &["init", "-b", "agents/p1"]).unwrap();
        g.run(wt, &["config", "user.email", "t@t"]).unwrap();
        g.run(wt, &["config", "user.name", "t"]).unwrap();
        let f = wt.join(rel);
        std::fs::create_dir_all(f.parent().unwrap()).unwrap();
        std::fs::write(&f, "content\n").unwrap();
        g.run(wt, &["add", "-A"]).unwrap();
        g.run(wt, &["commit", "-m", "c"]).unwrap();
        dir
    }

    #[test]
    fn mark_for_deletion_stages_a_real_removal() {
        let dir = repo_with("messages/001-user.md");
        let wt = dir.path();
        mark_for_deletion(wt, "messages/001-user.md", &RealGit::new()).unwrap();
        // Removed from the worktree and staged for the next commit.
        assert!(!wt.join("messages/001-user.md").exists());
        let staged = RealGit::new()
            .run_capture(wt, &["diff", "--cached", "--name-status"])
            .unwrap();
        assert!(staged.starts_with('D'), "staged deletion: {staged:?}");
    }

    #[test]
    fn mark_for_deletion_declines_a_nonexistent_path() {
        let dir = repo_with("keep.txt");
        let err = mark_for_deletion(dir.path(), "no/such.md", &RealGit::new()).unwrap_err();
        assert!(
            matches!(err, Error::Git { op: "mark_for_deletion rm", .. }),
            "{err:?}"
        );
    }
}
