//! The agent's **working-directory mark** — `refs/lernie/cwd/<agent-id>`
//! (ARCH §3.3 *Working directory*).
//!
//! An agent's working directory is one mutable per-agent fact: its
//! worktree by default, and thereafter whatever its own `cd` tool call
//! last set. This module is that fact's one home. It lives in the
//! per-agent **mark** namespace ([`super::MARK_REF_ROOT`], §2.2) beside
//! `conflicted` / `budget-exhausted` / `abandoned` / `notify`, so it is
//! reaped with the agent by `lernie delete` (§9.2 enumerates the mark
//! root, never a list of kinds), it crosses no fork and no transfer
//! (marks are keyed by agent id and nothing merges them), and it is not
//! context (§5.1 — the agent learns its cwd from the tool result, not
//! from its tree).
//!
//! **This mark carries a value where the others are bare assertions:**
//! the ref names a *blob* whose bytes are the absolute path. A ref may
//! name any object, so no second mechanism is needed to hold the one
//! extra fact — and `git gc` keeps the blob alive for exactly as long as
//! the mark does.
//!
//! The value round-trips through [`GitRunner::run_capture`], which
//! returns trimmed UTF-8, so [`write`] declines a directory whose path is
//! not preserved by that round trip rather than storing one that would
//! read back wrong (PRINCIPLES "Decline illegal operations").

use super::{MARK_REF_ROOT, repo_git};
use crate::template::GitRunner;
use std::io;
use std::os::unix::ffi::OsStrExt;
use std::path::{Path, PathBuf};

/// Ref-namespace prefix for the working-directory mark (§3.3).
pub const CWD_REF_PREFIX: &str = "cwd/";

/// `refs/lernie/cwd/<agent-id>` — the mark ref for one agent.
pub fn cwd_ref(agent_id: &str) -> String {
    format!("{MARK_REF_ROOT}{CWD_REF_PREFIX}{agent_id}")
}

/// The agent's stored working directory, or `None` when the mark is
/// unset — which is the ordinary state of an agent that never called
/// `cd`, not an error. An unreadable mark (no repo, a ref pointing at a
/// non-blob, a git that would not run) reads the same way: the caller's
/// default applies, and no tool call is lost to a mark.
pub fn read(workspace: &Path, agent_id: &str, git: &dyn GitRunner) -> Option<PathBuf> {
    let spec = cwd_ref(agent_id);
    let out = git
        .run_capture(&repo_git(workspace), &["cat-file", "blob", &spec])
        .ok()?;
    (!out.is_empty()).then(|| PathBuf::from(out))
}

/// Set the agent's working-directory mark to `dir` (an absolute path the
/// caller has already resolved and proven to be a directory). Writes the
/// path as a blob and points the mark at it — last write wins, exactly
/// as a `cd` should.
pub fn write(workspace: &Path, agent_id: &str, dir: &Path, git: &dyn GitRunner) -> io::Result<()> {
    storable(dir)?;
    let repo = repo_git(workspace);
    // `git hash-object` reads a file; the trait's two methods carry no
    // stdin, so the value is staged beside the repo under a pid-unique
    // name and removed once hashed. It is never inside a worktree, so
    // no `git add -A` can see it (§3.3 commit-per-side-effect).
    let staged = repo.join(format!("cwd-mark.{}.tmp", std::process::id()));
    std::fs::write(&staged, dir.as_os_str().as_bytes())?;
    let staged_str = staged.to_string_lossy().into_owned();
    let hashed = git.run_capture(&repo, &["hash-object", "-w", "--", &staged_str]);
    std::fs::remove_file(&staged)?;
    git.run(&repo, &["update-ref", &cwd_ref(agent_id), &hashed?])
}

/// Can `dir` survive the mark's storage round trip — written as bytes,
/// read back as trimmed UTF-8? A non-UTF-8 path or one with leading or
/// trailing whitespace cannot, and is declined here rather than stored
/// to read back as some other directory.
fn storable(dir: &Path) -> io::Result<()> {
    let text = dir.to_str().filter(|s| !s.is_empty() && s.trim() == *s);
    match text {
        Some(_) => Ok(()),
        None => Err(io::Error::other(format!(
            "cannot store {dir:?} as a working directory: the mark holds trimmed UTF-8 \
             text, so a path that is not UTF-8 or that leads or trails with whitespace \
             would read back as a different directory (ARCH §3.3)"
        ))),
    }
}

#[cfg(test)]
mod tests;
