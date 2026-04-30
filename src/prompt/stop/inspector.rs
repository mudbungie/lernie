//! Branch-state lookups for `lernie stop`.
//!
//! Two questions need answering before sending any signal:
//! - does `<branch>` exist as a ref in the conv-repo?
//! - is it already merged into `main`?
//!
//! Both are exit-code questions on `git`, so the trait is shaped
//! around them rather than a generic "run-git" surface. Tests inject
//! a stub that returns the bits directly; production shells out via
//! the supplied [`GitRunner`] inside `<repo>/root/` (ARCH §2.2 — the
//! primary worktree where `.git` lives).

use crate::template::GitRunner;
use std::io;
use std::path::Path;

/// Subdir under the conv-repo where the primary worktree (and the
/// only `.git`) lives. Mirrors [`crate::template::ROOT_WORKTREE`] —
/// duplicated here rather than re-exported so the stop module is a
/// single read.
const ROOT_WORKTREE: &str = "root";

/// The two ref-state questions [`super::run`] needs answered before
/// signaling. The trait is `&dyn`-shaped so tests pass a stub and
/// production passes [`GitInspector`] without paying the subprocess
/// cost in the test path.
pub trait BranchInspector {
    fn exists(&self, repo: &Path, branch: &str, git: &dyn GitRunner) -> io::Result<bool>;
    fn is_merged_into_main(
        &self,
        repo: &Path,
        branch: &str,
        git: &dyn GitRunner,
    ) -> io::Result<bool>;
}

/// Production [`BranchInspector`] — runs `git rev-parse --verify
/// refs/heads/<branch>` and `git merge-base --is-ancestor <branch>
/// main` inside `<repo>/root/`.
#[derive(Debug, Default, Clone, Copy)]
pub struct GitInspector;

impl BranchInspector for GitInspector {
    fn exists(&self, repo: &Path, branch: &str, git: &dyn GitRunner) -> io::Result<bool> {
        let root = repo.join(ROOT_WORKTREE);
        let refspec = format!("refs/heads/{branch}");
        // `rev-parse --verify` exits 0 when the ref resolves, non-
        // zero otherwise. `GitRunner::run` surfaces non-zero as `Err`
        // — translate that to the boolean we want.
        match git.run(&root, &["rev-parse", "--verify", "--quiet", &refspec]) {
            Ok(()) => Ok(true),
            Err(_) => Ok(false),
        }
    }

    fn is_merged_into_main(
        &self,
        repo: &Path,
        branch: &str,
        git: &dyn GitRunner,
    ) -> io::Result<bool> {
        let root = repo.join(ROOT_WORKTREE);
        // `merge-base --is-ancestor <branch> main` exits 0 when
        // branch's tip is reachable from main (i.e. merged), non-
        // zero when it is not. Same exit-code-as-boolean shape.
        match git.run(&root, &["merge-base", "--is-ancestor", branch, "main"]) {
            Ok(()) => Ok(true),
            Err(_) => Ok(false),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;
    use std::path::PathBuf;

    struct RecordingGit {
        invocations: RefCell<Vec<Vec<String>>>,
        result: io::Result<()>,
    }

    impl GitRunner for RecordingGit {
        fn run(&self, _: &Path, args: &[&str]) -> io::Result<()> {
            self.invocations
                .borrow_mut()
                .push(args.iter().map(|s| (*s).to_owned()).collect());
            // `io::Error` isn't `Clone`, so we mirror the kind /
            // message rather than re-emit the same instance.
            match &self.result {
                Ok(()) => Ok(()),
                Err(e) => Err(io::Error::new(e.kind(), e.to_string())),
            }
        }
        fn run_capture(&self, _: &Path, _: &[&str]) -> io::Result<String> {
            unreachable!("inspector never calls run_capture")
        }
    }

    fn ok_git() -> RecordingGit {
        RecordingGit {
            invocations: RefCell::new(Vec::new()),
            result: Ok(()),
        }
    }
    fn err_git() -> RecordingGit {
        RecordingGit {
            invocations: RefCell::new(Vec::new()),
            result: Err(io::Error::other("boom")),
        }
    }

    #[test]
    fn exists_true_on_zero_exit() {
        let git = ok_git();
        assert!(
            GitInspector
                .exists(&PathBuf::from("/r"), "br", &git)
                .unwrap()
        );
        let calls = git.invocations.borrow();
        assert_eq!(
            calls[0],
            vec!["rev-parse", "--verify", "--quiet", "refs/heads/br"]
        );
    }

    #[test]
    fn exists_false_on_nonzero_exit() {
        let git = err_git();
        assert!(
            !GitInspector
                .exists(&PathBuf::from("/r"), "br", &git)
                .unwrap()
        );
    }

    #[test]
    fn is_merged_true_on_zero_exit() {
        let git = ok_git();
        assert!(
            GitInspector
                .is_merged_into_main(&PathBuf::from("/r"), "br", &git)
                .unwrap()
        );
        let calls = git.invocations.borrow();
        assert_eq!(calls[0], vec!["merge-base", "--is-ancestor", "br", "main"]);
    }

    #[test]
    fn is_merged_false_on_nonzero_exit() {
        let git = err_git();
        assert!(
            !GitInspector
                .is_merged_into_main(&PathBuf::from("/r"), "br", &git)
                .unwrap()
        );
    }
}
