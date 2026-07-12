//! Conversation-repo scaffolding (ARCH §2.2).
//!
//! Embeds the [`template/`] directory at build time via `include_dir`,
//! so the `lernie` binary is self-contained — no runtime template
//! lookup. [`scaffold`] extracts the embedded tree to a destination,
//! creates the `root/` worktree subdir with `.gitattributes` pinning
//! `merge=ours` discipline, and initializes git inside `root/` via an
//! injected [`GitRunner`]. The custom merge driver `merge.ours.driver`
//! is registered via `git config` so the `merge=ours` attribute is
//! actually honored on hand-run merges (the harness's own merges
//! enforce the discipline more strictly via the rebase-time alignment
//! step in [`crate::prompt::merge`], but the .gitattributes + driver
//! pair is the backstop for any merge that bypasses the harness).

pub mod descriptions;

use include_dir::{Dir, include_dir};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::process::Command;

/// The embedded conversation-repo template (ARCH §2.2). Holds only the
/// control-plane files that live at the conv-repo root — `manifest.yaml`,
/// `workflow.yaml`, `providers.yaml`, `version`, `souls/`. The `root/`
/// worktree subdir and its `.gitattributes` are written by [`scaffold`]
/// rather than embedded so the file shape stays inspectable.
pub static TEMPLATE: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/template");

/// `merge=ours` pins committed on `main` inside `root/` at scaffold
/// time (ARCH §2.6). Goal, soul, and the running summary are
/// branch-scoped — without this driver, every subagent merge-back
/// would clobber the parent's copies on a both-modified merge.
pub const ROOT_GITATTRIBUTES: &str =
    "goal.md     merge=ours\nsoul.md     merge=ours\nsummary/**  merge=ours\n";

/// Pathspecs the merge=ours discipline (ARCH §2.6) applies to.
/// Shared by [`ROOT_GITATTRIBUTES`] (which uses `summary/**` glob) and
/// [`crate::prompt::merge`]'s rebase-time alignment (which treats
/// `summary` as a directory pathspec for `git rm -r` /
/// `git checkout`). Order is irrelevant; the harness applies all of
/// them in one pass.
pub const MERGE_OURS_PATHS: &[&str] = &["goal.md", "soul.md", "summary"];

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

/// Subdir under the conv-repo where the primary worktree (and the only
/// `.git`) lives (ARCH §2.2).
pub const ROOT_WORKTREE: &str = "root";

/// Create a new conversation repo at `dest` per ARCH §2.2:
///
/// 1. Refuse if `dest` already exists and is non-empty.
/// 2. Extract the embedded [`TEMPLATE`] tree to `dest/` (control-plane
///    files at the conv-repo root, outside any worktree).
/// 3. Create `dest/root/` and write `.gitattributes` pinning the
///    `merge=ours` rules from ARCH §2.6.
/// 4. Snapshot the descriptions-always tree (ARCH §3.3): every tool
///    schema and skill frontmatter from the `data_root` pools is copied
///    into `dest/root/descriptions/{tools,skills}/` so the initial commit
///    carries it and every agent branch inherits it via git. An empty or
///    absent pool yields an empty descriptions tree.
/// 5. Run `git init -b main`, register the `merge.ours.driver` config
///    so the `.gitattributes` rules are actually honored (without it
///    git silently ignores `merge=ours` on a hand-run merge), then
///    `git add -A` + an initial `git commit -m "init conversation repo"`
///    *inside* `dest/root/` via the supplied [`GitRunner`]. The `.git`
///    lives in `root/`; the control files at the conv-repo root are
///    deliberately untracked, while `descriptions/**` is tracked context.
pub fn scaffold<G: GitRunner>(dest: &Path, data_root: &Path, git: &G) -> Result<(), ScaffoldError> {
    check_dest(dest)?;
    fs::create_dir_all(dest).map_err(ScaffoldError::Io)?;
    TEMPLATE.extract(dest).map_err(ScaffoldError::Io)?;
    let root = dest.join(ROOT_WORKTREE);
    fs::create_dir_all(&root).map_err(ScaffoldError::Io)?;
    fs::write(root.join(".gitattributes"), ROOT_GITATTRIBUTES).map_err(ScaffoldError::Io)?;
    descriptions::snapshot(data_root, &root).map_err(ScaffoldError::Descriptions)?;
    git.run(&root, &["init", "-b", "main"])
        .map_err(ScaffoldError::Git)?;
    // `git config merge.ours.driver true`: registers a no-op driver
    // (`true` exits 0) so the `merge=ours` attribute resolves to
    // "keep ours" instead of being silently dropped. Per-repo config
    // is shared across all worktrees of this `.git` dir, so a single
    // registration at scaffold covers every subagent worktree the
    // harness ever spawns (ARCH §2.2).
    git.run(&root, &["config", "merge.ours.driver", "true"])
        .map_err(ScaffoldError::Git)?;
    git.run(&root, &["add", "-A"]).map_err(ScaffoldError::Git)?;
    git.run(&root, &["commit", "-m", "init conversation repo"])
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
