//! [`super::author`] failure arms that need a stubbed [`GitRunner`] to
//! fake the checkout: the two Io arms (checkout-dir collision, template
//! extraction) and the commit-step git failure. Split from
//! [`super::tests`] for the per-file line cap.

use super::{Error, Origin, author};
use crate::template::{GitRunner, RealGit, scaffold};
use std::fs;
use std::io;
use std::path::Path;
use tempfile::TempDir;

/// A scaffolded workspace with an empty pool. Returns `(holder, ws)`.
fn workspace() -> (TempDir, std::path::PathBuf) {
    let holder = TempDir::new().unwrap();
    let ws = holder.path().join("ws");
    scaffold(&ws, &holder.path().join("no-pool"), &RealGit::new()).unwrap();
    (holder, ws)
}

/// A no-op edit — every arm here fails before or at the commit, so what
/// the edit writes never matters.
fn noop(_dir: &Path) -> io::Result<()> {
    Ok(())
}

/// One configurable orphan-authoring stub: it runs `on_worktree_add` on
/// the checkout path (so a test can squat a file or dir there) and,
/// when `fail_commit` is set, fails the `git commit` step. Every other
/// command succeeds without touching the real repo.
struct StubGit<F: Fn(&Path)> {
    on_worktree_add: F,
    fail_commit: bool,
}

impl<F: Fn(&Path)> GitRunner for StubGit<F> {
    fn run(&self, _dest: &Path, args: &[&str]) -> io::Result<()> {
        if args.first() == Some(&"worktree") && args.get(1) == Some(&"add") {
            (self.on_worktree_add)(Path::new(args[args.len() - 1]));
        } else if self.fail_commit && args.first() == Some(&"commit") {
            return Err(io::Error::other("no identity"));
        }
        Ok(())
    }
    fn run_capture(&self, dest: &Path, args: &[&str]) -> io::Result<String> {
        self.run(dest, args).map(|_| String::new())
    }
}

fn run_orphan<F: Fn(&Path)>(ws: &Path, no_pool: &Path, git: &StubGit<F>) -> Result<(), Error> {
    author(ws, no_pool, "scratch", Origin::Orphan, noop, git)
}

#[test]
fn checkout_dir_collision_is_an_io_error() {
    // A regular file squats the checkout path, so `create_dir_all` on it
    // fails — the pre-extract Io arm.
    let (holder, ws) = workspace();
    let git = StubGit {
        on_worktree_add: |author: &Path| {
            let _ = fs::write(author, b"squat");
        },
        fail_commit: false,
    };
    let err = run_orphan(&ws, &holder.path().join("no-pool"), &git).unwrap_err();
    assert!(matches!(err, Error::Io(_)), "got {err:?}");
}

#[test]
fn template_extract_failure_is_an_io_error() {
    // A directory squats the `version` file target, so template
    // extraction fails — the post-create Io arm.
    let (holder, ws) = workspace();
    let git = StubGit {
        on_worktree_add: |author: &Path| {
            fs::create_dir_all(author.join("version")).unwrap();
        },
        fail_commit: false,
    };
    let err = run_orphan(&ws, &holder.path().join("no-pool"), &git).unwrap_err();
    assert!(matches!(err, Error::Io(_)), "got {err:?}");
}

#[test]
fn commit_step_failure_is_a_git_error() {
    // Worktree-add and add succeed against the real checkout, template
    // extracts, and `commit_checkout` fails at `git commit`.
    let (holder, ws) = workspace();
    let git = StubGit {
        on_worktree_add: |_author: &Path| {},
        fail_commit: true,
    };
    let err = run_orphan(&ws, &holder.path().join("no-pool"), &git).unwrap_err();
    assert!(matches!(err, Error::Git(_)), "got {err:?}");
}

#[test]
fn stub_run_capture_delegates_to_run() {
    // `author` only ever calls `run`; exercise the delegating
    // `run_capture` directly so the stub is fully covered.
    let git = StubGit {
        on_worktree_add: |_author: &Path| {},
        fail_commit: true,
    };
    let holder = TempDir::new().unwrap();
    assert_eq!(git.run_capture(holder.path(), &["status"]).unwrap(), "");
    assert!(git.run_capture(holder.path(), &["commit"]).is_err());
}
