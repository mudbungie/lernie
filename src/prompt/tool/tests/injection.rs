//! The host injection seam at the executor (ARCH §3.3 *Host-injected
//! tools*, `docs/DESIGN_TOOL_INJECTION.md`): a test embedder that
//! declares a tool of its own and routes it.
//!
//! What is asserted is the claim the design makes — that a routed tool
//! is indistinguishable from a spawned one *downstream*: the same result
//! envelope, the same `is_error`, the same bounded projection, the same
//! `input.json` / `output.json` pair under the same directory. The
//! difference is upstream and total: no binary is resolved and no
//! process is started.

use super::super::inject::{InjectedTool, RoutedCall, RoutedCapture, ToolInjection};
use super::super::{
    INPUT_FILE, OUTPUT_FILE, STEP_TOOLS_SUBDIR, SpawnTool, ToolCall, ToolExecutor, ToolInputRecord,
    ToolOutputRecord,
};
use super::fixtures::{FixedClock, HarnessRoot, StepDir, driver_target};
use serde_json::{Value, json};
use std::cell::RefCell;
use std::sync::atomic::AtomicBool;

/// A test embedder. It declares one tool, owns exactly the names it was
/// built with, and records what each routed invocation carried — so the
/// four wire facts a subprocess reads from stdin and its environment can
/// be asserted to reach a router unchanged.
struct Embedder {
    owns: &'static str,
    exit_code: i32,
    seen: RefCell<Vec<(String, String, Value, String)>>,
}

impl Embedder {
    fn new(owns: &'static str) -> Self {
        Self {
            owns,
            exit_code: 0,
            seen: RefCell::new(Vec::new()),
        }
    }

    fn failing(owns: &'static str) -> Self {
        Self {
            exit_code: 7,
            ..Self::new(owns)
        }
    }
}

impl ToolInjection for Embedder {
    fn tools(&self) -> Vec<InjectedTool> {
        vec![InjectedTool {
            name: self.owns.to_string(),
            input_schema: json!({"type": "object"}),
            description: Some("the host's own tool".into()),
        }]
    }

    fn route(&self, call: RoutedCall<'_>) -> Option<RoutedCapture> {
        if call.name != self.owns {
            return None;
        }
        self.seen.borrow_mut().push((
            call.id.to_string(),
            call.name.to_string(),
            call.input.clone(),
            call.agent.to_string(),
        ));
        assert!(call.workspace.is_dir(), "the router is told a live root");
        assert!(!call.stop.load(std::sync::atomic::Ordering::SeqCst));
        Some(RoutedCapture {
            stdout: b"routed product".to_vec(),
            stderr: if self.exit_code == 0 {
                Vec::new()
            } else {
                b"endpoint vanished".to_vec()
            },
            exit_code: self.exit_code,
        })
    }
}

#[test]
fn a_routed_tool_answers_without_a_binary_and_lands_the_record() {
    // The harness root is empty and the driver target is a bare name
    // that would fail to spawn — so if this call resolved at all, it
    // would fail loudly rather than pass quietly.
    let root = HarnessRoot::new();
    let clock = FixedClock::default();
    let step = StepDir::new();
    let host = Embedder::new("teleop");
    let exec = SpawnTool::new(root.path(), &clock, driver_target()).with_injection(Some(&host));

    let outcome = exec
        .execute(
            ToolCall {
                id: "toolu_r1",
                name: "teleop",
                input: &json!({"do": "thing"}),
            },
            &step.path,
            &AtomicBool::new(false),
            None,
        )
        .unwrap();

    assert!(!outcome.is_error);
    assert_eq!(outcome.content, b"Exit code: 0\nrouted product");
    assert_eq!(
        *host.seen.borrow(),
        vec![(
            "toolu_r1".to_string(),
            "teleop".to_string(),
            json!({"do": "thing"}),
            super::fixtures::AGENT_ID.to_string(),
        )],
    );

    let dir = step.path.join(STEP_TOOLS_SUBDIR).join("toolu_r1");
    let input: ToolInputRecord =
        serde_json::from_slice(&std::fs::read(dir.join(INPUT_FILE)).unwrap()).unwrap();
    assert_eq!(input.name, "teleop");
    assert_eq!(input.input, json!({"do": "thing"}));
    let output: ToolOutputRecord =
        serde_json::from_slice(&std::fs::read(dir.join(OUTPUT_FILE)).unwrap()).unwrap();
    assert_eq!(output.exit_code, 0);
    assert_eq!(output.stdout, "routed product");
    assert_eq!(output.started_at, "iso-1");
    assert_eq!(output.ended_at, "iso-2");
}

