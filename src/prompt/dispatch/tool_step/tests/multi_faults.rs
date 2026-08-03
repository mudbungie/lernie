//! The multi-tool's decline and fault paths (ARCH §3.3 *The
//! multi-tool*): no inner invocation bypasses the grant gate, nesting
//! is declined at depth 1, a malformed envelope is declined in-band,
//! and the §2.9 stop / §2.10 harness-fault readings match a top-level
//! invocation's exactly.

use super::multi::{Fixture, Scripted, outcome_text};
use super::{Resolution, branch_with_step};
use crate::prompt::Error;
use crate::prompt::dispatch::tool_step::multi::fan_out;
use crate::prompt::dispatch::tool_step::run_tool_calls;
use brazen::Content;
use serde_json::json;
use std::sync::atomic::AtomicBool;
use tempfile::TempDir;

#[test]
fn an_ungranted_inner_invocation_is_declined_and_never_reaches_the_executor() {
    // The same gate as a top-level refusal (declaring is not
    // permitting, bl-5a1f): `beta` is outside the grant, so its entry
    // is declined in place while the granted `alpha` still runs.
    let fx = Fixture::new();
    let exec = Scripted::new();
    let stop = AtomicBool::new(false);
    let deps = fx.deps(&exec, &stop);
    let resolution = Resolution::new();
    let grant = ["multi_tool".into(), "alpha".into()];
    let input = json!({"invocations": [
        {"name": "beta"}, {"name": "alpha"},
    ], "on_failure": "run_all"});
    let step_dir = TempDir::new().unwrap();
    let fanout = fan_out(
        "t2",
        &input,
        step_dir.path(),
        &resolution.of("sensor", &grant),
        &deps,
    )
    .unwrap();
    let log = exec.log.borrow();
    assert_eq!(log.len(), 1, "only the granted tool ran: {log:?}");
    assert_eq!(log[0].1, "alpha");
    drop(log);
    let (text, is_error) = outcome_text(fanout);
    assert!(is_error, "a declined entry marks the aggregate: {text}");
    assert!(
        text.starts_with("2 invocations: 1 ok, 1 failed, 0 skipped"),
        "{text}"
    );
    assert!(text.contains("=== [1/2] beta: declined ==="), "{text}");
    assert!(text.contains("not callable by a sensor"), "{text}");
}

#[test]
fn a_nested_multi_tool_is_declined_at_depth_one() {
    let fx = Fixture::new();
    let exec = Scripted::new();
    let stop = AtomicBool::new(false);
    let deps = fx.deps(&exec, &stop);
    let resolution = Resolution::new();
    let grant = ["multi_tool".into()];
    let input = json!({"invocations": [{"name": "multi_tool", "input": {"invocations": []}}]});
    let step_dir = TempDir::new().unwrap();
    let fanout = fan_out(
        "t3",
        &input,
        step_dir.path(),
        &resolution.of(crate::prompt::WORKER_ROLE, &grant),
        &deps,
    )
    .unwrap();
    assert!(exec.log.borrow().is_empty(), "nesting never executes");
    let (text, is_error) = outcome_text(fanout);
    assert!(is_error);
    assert!(
        text.contains("=== [1/1] multi_tool: declined ==="),
        "{text}"
    );
    assert!(text.contains("may not contain itself (depth 1)"), "{text}");
}

#[test]
fn a_malformed_envelope_is_declined_in_band_and_nothing_runs() {
    let fx = Fixture::new();
    let exec = Scripted::new();
    let stop = AtomicBool::new(false);
    let deps = fx.deps(&exec, &stop);
    let resolution = Resolution::new();
    let grant = ["multi_tool".into()];
    let input = json!({"invocations": "not a list"});
    let step_dir = TempDir::new().unwrap();
    let fanout = fan_out(
        "t4",
        &input,
        step_dir.path(),
        &resolution.of(crate::prompt::WORKER_ROLE, &grant),
        &deps,
    )
    .unwrap();
    assert!(exec.log.borrow().is_empty());
    let (text, is_error) = outcome_text(fanout);
    assert!(is_error);
    assert!(text.contains("multi_tool: malformed envelope"), "{text}");
    assert!(text.contains("Expected {\"invocations\""), "{text}");
}

