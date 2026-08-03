//! The `ETXTBSY` retry envelope around a tool spawn (ARCH §3.3).
//!
//! "Text file busy" is the kernel refusing to exec a binary some
//! sibling process still holds open for write — a race the harness
//! rides out rather than surfacing, since the window is a fork/exec
//! transition and not a real spawn failure. Both arms of that loop are
//! exercised here, plus the fixture's own exec.
//!
//! Every budget in this file is a **count of attempts** (README's
//! determinism rule, bl-edf6), injected and sized so the arm under test
//! is the one that structurally runs. A test that pits a fixture hold
//! measured on one clock against a budget measured on another reports
//! machine load, not code: under parallel-agent tarpaulin runs the
//! loser is whoever's close gate happens to be running (bl-1c2e,
//! bl-7a3f).

use super::super::{ExecError, SpawnTool, ToolCall, ToolExecutor};
use super::fixtures::{
    FixedClock, HarnessRoot, StepDir, after_header, driver_target, write_script,
};
use serde_json::json;
use std::sync::atomic::AtomicBool;
use tempfile::TempDir;

#[test]
fn write_script_helper_is_round_tripped_by_the_fixture() {
    // Sanity: `fixtures::write_script` produces a runnable script.
    // Without this the resolution / cascade tests would fail in a
    // confusing way.
    //
    // The exec is wrapped in the same ETXTBSY-retry envelope the
    // production spawner (`subprocess::spawn_with_etxtbsy_retry`) uses,
    // because cargo runs tests in parallel and a sibling thread's
    // fork can briefly inherit this thread's not-yet-CLOEXEC write
    // fd to the script — same race the production retry was added
    // to mask.
    // The envelope is far larger than the shipped budget for the same
    // reason the tests below inject theirs (bl-7a3f): nothing here
    // asserts a duration, so a production-sized budget would turn a
    // sibling fork under load into a spurious `panic!`. And it is an
    // attempt count, not a deadline (README's determinism rule):
    // machine load cannot spend attempts, only ETXTBSY readings can.
    use std::time::Duration;
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("hi.sh");
    write_script(&path, "echo hi");
    let mut attempt: u32 = 1;
    let out = loop {
        match std::process::Command::new(&path).output() {
            Ok(o) => break o,
            Err(e) if e.raw_os_error() == Some(libc::ETXTBSY) && attempt < 10_000 => {
                attempt += 1;
                std::thread::sleep(Duration::from_millis(2));
            }
            Err(e) => panic!("exec failed: {e}"),
        }
    };
    assert!(out.status.success());
    assert_eq!(out.stdout, b"hi\n");
}

#[test]
fn spawn_retries_past_transient_etxtbsy() {
    // A sibling thread holds the binary's write fd open briefly,
    // blocking concurrent `exec` with ETXTBSY. The retry rides out that
    // hold and the tool exec succeeds.
    //
    // The budget is injected, and injected enormous, on purpose: the
    // shipped envelope against a 40 ms hold is two clocks racing, and a
    // loaded machine stretching the holder past the budget turned this
    // into a spurious close-gate failure for whoever was committing
    // (bl-7a3f). A million attempts, each with a 2 ms sleep floor, is a
    // budget no plausible hold can outlast — and being a count, load
    // stretches it rather than spending it (bl-edf6).
    use std::fs::OpenOptions;
    use std::sync::{Arc, Barrier};
    use std::thread;
    use std::time::Duration;

    let root = HarnessRoot::new();
    let installed = root.install("blocked", "echo unblocked");
    let barrier = Arc::new(Barrier::new(2));
    let path_clone = installed.clone();
    let b2 = barrier.clone();
    let holder = thread::spawn(move || {
        let f = OpenOptions::new().write(true).open(&path_clone).unwrap();
        b2.wait();
        thread::sleep(Duration::from_millis(40));
        drop(f);
    });
    barrier.wait();

    let clock = FixedClock::default();
    let step = StepDir::new();
    let exec = SpawnTool::new(root.path(), &clock, driver_target()).with_etxtbsy_budget(1_000_000);
    let outcome = exec
        .execute(
            ToolCall {
                id: "tu_etx",
                name: "blocked",
                input: &json!({}),
            },
            &step.path,
            &AtomicBool::new(false),
            None,
        )
        .expect("a million-attempt budget rides out the hold on any machine");
    holder.join().unwrap();
    assert!(!outcome.is_error);
    assert_eq!(after_header(&outcome.content), b"unblocked\n");
}

#[test]
fn spawn_surfaces_etxtbsy_after_budget_exhausted() {
    // Same setup as the retry-success test, mirrored: the holder keeps
    // the fd open until the executor has already given up. The
    // executor surfaces `ExecError::Spawn` once the attempt budget is
    // exhausted.
    //
    // This is also the test that covers the retry *arm* — the sleep
    // between attempts — and with a count budget that cover is
    // structural, not probabilistic: the hold is permanent, so every
    // attempt is guaranteed to see ETXTBSY, and a budget of 3 attempts
    // runs the retry arm exactly twice before the third attempt gives
    // up — the same iteration count on every run, whatever the machine
    // is doing (bl-edf6). Its sibling holds the fd for a fixed 40 ms
    // and so only meets ETXTBSY when the spawn lands inside that
    // window: fine for asserting the outcome, useless as the sole
    // cover for a line (bl-1c2e).
    use std::fs::OpenOptions;
    use std::sync::{Arc, Barrier};
    use std::thread;
    use std::time::Duration;

    let root = HarnessRoot::new();
    let installed = root.install("forever-busy", "echo will-not-run");
    let barrier = Arc::new(Barrier::new(2));
    let stop_holder = Arc::new(AtomicBool::new(false));
    let path_clone = installed.clone();
    let b2 = barrier.clone();
    let stop_clone = stop_holder.clone();
    let holder = thread::spawn(move || {
        let f = OpenOptions::new().write(true).open(&path_clone).unwrap();
        b2.wait();
        while !stop_clone.load(std::sync::atomic::Ordering::SeqCst) {
            thread::sleep(Duration::from_millis(5));
        }
        drop(f);
    });
    barrier.wait();

    let clock = FixedClock::default();
    let step = StepDir::new();
    let exec = SpawnTool::new(root.path(), &clock, driver_target()).with_etxtbsy_budget(3);
    let err = exec
        .execute(
            ToolCall {
                id: "tu_etx_x",
                name: "forever-busy",
                input: &json!({}),
            },
            &step.path,
            &AtomicBool::new(false),
            None,
        )
        .expect_err("the attempt budget exhausts against a permanent hold");
    stop_holder.store(true, std::sync::atomic::Ordering::SeqCst);
    holder.join().unwrap();
    match err {
        ExecError::Spawn { name, .. } => assert_eq!(name, "forever-busy"),
        other => panic!("expected Spawn, got {other:?}"),
    }
}
