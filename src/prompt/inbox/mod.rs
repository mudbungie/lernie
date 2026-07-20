//! The inbox substrate (ARCH §2.11 *Messages*).
//!
//! A **message** is content addressed to an existing agent, deposited
//! into the recipient's inbox and delivered at its next step boundary.
//! This module lands the deposit half of the channel: the executor lock
//! ([`lock`]), the create-only deposit ([`deposit`]), and the
//! deposit-starts-a-driver probe ([`probe_and_launch`]) behind the
//! `lernie message` verb ([`cli_message`]). The delivery drain that moves
//! these files into the transcript lives with the executor's step loop
//! (bl-1129, [`crate::prompt::dispatch`] — a driver, not a writer). The
//! workspace-wide sweep-and-flush behind the **operator verb**
//! `lernie scan` — crash-rate compensation, never wired into any driver
//! hot path (§2.11) — is [`scan`] (bl-d148, bl-5846); it, the
//! result-message return path (bl-4ce8), and the §2.11 exit protocol's
//! self-directed launch (bl-5846) ride this same substrate.
//!
//! **Writer/driver totality (§2.11).** `lernie message` is a *writer*:
//! it deposits and, if it observes the recipient quiescent (the lock
//! probe succeeds), *launches* a driver and exits — launching is not
//! driving, so the probe lease is released the instant it is taken and
//! never held to step. A driver that loses the acquire exits as a clean
//! no-op. Because no verb combines the two arms, the losing path is the
//! same code as the uncontended one.

pub mod baton;
pub mod deposit;
pub mod lock;
pub mod scan;

#[cfg(test)]
mod tests;

pub use deposit::{DepositError, Epitaph, deposit, deposit_result};
pub use lock::{ExecutorLock, try_acquire};
pub use scan::scan;

use crate::prompt::{Clock, SystemClock};
use std::ffi::OsStr;
use std::io;
use std::path::{Path, PathBuf};

/// Workspace-root directory holding every agent's inbox, namespaced by
/// agent id exactly like `steps/` (§2.2, §2.11). Outside every worktree.
pub const INBOX_DIR: &str = "inbox";

/// The reserved sender token for a deposit made by the user rather than
/// by an agent (§2.11 — `<sender>` is an agent id or `user`).
pub const USER_SENDER: &str = "user";

/// The per-agent inbox directory `<workspace>/inbox/<agent-id>/` — the
/// deposit target and the executor lock's home (§2.11).
pub fn inbox_dir(workspace: &Path, agent_id: &str) -> PathBuf {
    workspace.join(INBOX_DIR).join(agent_id)
}

/// The parent agent's id — `agent_id` minus its last descent segment
/// (§2.11 "the parent's address is the agent's own id minus its last
/// descent segment") — or `None` when `agent_id` is a root (it has no
/// parent). An agent id is a hyphenated descent of `<ts>-<short>`
/// segments (§2.3), and both the compact timestamp and the short id are
/// hyphen-free (`clock.rs`), so each segment is exactly two
/// hyphen-delimited tokens: a root is two tokens, and stripping the last
/// segment removes the trailing two. This is the same token arithmetic
/// [`crate::prompt::budget::derive::depth`] already relies on.
pub fn parent_of(agent_id: &str) -> Option<String> {
    let tokens: Vec<&str> = agent_id.split('-').collect();
    if tokens.len() <= 2 {
        return None;
    }
    Some(tokens[..tokens.len() - 2].join("-"))
}

