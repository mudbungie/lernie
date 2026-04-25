//! Rebase-then-no-ff merge protocol (ARCH §2.6).
//!
//! One shared routine for every parent/child merge-back in v0.3. The
//! compactor merges back into its dispatching branch through it; the
//! dispatching root conversation then merges into `main` through it.
//! A single path mirrors "One obvious path" (`docs/PRINCIPLES.md`):
//! both merges share the same core operation — rebase child onto
//! parent tip, `--no-ff` merge child into parent, remove the child's
//! worktree — and differ only in which paths name parent and child.
//!
//! §2.6 step 5 says a rebase that conflicts "indicates a harness
//! defect — two branches were given overlapping write paths". v0.3
//! does not have the single-author-per-file machinery in place yet,
//! so in practice the conflict path is reached when concurrent root
//! conversations overlap on e.g. `goal.md` — which v0.3 does not
//! test. For v0.3 we surface conflicts as [`Error::Git`] with
//! `op: "rebase"`, aborting the rebase so the worktree is left in a
//! clean state (no mid-rebase garbage) and the operator can retry.

use super::Error;
use crate::template::GitRunner;
use std::path::Path;

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
pub(super) fn rebase_and_merge(
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
        // operator needs to see.
        let _ = git.run(child_worktree, &["rebase", "--abort"]);
        return Err(Error::Git {
            op: "rebase",
            source,
        });
    }

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

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;
    use std::path::PathBuf;

    /// Recording git stub. `fail_mask` takes a slice of call indices
    /// to fail on; all other calls succeed. This covers every branch
    /// `rebase_and_merge` has: single-failure paths (0-indexed per
    /// step) and the double-failure path (rebase AND its abort).
    struct ScriptedGit {
        runs: RefCell<Vec<(PathBuf, Vec<String>)>>,
        fail_mask: Vec<usize>,
    }
    impl ScriptedGit {
        fn new(fail_mask: &[usize]) -> Self {
            Self {
                runs: RefCell::new(Vec::new()),
                fail_mask: fail_mask.to_vec(),
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
        fn run_capture(&self, _: &Path, _: &[&str]) -> std::io::Result<String> {
            unreachable!()
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

    #[test]
    fn rebase_and_merge_issues_rebase_then_merge_then_remove() {
        let git = ScriptedGit::new(&[]);
        run_it(&git).unwrap();
        let runs = git.runs.borrow();
        assert_eq!(runs.len(), 3, "rebase + merge + worktree remove");
        assert_eq!(runs[0].0, Path::new("/repo/child-wt"));
        assert_eq!(runs[0].1, vec!["rebase", "parent"]);
        assert_eq!(runs[1].0, Path::new("/repo/parent-wt"));
        assert_eq!(runs[1].1, vec!["merge", "--no-ff", "child"]);
        assert_eq!(runs[2].0, Path::new("/repo"));
        assert_eq!(runs[2].1[0], "worktree");
        assert_eq!(runs[2].1[1], "remove");
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
    fn merge_failure_surfaces_as_git_merge_error() {
        // Rebase ok (0), merge fails (1). Remove never runs.
        let git = ScriptedGit::new(&[1]);
        let err = run_it(&git).unwrap_err();
        assert!(matches!(err, Error::Git { op: "merge", .. }));
        let runs = git.runs.borrow();
        assert_eq!(runs.len(), 2);
        assert_eq!(runs[0].1[0], "rebase");
        assert_eq!(runs[1].1[0], "merge");
    }

    #[test]
    fn worktree_remove_failure_surfaces_as_git_worktree_remove_error() {
        let git = ScriptedGit::new(&[2]);
        let err = run_it(&git).unwrap_err();
        assert!(matches!(
            err,
            Error::Git {
                op: "worktree remove",
                ..
            }
        ));
    }
}