#[test]
fn a_vanished_endpoint_is_an_in_band_error_result() {
    // The obligation the contract states: unreachable is a non-zero
    // result the model reads, never a harness fault and never a hang.
    let root = HarnessRoot::new();
    let clock = FixedClock::default();
    let step = StepDir::new();
    let host = Embedder::failing("teleop");
    let exec = SpawnTool::new(root.path(), &clock, driver_target()).with_injection(Some(&host));
    let outcome = exec
        .execute(
            ToolCall {
                id: "toolu_dead",
                name: "teleop",
                input: &json!({}),
            },
            &step.path,
            &AtomicBool::new(false),
            None,
        )
        .unwrap();
    assert!(
        outcome.is_error,
        "a non-zero routed exit is an error result"
    );
    assert_eq!(
        String::from_utf8_lossy(&outcome.content),
        "Exit code: 7\nrouted product\n--- stderr ---\nendpoint vanished",
    );
}

#[test]
fn a_declined_name_falls_through_to_the_spawn_path() {
    // `None` from the router is "not mine": resolution proceeds exactly
    // as if no injection were installed.
    let root = HarnessRoot::new();
    root.install("greet", r#"printf hello"#);
    let clock = FixedClock::default();
    let step = StepDir::new();
    let host = Embedder::new("teleop");
    let exec = SpawnTool::new(root.path(), &clock, driver_target()).with_injection(Some(&host));
    let outcome = exec
        .execute(
            ToolCall {
                id: "toolu_local",
                name: "greet",
                input: &json!({}),
            },
            &step.path,
            &AtomicBool::new(false),
            None,
        )
        .unwrap();
    assert_eq!(outcome.content, b"Exit code: 0\nhello");
    assert!(host.seen.borrow().is_empty(), "the host answered nothing");
}

#[test]
fn a_fan_runs_routed_and_spawned_invocations_and_reports_in_list_order() {
    // `execute_all` (a `parallel` multi-tool envelope, §3.3): the router
    // answers what it owns on this thread, the rest still overlap in the
    // scope, and results come back in the order the calls were given.
    let root = HarnessRoot::new();
    root.install("greet", r#"printf hello"#);
    let clock = FixedClock::default();
    let step = StepDir::new();
    let host = Embedder::new("teleop");
    let exec = SpawnTool::new(root.path(), &clock, driver_target()).with_injection(Some(&host));
    let input = json!({});
    let results = exec.execute_all(
        &[
            ToolCall {
                id: "f_local",
                name: "greet",
                input: &input,
            },
            ToolCall {
                id: "f_routed",
                name: "teleop",
                input: &input,
            },
        ],
        &step.path,
        &AtomicBool::new(false),
        None,
    );
    let rendered: Vec<String> = results
        .into_iter()
        .map(|r| String::from_utf8_lossy(&r.unwrap().content).into_owned())
        .collect();
    assert_eq!(
        rendered,
        vec!["Exit code: 0\nhello", "Exit code: 0\nrouted product"],
    );
    // Both landed their own record under their own tool id.
    for id in ["f_local", "f_routed"] {
        assert!(
            step.path
                .join(STEP_TOOLS_SUBDIR)
                .join(id)
                .join(OUTPUT_FILE)
                .exists(),
            "{id} landed no output record"
        );
    }
}

#[test]
fn a_fan_with_a_failed_prepare_keeps_the_routing_verdicts_aligned() {
    // A step dir the §2.2 shape cannot be read from fails `prepare` for
    // every call. The routing verdicts must still step in lockstep with
    // the calls, so the failure surfaces per call rather than shifting
    // the results by one.
    let root = HarnessRoot::new();
    let clock = FixedClock::default();
    let host = Embedder::new("teleop");
    let exec = SpawnTool::new(root.path(), &clock, driver_target()).with_injection(Some(&host));
    let input = json!({});
    let results = exec.execute_all(
        &[ToolCall {
            id: "f_nowhere",
            name: "teleop",
            input: &input,
        }],
        std::path::Path::new("/nonexistent/steps/agent/001"),
        &AtomicBool::new(false),
        None,
    );
    assert_eq!(results.len(), 1);
    assert!(
        matches!(
            results.into_iter().next().unwrap(),
            Err(super::super::ExecError::NoWorktree { .. })
        ),
        "an unresolvable caller declines before anything is routed"
    );
    assert!(host.seen.borrow().is_empty());
}

#[test]
fn the_executor_reports_the_hosts_definitions_and_nothing_without_one() {
    // The declaration half: what the composer splices and what the grant
    // gate unions, read off the executor that will answer them.
    let root = HarnessRoot::new();
    let clock = FixedClock::default();
    let host = Embedder::new("teleop");
    let with = SpawnTool::new(root.path(), &clock, driver_target()).with_injection(Some(&host));
    let names: Vec<String> = with.injected().into_iter().map(|t| t.name).collect();
    assert_eq!(names, vec!["teleop".to_string()]);
    let without = SpawnTool::new(root.path(), &clock, driver_target());
    assert!(
        without.injected().is_empty(),
        "no injection installed declares nothing"
    );
}
