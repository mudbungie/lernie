//! Resolution / spawn / disk-record I/O failure modes — every
//! [`super::super::ExecError`] variant gets a constructive test.

use super::super::spawn::which_in_path_env;
use super::super::{ExecError, SpawnTool, ToolCall, ToolExecutor};
use super::fixtures::{FixedClock, HarnessRoot, StepDir, driver_target, write_script};
use serde_json::json;
use std::sync::atomic::AtomicBool;
use tempfile::TempDir;

#[test]
fn spawn_error_when_resolved_binary_is_not_executable() {
    // Resolution succeeds (we drop a *file* under `tools/`) but
    // `Command::spawn` rejects it because it is not chmod +x.
    let root = HarnessRoot::new();
    let bin = root.dir.path().join(super::super::TOOLS_DIR).join(format!(
        "{}{}",
        super::super::EXTERNAL_PREFIX,
        "not-exec"
    ));
    std::fs::write(&bin, b"not a real binary").unwrap();
    let clock = FixedClock::default();
    let step = StepDir::new();
    let exec = SpawnTool::new(root.path(), &clock, driver_target());
    let err = exec
        .execute(
            ToolCall {
                id: "tu_e1",
                name: "not-exec",
                input: &json!({}),
            },
            &step.path,
            &AtomicBool::new(false),
        )
        .unwrap_err();
    match err {
        ExecError::Spawn { name, .. } => assert_eq!(name, "not-exec"),
        other => panic!("expected Spawn, got {other:?}"),
    }
}

#[test]
fn io_error_when_step_dir_is_a_file() {
    // `create_dir_all` refuses when the leaf is an existing file —
    // exercise the [`ExecError::Io`] branch.
    let root = HarnessRoot::new();
    root.install("anything", "true");
    let scratch = TempDir::new().unwrap();
    let bogus_step = scratch.path().join("not-a-dir");
    std::fs::write(&bogus_step, b"i am a file").unwrap();
    let clock = FixedClock::default();
    let exec = SpawnTool::new(root.path(), &clock, driver_target());
    let err = exec
        .execute(
            ToolCall {
                id: "tu_e2",
                name: "anything",
                input: &json!({}),
            },
            &bogus_step,
            &AtomicBool::new(false),
        )
        .unwrap_err();
    match err {
        ExecError::Io { dir, .. } => {
            assert!(dir.ends_with("tu_e2"), "wrong dir in error: {:?}", dir);
        }
        other => panic!("expected Io, got {other:?}"),
    }
}

#[test]
fn which_in_path_misses_when_no_dir_carries_the_binary() {
    let empty = TempDir::new().unwrap();
    assert_eq!(
        which_in_path_env("lernie-tool-nope", Some(empty.path().as_os_str())),
        None
    );
}

#[test]
fn which_in_path_live_env_returns_a_value_for_a_real_binary() {
    // Cover the live `which_in_path` (env-var-reading) wrapper. `sh`
    // is on PATH on every POSIX runner. We're not asserting where —
    // just that the env-read branch produces *something*.
    use super::super::spawn::which_in_path_env as wpe;
    let path = std::env::var_os("PATH");
    let hit = wpe("sh", path.as_deref());
    assert!(hit.is_some(), "expected /bin/sh or similar on PATH");
}

#[test]
fn live_which_in_path_reads_path_env_without_panicking() {
    // Covers the `var_os("PATH")` line in `which_in_path`. The
    // result is `Option` either way — under cargo test PATH is
    // typically set, but the wrapper must tolerate it being unset
    // (the `?` short-circuits) without us asserting a specific
    // outcome.
    let _ = super::super::spawn::which_in_path("lernie-tool-definitely-not-installed");
}

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
    use std::time::{Duration, Instant};
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("hi.sh");
    write_script(&path, "echo hi");
    let deadline = Instant::now() + Duration::from_millis(200);
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
    // blocking concurrent `exec` with ETXTBSY. The retry budget in
    // `subprocess::spawn_with_etxtbsy_retry` is generous enough to
    // outlive that hold; the tool exec eventually succeeds.
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
    let exec = SpawnTool::new(root.path(), &clock, driver_target());
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
        .expect("retry budget covers the ~40ms hold");
    holder.join().unwrap();
    assert!(!outcome.is_error);
    assert_eq!(outcome.content, b"unblocked\n");
}

#[test]
fn spawn_surfaces_etxtbsy_after_budget_exhausted() {
    // Same setup as the retry-success test, but the holder keeps the
    // fd open longer than the retry budget. The executor surfaces
    // `ExecError::Spawn` once retries time out.
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
    let exec = SpawnTool::new(root.path(), &clock, driver_target());
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
