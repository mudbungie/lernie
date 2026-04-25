//! Unit tests for [`super::rebase_and_merge`] and the alignment
//! helper. Lives in a sibling file rather than an inline `mod tests`
//! so the production module stays well under the 300-line repo cap.

use super::*;
use std::cell::RefCell;
use std::path::PathBuf;

/// Recording git stub. `fail_mask` takes a slice of call indices
/// to fail on; all other calls succeed. `captures` is a queue of
/// pre-canned `run_capture` results; tests that exercise the
/// alignment branches push the listings or staged-name strings
/// in the order the harness asks for them.
struct ScriptedGit {
    runs: RefCell<Vec<(PathBuf, Vec<String>)>>,
    fail_mask: Vec<usize>,
    captures: RefCell<Vec<String>>,
}

impl ScriptedGit {
    fn new(fail_mask: &[usize]) -> Self {
        Self::with_captures(fail_mask, &[])
    }
    fn with_captures(fail_mask: &[usize], captures: &[&str]) -> Self {
        // Reversed so `pop()` yields the captures in the order they
        // were passed — first capture in the slice is the first one
        // returned (FIFO), which mirrors how the harness consumes
        // them.
        Self {
            runs: RefCell::new(Vec::new()),
            fail_mask: fail_mask.to_vec(),
            captures: RefCell::new(captures.iter().rev().map(|s| (*s).to_string()).collect()),
        }
    }
}

impl GitRunner for ScriptedGit {
    fn run(&self, dest: &Path, args: &[&str]) -> std::io::Result<()> {
        let idx = self.runs.borrow().len();
        self.runs.borrow_mut().push((
            dest.to_path_buf(),
            args.iter().map(|s| (*s).to_owned()).collect(),
        ));
        if self.fail_mask.contains(&idx) {
            Err(std::io::Error::other(format!("scripted fail {idx}")))
        } else {
            Ok(())
        }
    }
    fn run_capture(&self, dest: &Path, args: &[&str]) -> std::io::Result<String> {
        self.run(dest, args)?;
        // Empty by default; tests that need a canned string seed
        // `captures` in advance (FIFO).
        Ok(self.captures.borrow_mut().pop().unwrap_or_default())
    }
}

fn run_it(git: &dyn GitRunner) -> Result<(), Error> {
    rebase_and_merge(
        Path::new("/repo"),
        "parent",
        Path::new("/repo/parent-wt"),
        Path::new("/repo/child-wt"),
        "child",
        git,
    )
}

/// Happy path with the parent carrying nothing under the
/// merge=ours pathspecs and no staged delta after the rm: ls-tree
/// returns empty (so checkout is skipped), diff --cached returns
/// empty (so the alignment commit is skipped). The harness still
/// runs the rebase, the rm, and the two captures, plus the merge
/// and worktree-remove — six calls total, two of which are
/// `run_capture`.
#[test]
fn rebase_and_merge_skips_alignment_commit_when_nothing_to_align() {
    let git = ScriptedGit::new(&[]);
    run_it(&git).unwrap();
    let runs = git.runs.borrow();
    assert_eq!(runs.len(), 6);
    assert_eq!(runs[0].0, Path::new("/repo/child-wt"));
    assert_eq!(runs[0].1, vec!["rebase", "parent"]);
    assert_eq!(runs[1].0, Path::new("/repo/child-wt"));
    assert_eq!(
        runs[1].1,
        vec![
            "rm",
            "-r",
            "--ignore-unmatch",
            "--",
            "goal.md",
            "soul.md",
            "summary"
        ]
    );
    assert_eq!(runs[2].0, Path::new("/repo/child-wt"));
    assert_eq!(
        runs[2].1,
        vec![
            "ls-tree",
            "-r",
            "--name-only",
            "parent",
            "--",
            "goal.md",
            "soul.md",
            "summary"
        ]
    );
    assert_eq!(runs[3].0, Path::new("/repo/child-wt"));
    assert_eq!(runs[3].1, vec!["diff", "--cached", "--name-only"]);
    assert_eq!(runs[4].0, Path::new("/repo/parent-wt"));
    assert_eq!(runs[4].1, vec!["merge", "--no-ff", "child"]);
    assert_eq!(runs[5].0, Path::new("/repo"));
    assert_eq!(runs[5].1[..2], ["worktree", "remove"]);
}

