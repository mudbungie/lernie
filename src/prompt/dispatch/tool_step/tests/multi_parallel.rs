//! `execution: "parallel"` (ARCH §3.3 *The multi-tool*): the envelope
//! asserts its entries do not collide, every entry is gated before any
//! runs, and the survivors go to the executor together. Results still
//! render in list order, and `on_failure` — a sequencing policy — is
//! not consulted.

use super::multi::{Fixture, Scripted, outcome_text};
use super::seam::{control_script, gated};
use super::{Resolution, branch_with_step};
use crate::prompt::dispatch::tool_step::multi::{Fanout, fan_out};
use serde_json::json;
use std::sync::atomic::AtomicBool;
use tempfile::TempDir;

/// Drive one envelope through [`fan_out`] with the given grant.
fn fan(envelope: serde_json::Value, exec: &Scripted, grant: &[String]) -> Fanout {
    let agent_id = "agent-ec74";
    let ws = TempDir::new().unwrap();
    let fx = Fixture::new();
    let (_worktree, step_dir_rel) = branch_with_step(&ws, agent_id, &fx.git);
    let stop = AtomicBool::new(false);
    let deps = fx.deps(exec, &stop);
    let resolution = Resolution::new();
    fan_out(
        "t1",
        &envelope,
        &ws.path().join(&step_dir_rel),
        &resolution.of(crate::prompt::WORKER_ROLE, grant),
        ws.path(),
        agent_id,
        &deps,
    )
    .unwrap()
}

fn grant_of(names: &[&str]) -> Vec<String> {
    names.iter().map(|n| (*n).to_string()).collect()
}

#[test]
fn parallel_runs_every_entry_and_reports_in_list_order() {
    // Completion order is the scheduler's; reported order is the
    // model's. The default executor runs `execute_all` serially, which
    // is a correct answer to "run these" — the ordering guarantee is
    // the fan's, not the executor's.
    let exec = Scripted::new();
    let out = fan(
        json!({"execution": "parallel", "invocations": [
            {"name": "alpha", "input": {"x": 1}},
            {"name": "beta"},
            {"name": "gamma"},
        ]}),
        &exec,
        &grant_of(&["alpha", "beta", "gamma"]),
    );
    let (text, is_error) = outcome_text(out);
    assert!(!is_error, "{text}");
    assert!(
        text.starts_with("3 invocations: 3 ok, 0 failed, 0 skipped"),
        "{text}"
    );
    assert!(
        text.contains("=== [1/3] alpha: ok ===\nran alpha"),
        "{text}"
    );
    assert!(text.contains("=== [2/3] beta: ok ===\nran beta"), "{text}");
    assert!(
        text.contains("=== [3/3] gamma: ok ===\nran gamma"),
        "{text}"
    );
    // Derived ids are still position-based, and the omitted input is
    // still the empty object.
    let log = exec.log.borrow();
    assert_eq!(log[0], ("t1-1".into(), "alpha".into(), json!({"x": 1})));
    assert_eq!(log[1], ("t1-2".into(), "beta".into(), json!({})));
    assert_eq!(log[2], ("t1-3".into(), "gamma".into(), json!({})));
}

#[test]
fn on_failure_abort_does_not_skip_under_parallel() {
    // `abort` is a sequencing policy: once every entry has started,
    // there is nothing left to skip. The entry after the failure runs
    // and reports its own outcome — the one behaviour that would
    // differ if `execution` were ignored.
    let mut exec = Scripted::new();
    exec.fail = &["beta"];
    let out = fan(
        json!({"execution": "parallel", "on_failure": "abort", "invocations": [
            {"name": "alpha"},
            {"name": "beta"},
            {"name": "gamma"},
        ]}),
        &exec,
        &grant_of(&["alpha", "beta", "gamma"]),
    );
    let (text, is_error) = outcome_text(out);
    assert!(is_error, "{text}");
    assert!(
        text.starts_with("3 invocations: 2 ok, 1 failed, 0 skipped"),
        "{text}"
    );
    assert!(
        text.contains("=== [3/3] gamma: ok ===\nran gamma"),
        "{text}"
    );
    assert_eq!(exec.log.borrow().len(), 3);
}

#[test]
fn a_declined_entry_weaves_back_into_its_own_position() {
    // The gate pass and the execute pass are separate walks, so the
    // weave has to put a never-executed entry back where the model
    // wrote it — here, between two that did run.
    let exec = Scripted::new();
    let out = fan(
        json!({"execution": "parallel", "invocations": [
            {"name": "alpha"},
            {"name": "ungranted"},
            {"name": "gamma"},
        ]}),
        &exec,
        &grant_of(&["alpha", "gamma"]),
    );
    let (text, is_error) = outcome_text(out);
    assert!(is_error, "{text}");
    assert!(
        text.starts_with("3 invocations: 2 ok, 1 failed, 0 skipped"),
        "{text}"
    );
    assert!(text.contains("=== [2/3] ungranted: declined ==="), "{text}");
    assert!(
        text.contains("=== [3/3] gamma: ok ===\nran gamma"),
        "{text}"
    );
    // The declined entry never reached the executor.
    let log = exec.log.borrow();
    assert_eq!(log.len(), 2);
    assert_eq!(log[1].1, "gamma");
}