/// Deposit a child's **result message** (§2.6) on its own behalf, into
/// its parent's inbox — the total return step (§2.3 step 5). A no-op
/// returning `Ok(None)` when `agent_id` is a root ([`parent_of`] is
/// `None`): a root has no parent inbox, its terminal response answers
/// the user instead (§2.4). Otherwise deposits and returns the created
/// path. The deposit is executor-side, never a model tool call ("Return
/// is not a verb", `docs/PRINCIPLES.md`).
pub fn deposit_child_result(
    workspace: &Path,
    agent_id: &str,
    epitaph: Epitaph,
    terminal_ref: &str,
    terminal_response: Option<&str>,
    clock: &dyn Clock,
) -> Result<Option<PathBuf>, DepositError> {
    match parent_of(agent_id) {
        None => Ok(None),
        Some(parent) => deposit_result(
            workspace,
            &parent,
            agent_id,
            epitaph,
            terminal_ref,
            terminal_response,
            clock,
        )
        .map(Some),
    }
}

/// Launches a driver for a quiescent agent — the one launch seam shared
/// by the writer's post-deposit probe, the `lernie scan` flush, and the
/// exit protocol's self-directed launch (§2.11). Kept as a trait so
/// every launch decision is testable with the spawn injected, and so the
/// production launch target can change without touching the callers. No
/// launcher ever decides whether launching is warranted; warrant is
/// decided by the launched driver under the lock (§2.11).
pub trait Launcher {
    /// Start a driver for `agent_id` under `workspace`. Called only
    /// with no lease held by the caller — the probe released its lease,
    /// the exiting executor released its lock — so the launched driver
    /// competes for the acquire like any other (§2.11 Writer/driver
    /// totality). Fire-and-forget: the caller never watches the driver.
    fn launch(&self, workspace: &Path, agent_id: &str) -> io::Result<()>;
}

/// The production launcher: detach-spawns `lernie advance <workspace>
/// <agent>` (§6), the workflow-chain driver that takes the lease,
/// rematerializes the worktree, drains the inbox, and steps (its
/// own-branch entry is [`crate::prompt::dispatch::driver::drive`]).
///
/// The spawn is **detached per §2.11**: `setsid` in the child (its own
/// session and process group — a §2.9 stop cascade against the launching
/// process never reaches the driver, and the driver outlives a launcher
/// running inside another agent's tool subprocess or a user's script),
/// stdio bound to null (the driver's observable state is its on-disk
/// step records and commits, §3.1), and [`baton::LOCK_FD_ENV`] scrubbed
/// (a launched driver *acquires*; only an exec'd successor adopts, §6).
/// Fire-and-forget: the child is never waited on — a launcher is
/// short-lived by design, and the unreaped driver reparents to init when
/// the launcher exits.
#[derive(Debug)]
pub struct AdvanceLauncher {
    exe: PathBuf,
}

impl AdvanceLauncher {
    /// Explicit binary path — the driver target the binding injects
    /// (`cmd::Fx::driver_target`, ARCH §2.11/§3.4) and every test picks.
    /// The library resolves no running-binary path of its own: the one
    /// `current_exe` for the launch/successor family lives at the exec
    /// binding (`src/bin/`), threaded down as this argument.
    pub fn with_exe(exe: PathBuf) -> Self {
        Self { exe }
    }
}

impl Launcher for AdvanceLauncher {
    fn launch(&self, workspace: &Path, agent_id: &str) -> io::Result<()> {
        let mut cmd = std::process::Command::new(&self.exe);
        cmd.arg("advance")
            .arg(workspace)
            .arg(agent_id)
            .env_remove(baton::LOCK_FD_ENV)
            .stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null());
        // SAFETY: `setsid()` is async-signal-safe and is the only call
        // between fork and exec; failure (already a session leader —
        // impossible post-fork) is ignored, the spawn proceeds grouped.
        unsafe {
            use std::os::unix::process::CommandExt;
            cmd.pre_exec(|| {
                // SAFETY: see above.
                libc::setsid();
                Ok(()) // LCOV_EXCL_LINE
            });
        }
        cmd.spawn()?;
        Ok(())
    }
}

/// Outcome of the post-deposit probe (§2.11 *A deposit into a quiescent
/// agent starts a driver*).
#[derive(Debug, PartialEq, Eq)]
pub enum ProbeOutcome {
    /// The branch was quiescent; a driver was launched.
    Launched,
    /// Another executor holds the lock; it will drain at its next step
    /// boundary. Nothing to launch.
    Busy,
}

