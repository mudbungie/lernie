//! Tests for the compaction merge (ARCH §2.6).
//!
//! The behavioral arms run against a **real** git repo so the `--no-ff`
//! land and — the key pin — the live-branch-wins resolution of a
//! work-product-deletion overlap are exercised end-to-end. The two git-op
//! error arms (add / commit failure after a merge is set up) route through
//! a stub whose `run_capture` reports `MERGE_HEAD` present.

use super::*;
use crate::template::RealGit;
use std::cell::RefCell;
use std::path::PathBuf;
use tempfile::TempDir;

fn g() -> RealGit {
    RealGit::new()
}

/// A repo whose working tree is checked out on `agents/p1` with a single
/// checkpoint commit `C` carrying `files`. Returns the TempDir (the
/// worktree at its root).
fn repo_at_checkpoint(files: &[(&str, &str)]) -> TempDir {
    let dir = TempDir::new().unwrap();
    let wt = dir.path();
    let git = g();
    git.run(wt, &["init", "-b", "agents/p1"]).unwrap();
    git.run(wt, &["config", "user.email", "t@t"]).unwrap();
    git.run(wt, &["config", "user.name", "t"]).unwrap();
    for (rel, content) in files {
        write(wt, rel, content);
    }
    git.run(wt, &["add", "-A"]).unwrap();
    git.run(wt, &["commit", "-m", "checkpoint C"]).unwrap();
    dir
}

fn write(wt: &Path, rel: &str, content: &str) {
    let path = wt.join(rel);
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(path, content).unwrap();
}

/// Fork the compactor branch `agents/p1-cmp` off `C`, apply `summary` +
/// `deletions`, commit, and switch the worktree back to `agents/p1`.
fn compactor_branch(wt: &Path, summary: (&str, &str), deletions: &[&str]) {
    let git = g();
    git.run(wt, &["checkout", "-b", "agents/p1-cmp"]).unwrap();
    write(wt, summary.0, summary.1);
    for path in deletions {
        git.run(wt, &["rm", "--", path]).unwrap();
    }
    git.run(wt, &["add", "-A"]).unwrap();
    git.run(wt, &["commit", "-m", "compaction"]).unwrap();
    git.run(wt, &["checkout", "agents/p1"]).unwrap();
}

/// Advance the live `agents/p1` branch with the given file writes, then
/// commit — the appends/rewrites that happen while the compactor runs.
fn advance_live(wt: &Path, files: &[(&str, &str)]) {
    let git = g();
    for (rel, content) in files {
        write(wt, rel, content);
    }
    git.run(wt, &["add", "-A"]).unwrap();
    git.run(wt, &["commit", "-m", "live step"]).unwrap();
}

fn head_parents(wt: &Path) -> usize {
    g().run_capture(wt, &["rev-list", "--parents", "-n", "1", "HEAD"])
        .unwrap()
        .split_whitespace()
        .count()
        - 1
}

#[test]
fn merge_lands_summary_and_clean_transcript_deletion() {
    // Compactor deletes a transcript entry the live branch never touched
    // and adds a summary; the live branch appended a new entry. Disjoint
    // write sets — the merge is clean and both deletions/adds land.
    let dir = repo_at_checkpoint(&[("messages/001-user.md", "hi\n")]);
    let wt = dir.path();
    compactor_branch(wt, ("summary/001.md", "digest\n"), &["messages/001-user.md"]);
    advance_live(wt, &[("messages/002-a.md", "reply\n")]);

    assert_eq!(merge(wt, "p1-cmp", &g()).unwrap(), MergeOutcome::Merged);
    assert!(!wt.join("messages/001-user.md").exists(), "deletion landed");
    assert!(wt.join("messages/002-a.md").exists(), "live append kept");
    assert!(wt.join("summary/001.md").exists(), "summary landed");
    assert_eq!(head_parents(wt), 2, "--no-ff two-parent merge commit");
}

