//! `lernie stop <repo> <branch>` — cascading SIGTERM per ARCH §2.9.
//!
//! Sends SIGTERM to the harness process group driving `<branch>` (and
//! any subagent harnesses spawned on descended branches per §3.4) with
//! a 5-second flush deadline before SIGKILL — the same signal cascade
//! §4.4 pins for adapters and §3.3 pins for tools, applied to the
//! harness itself.
//!
//! No on-disk cancel marker is written: per §2.9 the on-disk
//! signature of a stopped branch is the latest step's `response.json`
//! closed (`IN_CLOSE_WRITE`, §3.5) without a terminal brazen `end`
//! event. The kernel produces that signature for free when the
//! harness terminates without flushing — same way crashes and
//! external kills are indistinguishable on disk per §2.9.
//!
//! Pid discovery derives from `/proc/<pid>/fd/*` symlink targets
//! against the agent's **inbox directory** — the executor lock's
//! `flock` home (§2.11), held for the whole step loop. The target is
//! `<workspace>/inbox/<branch>/` (and any sibling `inbox/<branch>-*/`
//! for descended subagent conversations, hyphenated descent per §2.2).
//! No sidecar pid file: the open lock fd is the *is-anyone-driving*
//! signal the §2.11 lock probe and §3.5 classification already read —
//! and, unlike the `response.json` model-call fd, it is open across
//! tool execution and between-step gaps too, so a stop lands whenever
//! an executor is alive (§2.9).

use crate::prompt::inbox::INBOX_DIR;
use crate::template::{GitRunner, RealGit};
use std::io;
use std::path::{Path, PathBuf};
use std::time::Duration;
use thiserror::Error;

pub mod cascade;
pub mod discover;
pub mod inspector;

#[cfg(test)]
mod tests;

pub use cascade::{RealSignaler, Signaler, cascade};
pub use discover::{PgidFinder, ProcFsFinder};
pub use inspector::{BranchInspector, GitInspector};

/// SIGTERM-to-SIGKILL grace pinned by ARCH §2.9 (mirrors §4.4 / §3.3).
/// Tests pass a sub-second deadline; production uses this constant.
pub const STOP_DEADLINE: Duration = Duration::from_secs(5);

/// Polling cadence while waiting for SIGTERM'd processes to exit.
/// Small enough that user stop feels instant, large enough that an
/// idle wait costs nothing measurable.
const POLL_INTERVAL: Duration = Duration::from_millis(50);