/// Probe the executor lock for `agent_id` and, finding it quiescent,
/// release the probe and launch a driver (§2.11). A non-blocking
/// try-acquire whose *success* means nobody is driving: on success the
/// lease is dropped immediately — launching is not driving — before the
/// driver is launched, so the driver can win the acquire.
pub fn probe_and_launch(
    workspace: &Path,
    agent_id: &str,
    launcher: &dyn Launcher,
) -> io::Result<ProbeOutcome> {
    let dir = inbox_dir(workspace, agent_id);
    match lock::try_acquire(&dir)? {
        Some(guard) => {
            // Release the probe *before* launching so the driver's own
            // acquire is not blocked by our lease (§2.11).
            drop(guard);
            launcher.launch(workspace, agent_id)?;
            Ok(ProbeOutcome::Launched)
        }
        None => Ok(ProbeOutcome::Busy),
    }
}

/// Every way [`cli_message`] can fail.
#[derive(Debug, thiserror::Error)]
pub enum MessageError {
    #[error(transparent)]
    Deposit(#[from] DepositError),
    #[error(transparent)]
    Layout(#[from] crate::workspace::LayoutError),
    #[error("probe executor lock: {0}")]
    Probe(#[source] io::Error),
}

/// The `lernie message <workspace> <agent> <content>` verb (§2.11,
/// §3.4): deposit, then probe-and-launch. `sender` is resolved by the
/// caller — [`resolve_cli_sender`] for the bin, the calling agent's id
/// for the `message` tool — never from model input. Returns the probe
/// outcome so the caller can report whether a driver was launched.
pub fn cli_message(
    workspace: &Path,
    agent_id: &str,
    content: &str,
    sender: &str,
    clock: &dyn Clock,
    launcher: &dyn Launcher,
) -> Result<ProbeOutcome, MessageError> {
    deposit(workspace, agent_id, sender, content, clock)?;
    probe_and_launch(workspace, agent_id, launcher).map_err(MessageError::Probe)
}

/// CLI entry for `lernie message <workspace> <agent> <content>` (§3.4).
/// Kept in the lib so the bin stays under the 300-line cap and the wiring
/// is unit-testable — the same discipline as `stop::cli_run`. Resolves
/// the sender from the live `LERNIE_CONV_BRANCH` ([`resolve_cli_sender`])
/// and wires the production clock plus the real [`AdvanceLauncher`]
/// detached spawn (§2.11) at `driver_target` — the running-binary path
/// the exec binding injects (`cmd::Fx::driver_target`, §3.4), never
/// resolved in the library.
pub fn cli_run(
    workspace: &Path,
    agent: &str,
    content: &str,
    driver_target: &Path,
) -> Result<(), MessageError> {
    crate::workspace::require(workspace)?;
    let sender =
        resolve_cli_sender(std::env::var_os(crate::prompt::tool::ENV_CONV_BRANCH).as_deref());
    let launcher = AdvanceLauncher::with_exe(driver_target.to_path_buf());
    cli_message(workspace, agent, content, &sender, &SystemClock, &launcher)?;
    Ok(())
}

/// Resolve the deposit sender for a direct `lernie message` invocation
/// from the `LERNIE_CONV_BRANCH` value (§3.3): the calling agent's id
/// when the harness set it (an agent's `message` tool re-entering the
/// verb), else [`USER_SENDER`] for a bare user/frontend invocation. An
/// unset *or empty* value is `user`, mirroring the `LERNIE_HOME`
/// set-and-non-empty discipline (§2.2).
pub fn resolve_cli_sender(branch_env: Option<&OsStr>) -> String {
    match branch_env {
        Some(v) if !v.is_empty() => v.to_string_lossy().into_owned(),
        _ => USER_SENDER.to_string(),
    }
}
