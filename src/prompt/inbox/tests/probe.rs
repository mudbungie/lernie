//! Probe / launch / CLI-orchestration tests (ARCH §2.11 *A deposit into
//! a quiescent agent starts a driver*, Writer/driver totality).

use super::super::{
    AdvanceLauncher, Launcher, MessageError, ProbeOutcome, USER_SENDER, cli_message, cli_run,
    inbox_dir, probe_and_launch, resolve_cli_sender, try_acquire,
};
use crate::prompt::Clock;
use std::cell::RefCell;
use std::ffi::OsStr;
use std::io;
use std::path::Path;
use tempfile::TempDir;

struct FixedClock;
impl Clock for FixedClock {
    fn now_iso8601(&self) -> String {
        "2026-07-11T00:00:00Z".into()
    }
    fn now_compact(&self) -> String {
        unreachable!("deposit never reads the compact clock")
    }
}

/// Recording [`Launcher`] — captures each launch request.
#[derive(Default)]
struct StubLauncher {
    calls: RefCell<Vec<String>>,
}
impl Launcher for StubLauncher {
    fn launch(&self, _workspace: &Path, agent_id: &str) -> io::Result<()> {
        self.calls.borrow_mut().push(agent_id.to_string());
        Ok(())
    }
}

/// [`Launcher`] that fails to spawn — exercises the propagated error.
struct FailLauncher;
impl Launcher for FailLauncher {
    fn launch(&self, _workspace: &Path, _agent_id: &str) -> io::Result<()> {
        Err(io::Error::other("cannot spawn driver"))
    }
}

#[test]
fn probe_launches_a_driver_when_quiescent() {
    let ws = TempDir::new().unwrap();
    let launcher = StubLauncher::default();
    let out = probe_and_launch(ws.path(), "a1", &launcher).unwrap();
    assert_eq!(out, ProbeOutcome::Launched);
    assert_eq!(*launcher.calls.borrow(), vec!["a1".to_string()]);
}

#[test]
fn probe_is_busy_when_an_executor_holds_the_lock() {
    let ws = TempDir::new().unwrap();
    // Simulate a live executor by holding the lock across the probe.
    let _held = try_acquire(&inbox_dir(ws.path(), "a1"))
        .unwrap()
        .expect("free");
    let launcher = StubLauncher::default();
    let out = probe_and_launch(ws.path(), "a1", &launcher).unwrap();
    assert_eq!(out, ProbeOutcome::Busy);
    assert!(launcher.calls.borrow().is_empty(), "no launch while driven");
}

#[test]
fn probe_surfaces_try_acquire_error() {
    let ws = TempDir::new().unwrap();
    std::fs::write(ws.path().join("inbox"), b"not a dir").unwrap();
    let launcher = StubLauncher::default();
    assert!(probe_and_launch(ws.path(), "a1", &launcher).is_err());
}

#[test]
fn probe_propagates_launcher_error() {
    let ws = TempDir::new().unwrap();
    let err = probe_and_launch(ws.path(), "a1", &FailLauncher).unwrap_err();
    assert_eq!(err.to_string(), "cannot spawn driver");
}

#[test]
fn cli_message_deposits_then_launches() {
    let ws = TempDir::new().unwrap();
    let launcher = StubLauncher::default();
    let out = cli_message(ws.path(), "a1", "hello", "user", &FixedClock, &launcher).unwrap();
    assert_eq!(out, ProbeOutcome::Launched);
    assert!(inbox_dir(ws.path(), "a1").join("user-001.md").exists());
    assert_eq!(*launcher.calls.borrow(), vec!["a1".to_string()]);
}

#[test]
fn cli_message_surfaces_deposit_error() {
    let ws = TempDir::new().unwrap();
    std::fs::write(ws.path().join("inbox"), b"not a dir").unwrap();
    let err = cli_message(
        ws.path(),
        "a1",
        "hi",
        "user",
        &FixedClock,
        &StubLauncher::default(),
    )
    .unwrap_err();
    assert!(matches!(err, MessageError::Deposit(_)), "{err}");
}

#[test]
fn cli_message_surfaces_probe_error() {
    // Deposit succeeds; the launcher fails → MessageError::Probe.
    let ws = TempDir::new().unwrap();
    let err = cli_message(ws.path(), "a1", "hi", "user", &FixedClock, &FailLauncher).unwrap_err();
    assert!(matches!(err, MessageError::Probe(_)), "{err}");
    // The deposit still landed — undelivered, not lost (§2.11).
    assert!(inbox_dir(ws.path(), "a1").join("user-001.md").exists());
}

#[test]
fn resolve_cli_sender_defaults_to_user() {
    assert_eq!(resolve_cli_sender(None), USER_SENDER);
    assert_eq!(resolve_cli_sender(Some(OsStr::new(""))), USER_SENDER);
}

#[test]
fn resolve_cli_sender_uses_branch_when_set() {
    assert_eq!(resolve_cli_sender(Some(OsStr::new("p1-child"))), "p1-child");
}

#[test]
fn cli_run_deposits_via_production_deps() {
    // Exercises the production wiring: env-derived sender, SystemClock,
    // AdvanceLauncher (no-op). Whatever `LERNIE_CONV_BRANCH` is in the
    // test env, a single message file must land.
    let ws = TempDir::new().unwrap();
    cli_run(ws.path(), "a1", "hi").unwrap();
    let files: Vec<_> = std::fs::read_dir(inbox_dir(ws.path(), "a1"))
        .unwrap()
        .flatten()
        .filter(|e| e.file_name().to_string_lossy().ends_with(".md"))
        .collect();
    assert_eq!(files.len(), 1, "exactly one deposit landed");
}

#[test]
fn advance_launcher_is_a_noop_until_advance_exists() {
    // The production launcher is a documented stub pending `lernie
    // advance` (§6); it must succeed so the deposit path is not blocked.
    AdvanceLauncher.launch(Path::new("/tmp"), "a1").unwrap();
}
