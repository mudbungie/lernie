//! The workspace scan behind the operator verb `lernie scan` (ARCH
//! §2.11 *Crashes are a failure class*, §8).
//!
//! The scan compensates for crash-rate events, so it runs at operator
//! frequency — by hand, or by cron if an operator wants a heartbeat —
//! and is **never wired into any driver's hot path or default schedule**
//! (§2.11): normal operation is carried entirely by the deposit → probe →
//! launch channel plus the step-boundary drain, and the graceful-exit
//! crack is closed by the exit protocol (§2.11,
//! [`crate::prompt::dispatch`]), not by scanning. One pass, two derived
//! actions:
//!
//! 1. **Silent-death sweep** (§8): enumerate agent branches with *no live
//!    executor* (the [`try_acquire`] lock probe, released immediately)
//!    that either died mid-work (the latest step's `response.json` closed
//!    without a terminal `end`) or, for a child, never deposited a result
//!    message (no message from the child in the parent's inbox *and* none
//!    delivered in the parent's transcript — the sender-namespaced
//!    derivation, §2.11). For each hard-crashed **child** in that set,
//!    deposit the `died`-epitaph result message on the child's behalf
//!    ([`deposit_result`], sender = the child — the sweep is the scribe,
//!    not the author, §8).
//! 2. **Inbox flush** (§2.11): list `inbox/*/`; every agent with pending
//!    files and a free lock gets a driver *launched* — never drained: the
//!    scanner moves no files and commits nothing, only the lock-holding
//!    executor delivers. An agent whose lock is held is left alone.
//!
//! The sweep runs first, so its own deposits are picked up by the flush
//! that follows in the same pass.
//!
//! **Namespace.** The candidate enumeration is the §8 seam, landed: the
//! `agents/*` ref namespace (§2.3), read through
//! [`crate::workspace::agent_ids`] — config branches are excluded
//! structurally by the prefix, never by subtracting a reserved name
//! (there is no `main`, §2.2).
//!
//! **Scope note.** A child does not yet run a step loop (`worker.rs` stops
//! at the dispatch commit), so a real "died child" cannot arise from a run
//! today; the derivation is exercised against constructed on-disk states.
//! The derivation logic ([`derive`]) is fully unit-tested with the launch
//! injected; this module is the sweep/flush orchestration over it.

mod derive;

use super::deposit::Epitaph;
use super::{
    AdvanceLauncher, Launcher, ProbeOutcome, deposit_result, inbox_dir, parent_of, probe_and_launch,
};
use crate::prompt::{Clock, SystemClock};
use crate::template::{GitRunner, RealGit};
use derive::{
    agent_branches, branch_tip, died_mid_work, has_pending, inbox_agents, is_driven, returned,
};
use std::io;
use std::path::PathBuf;