#[test]
fn overlap_drops_work_product_deletion_live_branch_wins() {
    // THE PIN (§2.6): the compactor nominates a work product the live
    // agent rewrote since C. The deletion is dropped — the live version
    // survives — while the summary still lands. Lost compaction, never
    // lost work.
    let dir = repo_at_checkpoint(&[("code.txt", "v1\n")]);
    let wt = dir.path();
    compactor_branch(wt, ("summary/001.md", "digest\n"), &["code.txt"]);
    advance_live(wt, &[("code.txt", "v2\n")]);

    assert_eq!(merge(wt, "p1-cmp", &g()).unwrap(), MergeOutcome::Merged);
    assert_eq!(
        std::fs::read_to_string(wt.join("code.txt")).unwrap(),
        "v2\n",
        "live-branch-wins: the rewritten work product survives"
    );
    assert!(wt.join("summary/001.md").exists(), "summary still landed");
    assert_eq!(head_parents(wt), 2);
}

#[test]
fn already_up_to_date_is_a_noop() {
    // No compactor commits past C: the ref is an ancestor of HEAD, the
    // merge sets no MERGE_HEAD, and nothing lands (empty-diff general path).
    let dir = repo_at_checkpoint(&[("goal.md", "g\n")]);
    let wt = dir.path();
    let git = g();
    git.run(wt, &["branch", "agents/p1-cmp"]).unwrap();
    assert_eq!(merge(wt, "p1-cmp", &git).unwrap(), MergeOutcome::NoOp);
    // HEAD is unchanged — still the single root checkpoint commit (no
    // parent), so no merge commit was created.
    assert_eq!(head_parents(wt), 0, "no merge commit was created");
}

#[test]
fn a_bad_compactor_ref_is_declined_loudly() {
    let dir = repo_at_checkpoint(&[("goal.md", "g\n")]);
    let err = merge(dir.path(), "does-not-exist", &g()).unwrap_err();
    assert!(matches!(err, Error::Git { op: "compaction merge", .. }), "{err:?}");
}

/// Stub git reporting a merge in progress (non-empty `MERGE_HEAD`) so the
/// add/commit arms after merge setup are reachable; `run` fails at a
/// chosen call index.
struct StubGit {
    calls: RefCell<Vec<Vec<String>>>,
    fail_at: usize,
}
impl StubGit {
    fn failing_at(idx: usize) -> Self {
        Self {
            calls: RefCell::new(Vec::new()),
            fail_at: idx,
        }
    }
}
impl GitRunner for StubGit {
    fn run(&self, _dest: &Path, args: &[&str]) -> std::io::Result<()> {
        let idx = self.calls.borrow().len();
        self.calls
            .borrow_mut()
            .push(args.iter().map(|s| (*s).to_owned()).collect());
        if idx == self.fail_at {
            Err(std::io::Error::other("stub fail"))
        } else {
            Ok(())
        }
    }
    fn run_capture(&self, _dest: &Path, _args: &[&str]) -> std::io::Result<String> {
        // MERGE_HEAD present → merge_in_progress is true, so control
        // reaches the add/commit arms.
        Ok("deadbeefsha".into())
    }
}

#[test]
fn add_failure_surfaces_as_git_error() {
    // calls: 0=merge, 1=add(fail).
    let err = merge(&PathBuf::from("/x"), "p1-cmp", &StubGit::failing_at(1)).unwrap_err();
    assert!(
        matches!(err, Error::Git { op: "compaction merge add", .. }),
        "{err:?}"
    );
}

#[test]
fn commit_failure_surfaces_as_git_error() {
    // calls: 0=merge, 1=add, 2=commit(fail).
    let err = merge(&PathBuf::from("/x"), "p1-cmp", &StubGit::failing_at(2)).unwrap_err();
    assert!(
        matches!(err, Error::Git { op: "compaction merge commit", .. }),
        "{err:?}"
    );
}