#[test]
fn a_nested_envelope_is_declined_before_any_sibling_runs() {
    // Depth 1 is a gate, and every gate clears before the first
    // invocation starts — so the decline cannot land after a sibling
    // it was meant to precede.
    let exec = Scripted::new();
    let out = fan(
        json!({"execution": "parallel", "invocations": [
            {"name": "multi_tool"},
            {"name": "alpha"},
        ]}),
        &exec,
        &grant_of(&["alpha", "multi_tool"]),
    );
    let (text, _) = outcome_text(out);
    assert!(
        text.contains("=== [1/2] multi_tool: declined ==="),
        "{text}"
    );
    assert!(text.contains("may not contain itself"), "{text}");
    assert_eq!(exec.log.borrow().len(), 1);
}

#[test]
fn the_stop_observed_mid_fan_ceases_the_loop() {
    // Same reading as the serial path: the executor's own group
    // SIGTERM with the stop flag set is the §2.9 stop, not a fault —
    // nothing is committed.
    let agent_id = "agent-ec74";
    let ws = TempDir::new().unwrap();
    let fx = Fixture::new();
    let (_worktree, step_dir_rel) = branch_with_step(&ws, agent_id, &fx.git);
    let mut exec = Scripted::new();
    exec.kill = &["beta"];
    let stop = AtomicBool::new(true);
    let deps = fx.deps(&exec, &stop);
    let resolution = Resolution::new();
    let out = fan_out(
        "t1",
        &json!({"execution": "parallel", "invocations": [
            {"name": "alpha"},
            {"name": "beta"},
        ]}),
        &ws.path().join(&step_dir_rel),
        &resolution.of(crate::prompt::WORKER_ROLE, &grant_of(&["alpha", "beta"])),
        ws.path(),
        agent_id,
        &deps,
    )
    .unwrap();
    assert!(matches!(out, Fanout::Stopped));
}

#[test]
fn a_harness_fault_propagates_out_of_the_fan() {
    // A spawn fault is not a tool failure: it leaves the fan as
    // `Error::ToolExec`, exactly as it would from a top-level call.
    let agent_id = "agent-ec74";
    let ws = TempDir::new().unwrap();
    let fx = Fixture::new();
    let (_worktree, step_dir_rel) = branch_with_step(&ws, agent_id, &fx.git);
    let mut exec = Scripted::new();
    exec.fault = &["beta"];
    let stop = AtomicBool::new(false);
    let deps = fx.deps(&exec, &stop);
    let resolution = Resolution::new();
    let err = fan_out(
        "t1",
        &json!({"execution": "parallel", "invocations": [
            {"name": "alpha"},
            {"name": "beta"},
        ]}),
        &ws.path().join(&step_dir_rel),
        &resolution.of(crate::prompt::WORKER_ROLE, &grant_of(&["alpha", "beta"])),
        ws.path(),
        agent_id,
        &deps,
    )
    .unwrap_err();
    assert!(matches!(err, crate::prompt::Error::ToolExec { .. }));
}

#[test]
fn serial_stays_the_default_when_execution_is_absent() {
    // The field is opt-in: an envelope that says nothing keeps the
    // sequencing guarantee a list written in an order implies.
    let mut exec = Scripted::new();
    exec.fail = &["alpha"];
    let out = fan(
        json!({"invocations": [{"name": "alpha"}, {"name": "beta"}]}),
        &exec,
        &grant_of(&["alpha", "beta"]),
    );
    let (text, _) = outcome_text(out);
    assert!(
        text.starts_with("2 invocations: 0 ok, 1 failed, 1 skipped"),
        "{text}"
    );
    assert_eq!(exec.log.borrow().len(), 1);
}

#[test]
fn an_unknown_execution_mode_is_declined_in_band() {
    // Same idiom as any malformed envelope: the model is told the
    // shape and re-emits.
    let exec = Scripted::new();
    let out = fan(
        json!({"execution": "whenever", "invocations": [{"name": "alpha"}]}),
        &exec,
        &grant_of(&["alpha"]),
    );
    let (text, is_error) = outcome_text(out);
    assert!(is_error, "{text}");
    assert!(text.contains("malformed envelope"), "{text}");
    assert!(exec.log.borrow().is_empty());
}

#[test]
fn a_stop_during_the_gate_pass_ceases_the_fan_before_anything_runs() {
    // The gate walk is where a `parallel` fan differs: the §2.9 cascade
    // landing on the tool control fells the envelope while every entry
    // is still unstarted, so no invocation reaches the executor at all.
    let fx = Fixture::new();
    let exec = Scripted::new();
    let stop = AtomicBool::new(true);
    let deps = fx.deps(&exec, &stop);
    let scripts = TempDir::new().unwrap();
    let control = control_script(scripts.path(), "exec sleep 60");
    let mut resolution = Resolution::new();
    gated(&mut resolution, &control);
    let step_dir = TempDir::new().unwrap();
    let fanout = fan_out(
        "t1",
        &json!({"execution": "parallel", "invocations": [
            {"name": "alpha"},
            {"name": "beta"},
        ]}),
        step_dir.path(),
        &resolution.of(
            crate::prompt::WORKER_ROLE,
            &grant_of(&["multi_tool", "alpha", "beta"]),
        ),
        step_dir.path(),
        "agent-ec74",
        &deps,
    )
    .unwrap();
    assert!(matches!(fanout, Fanout::Stopped), "got {fanout:?}");
    assert!(exec.log.borrow().is_empty());
}
