//! The tool-control seam inside a multi-tool envelope (ARCH §3.3 *Tool
//! control*, *The multi-tool* — No bypass): the control gates inner
//! invocations exactly as top-level ones, except a hold cannot park
//! mid-envelope and degrades to an in-band decline instructing a
//! top-level re-issue.

use super::Resolution;
use super::multi::{Fixture, Scripted, outcome_text};
use super::seam::{control_script, gated};
use crate::prompt::dispatch::tool_step::multi::{Fanout, fan_out};
use serde_json::json;
use std::sync::atomic::AtomicBool;
use tempfile::TempDir;

#[test]
fn an_inner_refuse_declines_in_place_and_the_rest_still_runs() {
    let fx = Fixture::new();
    let exec = Scripted::new();
    let stop = AtomicBool::new(false);
    let deps = fx.deps(&exec, &stop);
    let scripts = TempDir::new().unwrap();
    let control = control_script(
        scripts.path(),
        "if grep -q '\"name\":\"beta\"'; then \
           echo '{\"verdict\":\"refuse\",\"reason\":\"beta is gated\"}'; \
         else echo '{\"verdict\":\"pass\"}'; fi",
    );
    let mut resolution = Resolution::new();
    gated(&mut resolution, &control);
    let grant = ["multi_tool".into(), "alpha".into(), "beta".into()];
    let input = json!({"invocations": [
        {"name": "beta"}, {"name": "alpha"},
    ], "on_failure": "run_all"});
    let step_dir = TempDir::new().unwrap();
    let fanout = fan_out(
        "t8",
        &input,
        step_dir.path(),
        &resolution.of(crate::prompt::WORKER_ROLE, &grant),
        step_dir.path(),
        "agent-multi",
        &deps,
    )
    .unwrap();
    let log = exec.log.borrow();
    assert_eq!(log.len(), 1, "only the passed tool ran: {log:?}");
    assert_eq!(log[0].1, "alpha");
    drop(log);
    let (text, is_error) = outcome_text(fanout);
    assert!(is_error);
    assert!(text.contains("=== [1/2] beta: declined ==="), "{text}");
    assert!(
        text.contains("refused by the workflow's tool control"),
        "{text}"
    );
    assert!(text.contains("beta is gated"), "{text}");
}

#[test]
fn an_inner_hold_degrades_to_a_decline_naming_the_top_level_re_issue() {
    // A hold cannot park mid-envelope (earlier entries already ran and
    // are uncommitted); the invocation still never executes.
    let fx = Fixture::new();
    let exec = Scripted::new();
    let stop = AtomicBool::new(false);
    let deps = fx.deps(&exec, &stop);
    let scripts = TempDir::new().unwrap();
    let control = control_script(
        scripts.path(),
        "echo '{\"verdict\":\"hold\",\"reason\":\"needs review\"}'",
    );
    let mut resolution = Resolution::new();
    gated(&mut resolution, &control);
    let grant = ["multi_tool".into(), "alpha".into()];
    let input = json!({"invocations": [{"name": "alpha"}]});
    let step_dir = TempDir::new().unwrap();
    let fanout = fan_out(
        "t9",
        &input,
        step_dir.path(),
        &resolution.of(crate::prompt::WORKER_ROLE, &grant),
        step_dir.path(),
        "agent-multi",
        &deps,
    )
    .unwrap();
    assert!(exec.log.borrow().is_empty(), "a held inner never executes");
    let (text, is_error) = outcome_text(fanout);
    assert!(is_error);
    assert!(text.contains("=== [1/1] alpha: declined ==="), "{text}");
    assert!(text.contains("cannot park mid-envelope"), "{text}");
    assert!(text.contains("re-issue this invocation"), "{text}");
    assert!(text.contains("needs review"), "{text}");
}

#[test]
fn a_stop_felling_an_inner_consult_ceases_the_envelope() {
    // The §2.9 cascade lands on the control mid-envelope: same reading
    // as a felled inner tool — the fan-out ceases, nothing aggregated.
    let fx = Fixture::new();
    let exec = Scripted::new();
    let stop = AtomicBool::new(true);
    let deps = fx.deps(&exec, &stop);
    let scripts = TempDir::new().unwrap();
    let control = control_script(scripts.path(), "exec sleep 60");
    let mut resolution = Resolution::new();
    gated(&mut resolution, &control);
    let grant = ["multi_tool".into(), "alpha".into()];
    let input = json!({"invocations": [{"name": "alpha"}]});
    let step_dir = TempDir::new().unwrap();
    let fanout = fan_out(
        "t10",
        &input,
        step_dir.path(),
        &resolution.of(crate::prompt::WORKER_ROLE, &grant),
        step_dir.path(),
        "agent-multi",
        &deps,
    )
    .unwrap();
    assert!(matches!(fanout, Fanout::Stopped), "got {fanout:?}");
    assert!(exec.log.borrow().is_empty());
}