/// Every way [`run`] can fail. Idempotent paths (no lock holder found,
/// already-stopped) are `Ok(())`, not errors — `lernie stop` is a
/// fire-and-forget operation, not a transactional one.
#[derive(Debug, Error)]
pub enum Error {
    #[error("branch {0:?} does not exist in repo")]
    BranchMissing(String),
    #[error("branch {0:?} is already merged into main")]
    AlreadyMerged(String),
    #[error("git {op}: {source}")]
    Git {
        op: &'static str,
        #[source]
        source: io::Error,
    },
    #[error("scan /proc: {0}")]
    Proc(#[source] io::Error),
    #[error("walk inbox directory: {0}")]
    InboxWalk(#[source] io::Error),
}

/// Stop the harness driving `branch` and any subagent descendants.
///
/// 1. Validate `branch` exists in `<repo>/root/.git` and is unmerged.
/// 2. Walk every agent-id namespace rooted at `branch`
///    (`inbox/<branch>/` and `inbox/<branch>-*/`) — each the home of
///    one executor lock (§2.11).
/// 3. Resolve each lock holder's pgid via the supplied [`PgidFinder`].
/// 4. SIGTERM the unique pgid set, wait `deadline`, SIGKILL leftovers.
///
/// Idempotent: a stopped branch (no lock holder found) returns `Ok(())`.
pub fn run(
    repo: &Path,
    branch: &str,
    inspector: &dyn BranchInspector,
    finder: &dyn PgidFinder,
    signaler: &dyn Signaler,
    deadline: Duration,
    git: &dyn GitRunner,
) -> Result<(), Error> {
    // The inspector takes the conv-repo path; it routes git inside
    // the primary worktree (`<repo>/root/`, ARCH §2.2) where `.git`
    // lives.
    if !inspector
        .exists(repo, branch, git)
        .map_err(|source| Error::Git {
            op: "rev-parse --verify",
            source,
        })?
    {
        return Err(Error::BranchMissing(branch.to_owned()));
    }
    if inspector
        .is_merged_into_main(repo, branch, git)
        .map_err(|source| Error::Git {
            op: "merge-base --is-ancestor",
            source,
        })?
    {
        return Err(Error::AlreadyMerged(branch.to_owned()));
    }

    let inbox_dirs = collect_inbox_dirs(repo, branch)?;
    let mut pgids = Vec::new();
    for dir in inbox_dirs {
        if let Some(pgid) = finder.find_holder_pgid(&dir).map_err(Error::Proc)? {
            pgids.push(pgid);
        }
    }
    pgids.sort();
    pgids.dedup();
    if pgids.is_empty() {
        return Ok(());
    }

    cascade(&pgids, signaler, deadline, POLL_INTERVAL);
    Ok(())
}

/// CLI entry point for `lernie stop` (ARCH §3.4 — kept in the lib so
/// the bin file stays under the 300-line code cap and the wiring
/// itself is unit-testable). Production builds use the default
/// deps; tests exercise [`run`] directly with stubs.
pub fn cli_run(repo: &Path, branch: &str) -> Result<(), Error> {
    run(
        repo,
        branch,
        &GitInspector,
        &ProcFsFinder::default(),
        &RealSignaler,
        STOP_DEADLINE,
        &RealGit::new(),
    )
}

/// Promote the calling process to a process-group leader so the
/// §2.9 cascade (`kill(-pgid, SIGTERM)`) reaches its provider
/// adapter and any subagent harnesses re-entered via `lernie
/// dispatch` without escaping into the invoking shell or UI's
/// process group. Called by `lernie prompt` at top-of-main; not
/// called by `lernie dispatch` (subagent re-entries deliberately
/// inherit the parent's pgid).
pub fn become_pgid_leader() {
    // SAFETY: setpgid is async-signal-safe; (0, 0) means "this
    // process; new group with itself as leader". Idempotent when
    // the process is already a pgid leader (typical when invoked
    // from a shell with job control). The branch-table is fully
    // exercised by `become_pgid_leader_with` (closure-injected
    // syscall); this wrapper itself is a one-liner.
    become_pgid_leader_with(|| unsafe { libc::setpgid(0, 0) });
}

/// Inner core for [`become_pgid_leader`]: parameterized on the
/// `setpgid` syscall so a unit test can exercise both branches
/// without mutating the test runner's pgid.
fn become_pgid_leader_with(setpgid: impl FnOnce() -> libc::c_int) {
    let r = setpgid();
    if r != 0 {
        eprintln!("lernie: setpgid: {}", io::Error::last_os_error());
    }
}

/// The inbox directory `inbox/<branch>/` and every `inbox/<branch>-*/`
/// (hyphenated descent per §2.2) that exists — each the home of one
/// executor lock (§2.11). The branch name itself is the agent id;
/// descended subagent conversations have ids that prefix-match the
/// parent's (`<conv>-<sub>`). Absent `inbox/` (an agent spawned but
/// whose executor has not yet opened a lock) yields an empty set — a
/// stop with nothing to signal, idempotently `Ok(())`.
fn collect_inbox_dirs(repo: &Path, branch: &str) -> Result<Vec<PathBuf>, Error> {
    let inbox_root = repo.join(INBOX_DIR);
    let mut dirs = Vec::new();
    if !inbox_root.exists() {
        return Ok(dirs);
    }
    let prefix_dash = format!("{branch}-");
    let entries = std::fs::read_dir(&inbox_root).map_err(Error::InboxWalk)?;
    for entry in entries {
        let entry = entry.map_err(Error::InboxWalk)?;
        let name = entry.file_name();
        let Some(name_str) = name.to_str() else {
            continue;
        };
        if name_str != branch && !name_str.starts_with(&prefix_dash) {
            continue;
        }
        dirs.push(entry.path());
    }
    Ok(dirs)
}