/// Every way the [`scan`] can fail. Enumeration and transcript reads go
/// through `git`; deposits and launches surface their own I/O.
#[derive(Debug, thiserror::Error)]
pub enum ScanError {
    #[error("git {op}: {source}")]
    Git {
        op: &'static str,
        #[source]
        source: io::Error,
    },
    #[error(transparent)]
    Layout(#[from] crate::workspace::LayoutError),
    #[error(transparent)]
    Deposit(#[from] super::DepositError),
    #[error("probe executor lock at {path}: {source}")]
    Probe {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
    #[error("flush (probe-and-launch) for {agent}: {source}")]
    Flush {
        agent: String,
        #[source]
        source: io::Error,
    },
    #[error("read inbox root {path}: {source}")]
    InboxRoot {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
}

/// What one [`scan`] did, for the §8 health metrics and for tests. All
/// three are derived on the fly — nothing is stored (PRINCIPLES SSOT).
/// `Display` is the operator-facing summary `lernie scan` prints.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct ScanReport {
    /// §8 silent-death count: candidate branches (no live executor that
    /// died mid-work or, for a child, never deposited).
    pub silent_deaths: usize,
    /// Child ids the sweep deposited a `died` result for, in enumeration
    /// order.
    pub swept: Vec<String>,
    /// Agent ids the flush launched a driver for, in sorted order.
    pub flushed: Vec<String>,
}

/// Run the workspace-wide scan under `workspace` (§2.11, §8): the
/// silent-death sweep, then the inbox flush. `git` reads the branch and
/// transcript state against the bare `<workspace>/repo.git` (§2.2);
/// `launcher` is the injected driver launcher (production is the
/// [`super::AdvanceLauncher`] detached `lernie advance` spawn, §2.11),
/// so the whole decision logic is testable with launches captured.
pub fn scan(
    workspace: &std::path::Path,
    git: &dyn GitRunner,
    clock: &dyn Clock,
    launcher: &dyn Launcher,
) -> Result<ScanReport, ScanError> {
    let mut report = ScanReport::default();
    sweep(workspace, git, clock, &mut report)?;
    flush(workspace, launcher, &mut report)?;
    Ok(report)
}

/// The `lernie scan <workspace>` entry (§2.11, §3.4): run [`scan`] with
/// the real deps (`git`, clock, and the [`AdvanceLauncher`] detached
/// `lernie advance` spawn) wired in. An operator verb is loud, not
/// best-effort: errors propagate to a non-zero exit rather than being
/// swallowed — the operator invoked the sweep and is owed its outcome.
/// Mirrors the [`super::cli_run`] production-wiring convenience for
/// `lernie message`. `driver_target` is the running-binary path the
/// exec binding injects (`cmd::Fx::driver_target`, §3.4) for the
/// detached `lernie advance` launches; the library resolves none itself.
pub fn cli_run(
    workspace: &std::path::Path,
    driver_target: &std::path::Path,
) -> Result<ScanReport, ScanError> {
    crate::workspace::require(workspace)?;
    let launcher = AdvanceLauncher::with_exe(driver_target.to_path_buf());
    scan(workspace, &RealGit::new(), &SystemClock, &launcher)
}

impl std::fmt::Display for ScanReport {
    /// One operator-facing line: the §8 health counts plus what this
    /// pass did about them.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "silent deaths: {}; died deposits swept: {}; drivers launched: {}",
            self.silent_deaths,
            self.swept.len(),
            self.flushed.len()
        )
    }
}

/// The silent-death sweep (§8): count every candidate and, for a child
/// that never deposited, deposit the `died` result on its behalf.
fn sweep(
    workspace: &std::path::Path,
    git: &dyn GitRunner,
    clock: &dyn Clock,
    report: &mut ScanReport,
) -> Result<(), ScanError> {
    for branch in agent_branches(workspace, git)? {
        // A live executor holds the branch's inbox lock — it is either
        // working (never a silent death) or will drain at its own next
        // boundary. The probe lease is released the instant it is taken.
        if is_driven(workspace, &branch)? {
            continue;
        }
        let died = died_mid_work(workspace, &branch);
        // "for a child, never deposited" — the deposit condition, and the
        // idempotence hinge: a prior sweep's own deposit is a message
        // *from the child*, so a re-scan sees it and does not re-deposit.
        let child_never = match parent_of(&branch) {
            Some(parent) => !returned(workspace, git, &parent, &branch)?,
            None => false,
        };
        if died || child_never {
            report.silent_deaths += 1;
        }
        if child_never {
            let parent = parent_of(&branch).expect("child_never implies a parent");
            let tip = branch_tip(workspace, git, &branch)?;
            deposit_result(
                workspace,
                &parent,
                &branch,
                Epitaph::Died,
                &tip,
                None,
                clock,
            )?;
            report.swept.push(branch);
        }
    }
    Ok(())
}

/// The inbox flush (§2.11): every agent with pending files and a free
/// lock gets a driver *launched*. The scanner moves nothing and commits
/// nothing — only the lock-holding executor delivers. An agent whose lock
/// is held is left alone. Enumerated in sorted order for determinism.
fn flush(
    workspace: &std::path::Path,
    launcher: &dyn Launcher,
    report: &mut ScanReport,
) -> Result<(), ScanError> {
    for agent in inbox_agents(workspace)? {
        if !has_pending(&inbox_dir(workspace, &agent)) {
            continue;
        }
        // Reuse the writer's own probe-and-launch seam (§2.11): a free
        // lock ⇒ launch a driver; a held lock ⇒ its executor drains at its
        // next boundary, so leave it alone (Writer/driver totality). The
        // scanner never holds the lock or moves a file — it only launches.
        match probe_and_launch(workspace, &agent, launcher).map_err(|source| ScanError::Flush {
            agent: agent.clone(),
            source,
        })? {
            ProbeOutcome::Launched => report.flushed.push(agent),
            ProbeOutcome::Busy => {}
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests;
