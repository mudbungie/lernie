//! Conversation-repo scaffolding.
//!
//! Embeds the [`template/`] directory at build time via `include_dir`,
//! so the `lernie` binary is self-contained — no runtime template
//! lookup. [`scaffold`] extracts the embedded tree to a destination and
//! initializes git via an injected [`GitRunner`], which makes the
//! orchestration testable without a mock HTTP layer or PATH hacks.

use include_dir::{Dir, include_dir};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::process::Command;

/// The embedded conversation-repo template (ARCH §2.2).
pub static TEMPLATE: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/template");

/// Errors [`scaffold`] can return.
#[derive(Debug, thiserror::Error)]
pub enum ScaffoldError {
    #[error("destination {0} already exists and is not empty")]
    DestNotEmpty(PathBuf),
    #[error("I/O error: {0}")]
    Io(#[source] io::Error),
    #[error("git error: {0}")]
    Git(#[source] io::Error),
}

/// Abstraction over running `git` subcommands inside a target directory.
/// Implemented for [`RealGit`] by shelling out; tests supply their own
/// implementations to exercise the error paths in [`scaffold`].
pub trait GitRunner {
    /// Run `git <args>` with `-C dest`. Returns `Err` when the process
    /// cannot start or exits non-zero.
    fn run(&self, dest: &Path, args: &[&str]) -> io::Result<()>;
}

/// `GitRunner` that invokes a `git` binary on disk.
///
/// The binary path is a field so tests can swap in a nonexistent path
/// to exercise the spawn-failure branch.
pub struct RealGit {
    bin: PathBuf,
}

impl RealGit {
    /// Use the `git` found on `PATH`.
    pub fn new() -> Self {
        Self {
            bin: PathBuf::from("git"),
        }
    }
}

impl Default for RealGit {
    fn default() -> Self {
        Self::new()
    }
}

impl GitRunner for RealGit {
    fn run(&self, dest: &Path, args: &[&str]) -> io::Result<()> {
        // When invoked from a git-hook context, GIT_DIR / GIT_INDEX_FILE
        // / GIT_WORK_TREE / GIT_OBJECT_DIRECTORY are in the environment
        // and would cause the child `git` to operate on the outer repo
        // regardless of `-C`. Scrub them before spawning.
        let mut cmd = Command::new(&self.bin);
        for var in INHERITED_GIT_ENV {
            cmd.env_remove(var);
        }
        let out = cmd.arg("-C").arg(dest).args(args).output()?;
        if !out.status.success() {
            return Err(io::Error::other(format!(
                "git {args:?} exited with {}: {}",
                out.status,
                String::from_utf8_lossy(&out.stderr).trim()
            )));
        }
        Ok(())
    }
}

const INHERITED_GIT_ENV: &[&str] = &[
    "GIT_DIR",
    "GIT_WORK_TREE",
    "GIT_INDEX_FILE",
    "GIT_OBJECT_DIRECTORY",
    "GIT_PREFIX",
    "GIT_COMMON_DIR",
    "GIT_ALTERNATE_OBJECT_DIRECTORIES",
];

/// Create a new conversation repo at `dest`:
///
/// 1. Refuse if `dest` already exists and is non-empty.
/// 2. Extract the embedded [`TEMPLATE`] tree into `dest`.
/// 3. Run `git init -b main`, `git add -A`, and an initial
///    `git commit -m "init conversation repo"` via the supplied
///    [`GitRunner`].
///
/// The commit message is the v0.1 success-criterion string from ARCH §12.
pub fn scaffold<G: GitRunner>(dest: &Path, git: &G) -> Result<(), ScaffoldError> {
    check_dest(dest)?;
    TEMPLATE.extract(dest).map_err(ScaffoldError::Io)?;
    git.run(dest, &["init", "-b", "main"])
        .map_err(ScaffoldError::Git)?;
    git.run(dest, &["add", "-A"]).map_err(ScaffoldError::Git)?;
    git.run(dest, &["commit", "-m", "init conversation repo"])
        .map_err(ScaffoldError::Git)?;
    Ok(())
}

fn check_dest(dest: &Path) -> Result<(), ScaffoldError> {
    match fs::read_dir(dest) {
        Ok(mut entries) => {
            if entries.next().is_some() {
                Err(ScaffoldError::DestNotEmpty(dest.to_path_buf()))
            } else {
                Ok(())
            }
        }
        Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(()),
        Err(e) => Err(ScaffoldError::Io(e)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;
    use tempfile::TempDir;

    // --- check_dest --------------------------------------------------
    #[test]
    fn check_dest_allows_missing_path() {
        let holder = TempDir::new().unwrap();
        let missing = holder.path().join("nope");
        check_dest(&missing).unwrap();
    }

    #[test]
    fn check_dest_allows_empty_directory() {
        let holder = TempDir::new().unwrap();
        check_dest(holder.path()).unwrap();
    }

    #[test]
    fn check_dest_rejects_non_empty_directory() {
        let holder = TempDir::new().unwrap();
        fs::write(holder.path().join("occupant"), b"x").unwrap();
        let err = check_dest(holder.path()).unwrap_err();
        assert!(matches!(err, ScaffoldError::DestNotEmpty(_)));
    }

    #[test]
    fn check_dest_surfaces_other_io_errors() {
        // read_dir on a regular file fails with kind != NotFound, so the
        // third arm of check_dest fires.
        let holder = TempDir::new().unwrap();
        let file = holder.path().join("actually-a-file");
        fs::write(&file, b"not a dir").unwrap();
        let err = check_dest(&file).unwrap_err();
        assert!(matches!(err, ScaffoldError::Io(_)), "got {err:?}");
    }

    // --- scaffold orchestration via stub GitRunner -------------------
    /// Records every `git` subprocess run and can be programmed to fail
    /// at a chosen run index.
    struct StubGit {
        runs: RefCell<Vec<Vec<String>>>,
        fail_at: Option<usize>,
    }

    impl StubGit {
        fn ok() -> Self {
            Self {
                runs: RefCell::new(Vec::new()),
                fail_at: None,
            }
        }
        fn failing_at(idx: usize) -> Self {
            Self {
                runs: RefCell::new(Vec::new()),
                fail_at: Some(idx),
            }
        }
    }

    impl GitRunner for StubGit {
        fn run(&self, _dest: &Path, args: &[&str]) -> io::Result<()> {
            let mut runs = self.runs.borrow_mut();
            let idx = runs.len();
            runs.push(args.iter().map(|s| (*s).to_owned()).collect());
            if self.fail_at == Some(idx) {
                Err(io::Error::other(format!("stub fail at {idx}")))
            } else {
                Ok(())
            }
        }
    }

    #[test]
    fn scaffold_happy_path_runs_git_in_order() {
        let holder = TempDir::new().unwrap();
        let dest = holder.path().join("conv");
        let git = StubGit::ok();
        scaffold(&dest, &git).unwrap();
        let runs = git.runs.borrow();
        assert_eq!(runs.len(), 3);
        assert_eq!(runs[0], vec!["init", "-b", "main"]);
        assert_eq!(runs[1], vec!["add", "-A"]);
        assert_eq!(runs[2], vec!["commit", "-m", "init conversation repo"]);
        assert!(dest.join(".agent/version").is_file());
        assert!(dest.join(".agent/system/prompts/base.md").is_file());
    }

    #[test]
    fn scaffold_propagates_init_failure() {
        let holder = TempDir::new().unwrap();
        let dest = holder.path().join("conv");
        let err = scaffold(&dest, &StubGit::failing_at(0)).unwrap_err();
        assert!(matches!(err, ScaffoldError::Git(_)), "got {err:?}");
    }

    #[test]
    fn scaffold_propagates_add_failure() {
        let holder = TempDir::new().unwrap();
        let dest = holder.path().join("conv");
        let err = scaffold(&dest, &StubGit::failing_at(1)).unwrap_err();
        assert!(matches!(err, ScaffoldError::Git(_)));
    }

    #[test]
    fn scaffold_propagates_commit_failure() {
        let holder = TempDir::new().unwrap();
        let dest = holder.path().join("conv");
        let err = scaffold(&dest, &StubGit::failing_at(2)).unwrap_err();
        assert!(matches!(err, ScaffoldError::Git(_)));
    }

    #[test]
    fn scaffold_refuses_non_empty_dest() {
        let holder = TempDir::new().unwrap();
        fs::write(holder.path().join("x"), b"x").unwrap();
        let err = scaffold(holder.path(), &StubGit::ok()).unwrap_err();
        assert!(matches!(err, ScaffoldError::DestNotEmpty(_)));
    }

    #[test]
    fn scaffold_surfaces_extract_io_error() {
        // A path segment that exists as a regular file makes include_dir's
        // `extract` fail when it tries to create the sub-directory —
        // hits the `ScaffoldError::Io` arm of scaffold().
        let holder = TempDir::new().unwrap();
        let blocker = holder.path().join("blocker");
        fs::write(&blocker, b"blocks extraction").unwrap();
        let dest = blocker.join("child");
        let err = scaffold(&dest, &StubGit::ok()).unwrap_err();
        assert!(matches!(err, ScaffoldError::Io(_)), "got {err:?}");
    }

    // --- RealGit -----------------------------------------------------
    #[test]
    fn realgit_default_matches_new() {
        let _ = RealGit::default();
    }

    #[test]
    fn realgit_succeeds_on_valid_command() {
        let holder = TempDir::new().unwrap();
        RealGit::new()
            .run(holder.path(), &["init", "-b", "main"])
            .unwrap();
        assert!(holder.path().join(".git").is_dir());
    }

    #[test]
    fn realgit_returns_error_on_nonzero_exit() {
        let holder = TempDir::new().unwrap();
        // No git repo here, so `git status` exits non-zero. That hits
        // the `!status.success()` branch without needing a missing
        // binary.
        let err = RealGit::new()
            .run(holder.path(), &["status", "--porcelain"])
            .unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains("exited with"), "unexpected: {msg}");
    }

    #[test]
    fn realgit_returns_error_when_binary_missing() {
        let holder = TempDir::new().unwrap();
        let git = RealGit {
            bin: PathBuf::from("/no/such/lernie-test-git"),
        };
        let err = git.run(holder.path(), &["init"]).unwrap_err();
        assert_eq!(err.kind(), io::ErrorKind::NotFound);
    }
}
