//! SIGTERM-then-SIGKILL cascade (ARCH §3.3) and the "killed by some
//! other signal → harness-level fault per §2.10" branch.

use super::super::{ExecError, SpawnTool, ToolCall, ToolExecutor};
use super::fixtures::{FixedClock, HarnessRoot, StepDir, driver_target};
use serde_json::json;
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::{Duration, Instant};

/// Attempt budget for [`await_marker`] — a count, never a deadline
/// (README's determinism rule). The marker is the fixture's observable
/// "my trap is installed" signal; the budget exists only to turn a tool
/// that never started into a panic instead of a hang.
const MARKER_RETRIES: u32 = 12_000;

/// Wait for the fixture to touch `marker` in its working directory.
/// Evidence-driven, not timed: the fixtures below touch the marker
/// *after* installing their TERM trap, so a stop flipped on this
/// evidence sends SIGTERM to a shell whose handler is provably in
/// place. (The old timed flip — sleep, then stop — raced the shell's
/// startup: under load the SIGTERM could land before `trap` ran and
/// the default disposition killed the fixture, failing the test on
/// machine load rather than on code.)
fn await_marker(marker: &Path) {
    for _ in 0..MARKER_RETRIES {
        if marker.exists() {
            return;
        }
        thread::sleep(Duration::from_millis(5));
    }
    panic!("fixture never touched {}", marker.display());
}

#[test]
fn sigterm_inside_deadline_lets_tool_exit_with_its_own_code() {
    // The fixture catches SIGTERM, prints a goodbye line, and exits 9.
    // The cascade must reap the real exit code rather than synthesize a
    // "killed by signal" status.
    let root = HarnessRoot::new();
    root.install(
        "well-behaved",
        r#"
trap 'echo bye; exit 9' TERM
touch ready
while true; do sleep 0.05; done
"#,
    );
    let clock = FixedClock::default();
    let step = StepDir::new();
    // The deadline never expires on this path (the tool exits within
    // one sleep cycle of the SIGTERM its trap provably handles); it is
    // injected enormous so no scheduling stretch can turn this into
    // the SIGKILL branch (bl-7a3f).
    let exec =
        SpawnTool::new(root.path(), &clock, driver_target()).with_deadline(Duration::from_secs(60));
    let stop = AtomicBool::new(false);
    let ready = step.worktree.join("ready");

    // Flip the stop flag once the tool is observably past its trap
    // install, so the executor's polling loop catches it.
    thread::scope(|s| {
        s.spawn(|| {
            await_marker(&ready);
            stop.store(true, Ordering::SeqCst);
        });
        let outcome = exec
            .execute(
                ToolCall {
                    id: "tu_c1",
                    name: "well-behaved",
                    input: &json!({}),
                },
                &step.path,
                &stop,
            )
            .unwrap();
        assert!(outcome.is_error, "exit 9 → is_error true");
        // §3.3 result envelope: the code the tool chose for itself is
        // stated, so a SIGTERM the tool caught and handled is legible as
        // exit 9 rather than as the cancel's own 143. Stderr is empty
        // here, so no marked block follows.
        assert_eq!(outcome.content, b"Exit code: 9\nbye\n");
    });
}

#[test]
fn sigkill_after_deadline_when_tool_ignores_sigterm() {
    // The fixture installs a SIGTERM handler that does nothing and
    // keeps spinning. After the deadline the executor must SIGKILL it
    // and surface a §2.10 fault.
    let root = HarnessRoot::new();
    root.install(
        "stubborn",
        r#"
trap 'echo ignored 1>&2' TERM
touch ready
while true; do sleep 0.05; done
"#,
    );
    let clock = FixedClock::default();
    let step = StepDir::new();
    // Sub-second so the SIGKILL branch is observable without a 5s
    // wait. Load-safe in this direction: the trap is provably
    // installed before the SIGTERM (marker evidence below), so no
    // amount of stretching changes which branch runs — expiry is the
    // only outcome against a handler that never exits.
    let deadline = Duration::from_millis(250);
    let exec = SpawnTool::new(root.path(), &clock, driver_target()).with_deadline(deadline);
    let stop = AtomicBool::new(false);
    let ready = step.worktree.join("ready");
    let started = Instant::now();
    thread::scope(|s| {
        s.spawn(|| {
            await_marker(&ready);
            stop.store(true, Ordering::SeqCst);
        });
        let err = exec
            .execute(
                ToolCall {
                    id: "tu_c2",
                    name: "stubborn",
                    input: &json!({}),
                },
                &step.path,
                &stop,
            )
            .unwrap_err();
        match err {
            ExecError::KilledBySignal { name, signal } => {
                assert_eq!(name, "stubborn");
                assert_eq!(signal, libc::SIGKILL);
            }
            other => panic!("expected KilledBySignal, got {other:?}"),
        }
    });
    // Sanity: the cascade did wait the deadline (give it slack for
    // scheduler jitter and the polling cadence).
    assert!(
        started.elapsed() >= deadline,
        "cascade returned before deadline elapsed"
    );
}

#[test]
fn unsolicited_signal_kill_is_reported_as_harness_fault() {
    // The fixture sends SIGSEGV to itself. Per ARCH §3.3 / §2.10,
    // termination by a signal *other than the harness's own SIGTERM*
    // is a harness-level fault, not a tool failure delivered to the
    // model.
    let root = HarnessRoot::new();
    root.install(
        "self-segv",
        r#"
echo about-to-segv
kill -SEGV $$
"#,
    );
    let clock = FixedClock::default();
    let step = StepDir::new();
    let exec = SpawnTool::new(root.path(), &clock, driver_target());
    let err = exec
        .execute(
            ToolCall {
                id: "tu_c3",
                name: "self-segv",
                input: &json!({}),
            },
            &step.path,
            &AtomicBool::new(false),
        )
        .unwrap_err();
    match err {
        ExecError::KilledBySignal { name, signal } => {
            assert_eq!(name, "self-segv");
            assert_eq!(signal, libc::SIGSEGV);
        }
        other => panic!("expected KilledBySignal(SIGSEGV), got {other:?}"),
    }
}
