//! SIGTERM-then-SIGKILL cascade (ARCH §3.3) and the "killed by some
//! other signal → harness-level fault per §2.10" branch.

use super::super::{ExecError, SpawnTool, ToolCall, ToolExecutor};
use super::fixtures::{FixedClock, HarnessRoot, StepDir, driver_target};
use serde_json::json;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::{Duration, Instant};

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
while true; do sleep 0.05; done
"#,
    );
    let clock = FixedClock::default();
    let step = StepDir::new();
    let exec =
        SpawnTool::new(root.path(), &clock, driver_target()).with_deadline(Duration::from_secs(2));
    let stop = AtomicBool::new(false);

    // Flip the stop flag from another thread shortly after spawn so the
    // executor's polling loop catches it.
    thread::scope(|s| {
        s.spawn(|| {
            thread::sleep(Duration::from_millis(150));
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
        // §3.3: stderr concatenated after stdout when exit non-zero;
        // here stderr is empty, content is just stdout.
        assert_eq!(outcome.content, b"bye\n");
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
while true; do sleep 0.05; done
"#,
    );
    let clock = FixedClock::default();
    let step = StepDir::new();
    let deadline = Duration::from_millis(250);
    let exec = SpawnTool::new(root.path(), &clock, driver_target()).with_deadline(deadline);
    let stop = AtomicBool::new(false);
    let started = Instant::now();
    thread::scope(|s| {
        s.spawn(|| {
            thread::sleep(Duration::from_millis(150));
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
