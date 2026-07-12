//! The inbox substrate (ARCH §2.11 *Messages*).
//!
//! A **message** is content addressed to an existing agent, deposited
//! into the recipient's inbox and delivered at its next step boundary.
//! This module lands the deposit half of the channel: the executor lock
//! ([`lock`]), the create-only deposit ([`deposit`]), and the
//! deposit-starts-a-driver probe ([`probe_and_launch`]) behind the
//! `lernie message` verb ([`cli_message`]). The delivery drain
//! (bl-1129), the startup scan (bl-d148), and the result-message return
//! path (bl-4ce8) ride this substrate but are out of scope here — for
//! now deposited files simply accumulate in the inbox.
//!
//! **Writer/driver totality (§2.11).** `lernie message` is a *writer*:
//! it deposits and, if it observes the recipient quiescent (the lock
//! probe succeeds), *launches* a driver and exits — launching is not
//! driving, so the probe lease is released the instant it is taken and
//! never held to step. A driver that loses the acquire exits as a clean
//! no-op. Because no verb combines the two arms, the losing path is the
//! same code as the uncontended one.

pub mod deposit;
pub mod lock;

#[cfg(test)]
mod tests;

pub use deposit::{DepositError, deposit};
pub use lock::{ExecutorLock, try_acquire};

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

/// Launches a driver for a quiescent agent. Kept as a trait so the
/// deposit-starts-a-driver decision is testable with the launch injected
/// (§2.11), and so the production launch target can change without
/// touching the probe logic.
pub trait Launcher {
    /// Start a driver for `agent_id` under `workspace`. Called only
    /// after the probe found the branch quiescent and released its
    /// lease, so the launched driver competes for the acquire like any
    /// other (§2.11 Writer/driver totality).
    fn launch(&self, workspace: &Path, agent_id: &str) -> io::Result<()>;
}

/// The production launch target is `lernie advance <workspace> <agent>`
/// (§6), the workflow-chain driver that acquires the lock, rematerializes
/// the worktree, drains the inbox, and steps. That verb is not yet
/// implemented (specced in §6; tracked separately), so this launcher is
/// a deliberate no-op: the deposit has already landed and will be
/// delivered by the next driver that runs against the branch (§2.11
/// "Undelivered is derived"). The probe decision above it is live and
/// tested; only the spawn is stubbed pending `lernie advance`.
#[derive(Debug, Default)]
pub struct AdvanceLauncher;

impl Launcher for AdvanceLauncher {
    fn launch(&self, _workspace: &Path, _agent_id: &str) -> io::Result<()> {
        // No-op until `lernie advance` (§6) exists. See the type doc.
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
/// and wires the production clock plus the [`AdvanceLauncher`] stub.
pub fn cli_run(workspace: &Path, agent: &str, content: &str) -> Result<(), MessageError> {
    let sender =
        resolve_cli_sender(std::env::var_os(crate::prompt::tool::ENV_CONV_BRANCH).as_deref());
    cli_message(
        workspace,
        agent,
        content,
        &sender,
        &SystemClock,
        &AdvanceLauncher,
    )?;
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
