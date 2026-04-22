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

    /// Like [`GitRunner::run`], but captures stdout and returns it as a
    /// trimmed string. Used by commands that need the output (e.g.
    /// `git rev-parse HEAD` after a commit).
    fn run_capture(&self, dest: &Path, args: &[&str]) -> io::Result<String>;
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
        self.run_capture(dest, args).map(|_| ())
    }

    fn run_capture(&self, dest: &Path, args: &[&str]) -> io::Result<String> {
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
        Ok(String::from_utf8_lossy(&out.stdout).trim().to_string())
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
mod tests;