#[test]
fn a_stop_mid_envelope_ceases_the_loop_and_commits_nothing() {
    // §2.9 step 3 through the envelope: the executor's own group
    // SIGTERM with the stop flag set is the stop — the whole tool
    // window ceases for the stopped-deposit exit, and no aggregate
    // entry is committed (the envelope never resolved).
    let agent_id = "agent-8ee7-stop";
    let ws = TempDir::new().unwrap();
    let fx = Fixture::new();
    let (worktree, step_dir_rel) = branch_with_step(&ws, agent_id, &fx.git);
    let exec = Scripted {
        kill: &["boom"],
        ..Scripted::new()
    };
    let stop = AtomicBool::new(true);
    let deps = fx.deps(&exec, &stop);
    let content = vec![Content::ToolUse {
        id: "t5".into(),
        name: "multi_tool".into(),
        input: json!({"invocations": [
            {"name": "alpha"}, {"name": "boom"}, {"name": "gamma"},
        ]}),
        signature: None,
    }];
    let grant = [
        "multi_tool".into(),
        "alpha".into(),
        "boom".into(),
        "gamma".into(),
    ];
    let resolution = Resolution::new();
    let stopped = run_tool_calls(
        ws.path(),
        &worktree,
        agent_id,
        &resolution.of(crate::prompt::WORKER_ROLE, &grant),
        &step_dir_rel,
        &content,
        &deps,
    )
    .unwrap();
    assert!(stopped, "the stop ceases the loop");
    assert_eq!(exec.log.borrow().len(), 2, "the third entry never ran");
    assert!(!worktree.join("messages/002-tool.json").exists());
}

#[test]
fn a_harness_fault_in_an_inner_invocation_propagates_as_tool_exec() {
    // §2.10: a spawn failure is a harness-level fault, not an in-band
    // entry — same reading as a top-level invocation.
    let fx = Fixture::new();
    let exec = Scripted {
        fault: &["alpha"],
        ..Scripted::new()
    };
    let stop = AtomicBool::new(false);
    let deps = fx.deps(&exec, &stop);
    let resolution = Resolution::new();
    let grant = ["multi_tool".into(), "alpha".into()];
    let input = json!({"invocations": [{"name": "alpha"}]});
    let step_dir = TempDir::new().unwrap();
    let err = fan_out(
        "t6",
        &input,
        step_dir.path(),
        &resolution.of(crate::prompt::WORKER_ROLE, &grant),
        &deps,
    )
    .unwrap_err();
    let Error::ToolExec { tool, .. } = err else {
        panic!("expected ToolExec, got {err:?}");
    };
    assert_eq!(tool, "alpha");
}

#[test]
fn a_kill_with_no_stop_pending_is_a_genuine_fault() {
    // KilledBySignal with the stop flag clear is a crash (§2.10), not
    // the stop — it propagates instead of ceasing cleanly.
    let fx = Fixture::new();
    let exec = Scripted {
        kill: &["alpha"],
        ..Scripted::new()
    };
    let stop = AtomicBool::new(false);
    let deps = fx.deps(&exec, &stop);
    let resolution = Resolution::new();
    let grant = ["multi_tool".into(), "alpha".into()];
    let input = json!({"invocations": [{"name": "alpha"}]});
    let step_dir = TempDir::new().unwrap();
    let err = fan_out(
        "t7",
        &input,
        step_dir.path(),
        &resolution.of(crate::prompt::WORKER_ROLE, &grant),
        &deps,
    )
    .unwrap_err();
    assert!(matches!(err, Error::ToolExec { .. }), "{err:?}");
}
