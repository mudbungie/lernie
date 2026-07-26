//! The `ETXTBSY` retry envelope around a tool spawn (ARCH §3.3).
//!
//! "Text file busy" is the kernel refusing to exec a binary some
//! sibling process still holds open for write — a race the harness
//! rides out rather than surfacing, since the window is a fork/exec
//! transition and not a real spawn failure. Both arms of that loop are
//! exercised here, plus the fixture's own exec.
//!
//! Every wait in this file is sized far past what it is waiting on, or
//! injected outright. A test that pits a fixture hold measured on one
//! clock against a budget measured on another reports machine load,
//! not code: under parallel-agent tarpaulin runs the loser is whoever's
//! close gate happens to be running (bl-1c2e, bl-7a3f).

use super::super::{ExecError, SpawnTool, ToolCall, ToolExecutor};
use super::fixtures::{FixedClock, HarnessRoot, StepDir, driver_target, write_script};
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
    // The envelope is far longer than the shipped budget for the same
    // reason the tests below inject theirs (bl-7a3f): nothing here
    // asserts a duration, so a production-sized wait would turn a
    // sibling fork under load into a spurious `panic!`.
    use std::time::{Duration, Instant};
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("hi.sh");
    write_script(&path, "echo hi");
    let deadline = Instant::now() + Duration::from_secs(30);
    let out = loop {
        match std::process::Command::new(&path).output() {
            Ok(o) => break o,
            Err(e) if e.raw_os_error() == Some(libc::ETXTBSY) && Instant::now() < deadline => {
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
    // shipped 200 ms against a 40 ms hold is two clocks racing, and a
    // loaded machine stretching the holder past the budget turned this
    // into a spurious close-gate failure for whoever was committing
    // (bl-7a3f). A budget no plausible hold can outlast makes the arm
    // under test the one that runs.
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
    let exec = SpawnTool::new(root.path(), &clock, driver_target())
        .with_etxtbsy_budget(Duration::from_secs(30));
    let outcome = exec
        .execute(
            ToolCall {
                id: "tu_etx",
                name: "blocked",
                input: &json!({}),
            },
            &step.path,
            &AtomicBool::new(false),
        )
        .expect("a 30s retry budget rides out the hold on any machine");
    holder.join().unwrap();
    assert!(!outcome.is_error);
    assert_eq!(outcome.content, b"unblocked\n");
}

#[test]
fn spawn_surfaces_etxtbsy_after_budget_exhausted() {
    // Same setup as the retry-success test, mirrored: the holder keeps
    // the fd open until the executor has already given up. The
    // executor surfaces `ExecError::Spawn` once retries time out.
    //
    // This is also the test that covers the retry *arm* — the sleep
    // between attempts — and it is the one that can do so without a
    // race. The hold here is permanent, so every attempt is guaranteed
    // to see ETXTBSY, and a budget of a whole second cannot expire
    // inside the straight-line microseconds between computing the
    // deadline and the first failed spawn. Its sibling holds the fd for
    // a fixed 40 ms and so only meets ETXTBSY when the spawn lands
    // inside that window: fine for asserting the outcome, useless as
    // the sole cover for a line (bl-1c2e).
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
    let exec = SpawnTool::new(root.path(), &clock, driver_target())
        .with_etxtbsy_budget(Duration::from_secs(1));
    let err = exec
        .execute(
            ToolCall {
                id: "tu_etx_x",
                name: "forever-busy",
                input: &json!({}),
            },
            &step.path,
            &AtomicBool::new(false),
        )
        .expect_err("retry budget should expire before the holder releases");
    stop_holder.store(true, std::sync::atomic::Ordering::SeqCst);
    holder.join().unwrap();
    match err {
        ExecError::Spawn { name, .. } => assert_eq!(name, "forever-busy"),
        other => panic!("expected Spawn, got {other:?}"),
    }
}
