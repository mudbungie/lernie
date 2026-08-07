//! [`ToolExecutor::execute_all`] (ARCH §3.3 *The multi-tool*,
//! `execution: "parallel"`): the spawning executor overlaps N calls,
//! returns one result per tool call in the order given, and lands every
//! per-tool-call record exactly as `execute` does.

use super::super::{
    OUTPUT_FILE, STEP_TOOLS_SUBDIR, SpawnTool, ToolCall, ToolExecutor, ToolOutputRecord,
};
use super::fixtures::{FixedClock, HarnessRoot, StepDir, driver_target};
use serde_json::json;
use std::sync::atomic::AtomicBool;

#[test]
fn execute_all_actually_overlaps_the_calls() {
    // A rendezvous, not a stopwatch: each tool drops its own marker in
    // the shared worktree and then waits for its partner's. Under real
    // overlap both find it and exit 0; run one-at-a-time the first
    // would spin out its budget and exit 1. The assertion is on the
    // outcome, never on elapsed time — machine load must not decide a
    // test (README's determinism rule).
    let root = HarnessRoot::new();
    for (me, them) in [("a", "b"), ("b", "a")] {
        root.install(
            &format!("rendezvous_{me}"),
            &format!(
                r#"
                touch mark_{me}
                for _ in $(seq 1 400); do
                    [ -f mark_{them} ] && exit 0
                    sleep 0.05
                done
                echo "never saw mark_{them}" >&2
                exit 1
                "#
            ),
        );
    }
    let clock = FixedClock::default();
    let step = StepDir::new();
    let exec = SpawnTool::new(root.path(), &clock, driver_target());
    let input = json!({});
    let results = exec.execute_all(
        &[
            ToolCall {
                id: "toolu_01-1",
                name: "rendezvous_a",
                input: &input,
            },
            ToolCall {
                id: "toolu_01-2",
                name: "rendezvous_b",
                input: &input,
            },
        ],
        &step.path,
        &AtomicBool::new(false),
        None,
    );
    assert_eq!(results.len(), 2);
    for (idx, result) in results.iter().enumerate() {
        let outcome = result.as_ref().expect("no harness fault");
        assert!(
            !outcome.is_error,
            "entry {idx} did not meet its partner: {}",
            String::from_utf8_lossy(&outcome.content)
        );
    }
}

#[test]
fn execute_all_returns_results_in_call_order_and_lands_every_record() {
    // Order is the caller's, whatever the scheduler did — and each
    // call still owns its own `tools/<id>/` record.
    let root = HarnessRoot::new();
    root.install("slow", r#"sleep 0.3; printf "slow""#);
    root.install("quick", r#"printf "quick""#);
    let clock = FixedClock::default();
    let step = StepDir::new();
    let exec = SpawnTool::new(root.path(), &clock, driver_target());
    let input = json!({});
    let results = exec.execute_all(
        &[
            ToolCall {
                id: "toolu_01-1",
                name: "slow",
                input: &input,
            },
            ToolCall {
                id: "toolu_01-2",
                name: "quick",
                input: &input,
            },
        ],
        &step.path,
        &AtomicBool::new(false),
        None,
    );
    // The slow call finished second but reports first.
    assert_eq!(results[0].as_ref().unwrap().content, b"Exit code: 0\nslow");
    assert_eq!(results[1].as_ref().unwrap().content, b"Exit code: 0\nquick");
    for (id, want) in [("toolu_01-1", "slow"), ("toolu_01-2", "quick")] {
        let dir = step.path.join(STEP_TOOLS_SUBDIR).join(id);
        let record: ToolOutputRecord =
            serde_json::from_slice(&std::fs::read(dir.join(OUTPUT_FILE)).unwrap()).unwrap();
        assert_eq!(record.stdout, want);
        assert_eq!(record.exit_code, 0);
        // One clock window for the whole fan: they did start together.
        assert_eq!(record.started_at, "iso-1");
        assert_eq!(record.ended_at, "iso-2");
    }
}

#[test]
fn a_prepare_failure_becomes_that_call_s_result() {
    // Preparation runs before any process starts, so a step dir with
    // no resolvable worktree fails every call in place rather than
    // spawning anything.
    let root = HarnessRoot::new();
    root.install("greet", r#"printf "hi""#);
    let clock = FixedClock::default();
    let nowhere = tempfile::TempDir::new().unwrap();
    let exec = SpawnTool::new(root.path(), &clock, driver_target());
    let input = json!({});
    let results = exec.execute_all(
        &[ToolCall {
            id: "toolu_01-1",
            name: "greet",
            input: &input,
        }],
        nowhere.path(),
        &AtomicBool::new(false),
        None,
    );
    assert_eq!(results.len(), 1);
    assert!(matches!(
        results[0],
        Err(super::super::ExecError::NoWorktree { .. })
    ));
}

#[test]
fn the_default_execute_all_is_serial() {
    // An executor that declines to overlap inherits the trait's own
    // implementation and is still correct — concurrency is an
    // optimization, not a semantic.
    struct Counting(std::cell::RefCell<Vec<String>>);
    impl ToolExecutor for Counting {
        fn execute(
            &self,
            call: ToolCall<'_>,
            _step_dir: &std::path::Path,
            _stop: &AtomicBool,
            _bound: Option<crate::config::ToolOutputBound>,
        ) -> Result<super::super::ToolOutcome, super::super::ExecError> {
            self.0.borrow_mut().push(call.name.to_string());
            Ok(super::super::ToolOutcome {
                content: call.name.as_bytes().to_vec(),
                is_error: false,
            })
        }
    }
    let exec = Counting(std::cell::RefCell::new(Vec::new()));
    let input = json!({});
    let results = exec.execute_all(
        &[
            ToolCall {
                id: "1",
                name: "alpha",
                input: &input,
            },
            ToolCall {
                id: "2",
                name: "beta",
                input: &input,
            },
        ],
        std::path::Path::new("/unused"),
        &AtomicBool::new(false),
        None,
    );
    assert_eq!(*exec.0.borrow(), vec!["alpha", "beta"]);
    assert_eq!(results.len(), 2);
    assert_eq!(results[0].as_ref().unwrap().content, b"alpha");
}
