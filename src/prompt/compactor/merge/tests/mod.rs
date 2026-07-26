//! Tests for the compaction merge (ARCH §2.6).
//!
//! The behavioral arms here run against a **real** git repo, so the
//! `--no-ff` land and the two pins — the live-branch-wins resolution of a
//! work-product-deletion overlap, and the filter that keeps the
//! compactor's own dialog off the dispatching branch — are exercised end
//! to end. The git-op error arms live in [`stub`].

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
    compactor_branch_with_dialog(wt, summary, deletions, &[]);
}

/// [`compactor_branch`] plus the compactor's **own** private context —
/// the `goal.md`/`soul.md` its dispatch commit rewrites and the
/// transcript entries its step loop appends (§2.3). Everything in
/// `dialog` is the compactor's record, not the dispatching branch's.
fn compactor_branch_with_dialog(
    wt: &Path,
    summary: (&str, &str),
    deletions: &[&str],
    dialog: &[(&str, &str)],
) {
    let git = g();
    git.run(wt, &["checkout", "-b", "agents/p1-cmp"]).unwrap();
    for (rel, content) in dialog {
        write(wt, rel, content);
    }
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
    compactor_branch(
        wt,
        ("summary/001.md", "digest\n"),
        &["messages/001-user.md"],
    );
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
fn the_compactors_own_dialog_never_crosses_the_merge() {
    // THE PIN (§2.6 filtered to the compaction product): the compactor's
    // branch carries its own dispatch `goal.md`/`soul.md` and its own
    // transcript entries. Only the summary and the deletions cross; the
    // private dialog stays on the compactor's ref.
    let dir = repo_at_checkpoint(&[
        ("goal.md", "parent goal\n"),
        ("soul.md", "parent soul\n"),
        ("messages/001-user.md", "hi\n"),
        ("messages/002-a.md", "reply\n"),
    ]);
    let wt = dir.path();
    compactor_branch_with_dialog(
        wt,
        ("summary/001.md", "digest\n"),
        &["messages/001-user.md"],
        &[
            ("goal.md", "compact the branch\n"),
            ("soul.md", "compactor soul\n"),
            ("messages/003-goal.md", "compact the branch\n"),
            ("messages/004-model.json", "{}\n"),
            ("messages/005-tool.json", "{\"error\":\"no such path\"}\n"),
        ],
    );
    advance_live(wt, &[("messages/006-b.md", "later\n")]);

    assert_eq!(merge(wt, "p1-cmp", &g()).unwrap(), MergeOutcome::Merged);
    assert_eq!(
        std::fs::read_to_string(wt.join("summary/001.md")).unwrap(),
        "digest\n",
        "the compaction product lands"
    );
    assert!(!wt.join("messages/001-user.md").exists(), "deletion landed");
    assert!(wt.join("messages/006-b.md").exists(), "live append kept");
    for private in ["003-goal.md", "004-model.json", "005-tool.json"] {
        assert!(
            !wt.join("messages").join(private).exists(),
            "compactor transcript entry {private} crossed the merge"
        );
    }
    // The dispatching branch keeps its own pinned context (§2.8), and
    // nothing is left unstaged: the filter is committed state, not a
    // worktree edit the next `add -A` would resurrect.
    let read = |rel: &str| std::fs::read_to_string(wt.join(rel)).unwrap();
    assert_eq!(read("goal.md"), "parent goal\n");
    assert_eq!(read("soul.md"), "parent soul\n");
    assert_eq!(g().run_capture(wt, &["status", "--porcelain"]).unwrap(), "");
    // The compactor's own ref keeps its full transcript — its record is
    // its own branch (§2.6).
    assert!(
        g().run_capture(
            wt,
            &["cat-file", "-e", "agents/p1-cmp:messages/004-model.json"]
        )
        .is_ok(),
        "the compactor branch keeps its own dialog"
    );
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
    assert_git_op(err, "compaction merge");
}

/// Assert `err` is the git failure of operation `want`.
fn assert_git_op(err: Error, want: &str) {
    match err {
        Error::Git { op, .. } => assert_eq!(op, want),
        other => panic!("{other:?}"),
    }
}

mod stub;
