//! Workspace creation and first config-commit authoring (ARCH §2.2).
//!
//! Embeds the [`template/`] directory at build time via `include_dir`,
//! so the `lernie` binary is self-contained — no runtime template
//! lookup. [`scaffold`] creates the bare workspace repository at
//! `<dest>/repo.git` and authors the workspace's **first config
//! commit** — an orphan root on `config/default` (§2.2) — as the
//! harness-assisted act §2.2 describes: materialize a checkout, write
//! the control files from the embedded template plus the
//! `descriptions/**` snapshot from the data-root pools (§3.3), commit,
//! and tear the checkout down. There is no `main` and no primary
//! worktree: agents fork off the config branch's head (§2.3), and the
//! fork *is* the freeze (§2.2).

pub mod descriptions;

use include_dir::{Dir, include_dir};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::process::Command;

/// The embedded config-commit template (ARCH §2.2). Holds the control
/// files a config commit carries — `manifest.yaml`, `workflow.yaml`,
/// `providers.yaml`, `version`, `souls/` — authored onto the orphan
/// `config/default` root by [`scaffold`].
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
    #[error("descriptions-always: {0}")]
    Descriptions(#[source] descriptions::Error),
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

/// Transient checkout directory used while authoring the first config
/// commit; removed (as a git worktree) once the commit lands.
const CONFIG_AUTHOR_DIR: &str = ".config-author";

/// Create a new workspace at `dest` per ARCH §2.2:
///
/// 1. Refuse if `dest` already exists and is non-empty.
/// 2. `git init --bare -b config/default <dest>/repo.git` — the
///    workspace repository. No `main` is ever created (§2.2).
/// 3. Author the first config commit (an orphan root, §2.2) through a
///    transient checkout: `git worktree add --orphan`, extract the
///    embedded [`TEMPLATE`] control files, snapshot the
///    descriptions-always tree from the `data_root` pools into
///    `descriptions/{tools,skills}/` (ARCH §3.3 — an empty or absent
///    pool yields an empty descriptions tree), `git add -A`, commit.
/// 4. Remove the authoring worktree. The workspace is left with exactly
///    one ref, `config/default`, whose head is the config commit every
///    fresh root agent forks off (§2.3) — the fork is the freeze.
pub fn scaffold<G: GitRunner>(dest: &Path, data_root: &Path, git: &G) -> Result<(), ScaffoldError> {
    check_dest(dest)?;
    let repo = crate::workspace::repo_git(dest);
    let config_ref = crate::workspace::DEFAULT_CONFIG_REF;
    fs::create_dir_all(&repo).map_err(ScaffoldError::Io)?;
    let init_args = ["init", "--bare", "-b", config_ref];
    git.run(&repo, &init_args).map_err(ScaffoldError::Git)?;

    let author = dest.join(CONFIG_AUTHOR_DIR);
    let author_str = author.to_string_lossy().to_string();
    let mut add_args = vec!["worktree", "add", "--orphan", "-b", config_ref];
    add_args.push(author_str.as_str());
    git.run(&repo, &add_args).map_err(ScaffoldError::Git)?;

    // `git worktree add` creates the directory in production; the
    // explicit `create_dir_all` is for stub-git tests (and a harmless
    // no-op in production) — the same pattern as the subagent spawn.
    fs::create_dir_all(&author).map_err(ScaffoldError::Io)?;
    TEMPLATE.extract(&author).map_err(ScaffoldError::Io)?;
    descriptions::snapshot(data_root, &author).map_err(ScaffoldError::Descriptions)?;
    git.run(&author, &["add", "-A"])
        .map_err(ScaffoldError::Git)?;
    let msg = format!("config: init [{config_ref}]");
    git.run(&author, &["commit", "-m", msg.as_str()])
        .map_err(ScaffoldError::Git)?;
    git.run(&repo, &["worktree", "remove", author_str.as_str()])
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
#[cfg(test)]
mod tests_realgit;