/// When the parent has matching paths AND the rm produced staged
/// deletions, both the checkout and the alignment commit fire.
/// FIFO captures: first slice element is the ls-tree result,
/// second is the diff result.
#[test]
fn alignment_runs_checkout_and_commit_when_changes_exist() {
    let git = ScriptedGit::with_captures(&[], &["goal.md\nsummary/001.md", "goal.md"]);
    run_it(&git).unwrap();
    let runs = git.runs.borrow();
    // rebase + rm + ls-tree + checkout + diff + commit + merge +
    // worktree-remove.
    assert_eq!(runs.len(), 8);
    assert_eq!(runs[3].0, Path::new("/repo/child-wt"));
    assert_eq!(
        runs[3].1,
        vec!["checkout", "parent", "--", "goal.md", "summary"]
    );
    assert_eq!(runs[5].0, Path::new("/repo/child-wt"));
    assert_eq!(runs[5].1[0], "commit");
    assert!(runs[5].1[2].contains("merge=ours alignment with parent"));
    assert_eq!(runs[6].1, vec!["merge", "--no-ff", "child"]);
    assert_eq!(runs[7].1[0], "worktree");
    assert_eq!(runs[7].1[1], "remove");
}

#[test]
fn rebase_failure_aborts_and_surfaces_as_git_rebase_error() {
    // Fail the rebase (0); abort (1) succeeds; merge + remove
    // never run.
    let git = ScriptedGit::new(&[0]);
    let err = run_it(&git).unwrap_err();
    assert!(matches!(err, Error::Git { op: "rebase", .. }));
    let runs = git.runs.borrow();
    assert_eq!(runs.len(), 2);
    assert_eq!(runs[0].1, vec!["rebase", "parent"]);
    assert_eq!(runs[1].1, vec!["rebase", "--abort"]);
}

#[test]
fn rebase_abort_failure_is_swallowed_but_rebase_error_surfaces() {
    // Rebase AND abort both fail. The primary error still
    // surfaces; we cannot do more than report it.
    let git = ScriptedGit::new(&[0, 1]);
    let err = run_it(&git).unwrap_err();
    assert!(matches!(err, Error::Git { op: "rebase", .. }));
    let runs = git.runs.borrow();
    assert_eq!(runs.len(), 2);
}

#[test]
fn merge_ours_rm_failure_surfaces() {
    let git = ScriptedGit::new(&[1]);
    let err = run_it(&git).unwrap_err();
    assert!(matches!(
        err,
        Error::Git {
            op: "merge=ours rm",
            ..
        }
    ));
}

#[test]
fn merge_ours_ls_tree_failure_surfaces() {
    let git = ScriptedGit::new(&[2]);
    let err = run_it(&git).unwrap_err();
    assert!(matches!(
        err,
        Error::Git {
            op: "merge=ours ls-tree",
            ..
        }
    ));
}

#[test]
fn merge_ours_checkout_failure_surfaces() {
    // Need ls-tree to report a path so checkout actually runs;
    // failing index 3 = the checkout call. FIFO capture: first
    // (and only) string is the ls-tree result.
    let git = ScriptedGit::with_captures(&[3], &["goal.md"]);
    let err = run_it(&git).unwrap_err();
    assert!(matches!(
        err,
        Error::Git {
            op: "merge=ours checkout",
            ..
        }
    ));
}

#[test]
fn merge_ours_diff_failure_surfaces() {
    // No checkout → diff is index 3.
    let git = ScriptedGit::new(&[3]);
    let err = run_it(&git).unwrap_err();
    assert!(matches!(
        err,
        Error::Git {
            op: "merge=ours diff",
            ..
        }
    ));
}

#[test]
fn merge_ours_commit_failure_surfaces() {
    // FIFO captures: first is the ls-tree result (empty → no
    // checkout), second is the diff result (non-empty → commit
    // fires). With no checkout, commit is index 4.
    let git = ScriptedGit::with_captures(&[4], &["", "goal.md"]);
    let err = run_it(&git).unwrap_err();
    assert!(matches!(
        err,
        Error::Git {
            op: "merge=ours commit",
            ..
        }
    ));
}

#[test]
fn merge_failure_surfaces_as_git_merge_error() {
    // Rebase + rm + ls-tree + diff ok (no checkout, no commit),
    // merge fails at index 4.
    let git = ScriptedGit::new(&[4]);
    let err = run_it(&git).unwrap_err();
    assert!(matches!(err, Error::Git { op: "merge", .. }));
}

#[test]
fn worktree_remove_failure_surfaces_as_git_worktree_remove_error() {
    // Same default sequence as above; worktree remove is at
    // index 5.
    let git = ScriptedGit::new(&[5]);
    let err = run_it(&git).unwrap_err();
    assert!(matches!(
        err,
        Error::Git {
            op: "worktree remove",
            ..
        }
    ));
}
