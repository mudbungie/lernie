//! The multi-tool's fan-out semantics (ARCH §3.3 *The multi-tool*):
//! one `tool_use` envelope, N inner invocations run serially in list
//! order through the same executor as top-level ones, all results
//! aggregated into one committed `tool_result`. Declines and harness
//! faults live in [`super::multi_faults`].

use super::{NoAdapter, NoLauncher, NoSleeper, Resolution, branch_with_step};
use crate::prompt::clock::SystemClock;
use crate::prompt::dispatch::tool_step::multi::{Fanout, fan_out};
use crate::prompt::dispatch::tool_step::{ToolWindow, run_tool_calls};
use crate::prompt::tool::{ExecError, ToolCall, ToolExecutor, ToolOutcome};
use crate::template::RealGit;
use brazen::Content;
use serde_json::{Value, json};
use std::cell::RefCell;
use std::sync::atomic::AtomicBool;
use tempfile::TempDir;

/// A scripted executor: records every invocation it is handed (derived
/// id, name, input) and answers per name — an `is_error` outcome for
/// `fail` names, the §2.9 group-SIGTERM signature for `kill` names, a
/// spawn fault for `fault` names, a plain `ran <name>` outcome
/// otherwise.
pub(super) struct Scripted {
    pub(super) log: RefCell<Vec<(String, String, Value)>>,
    pub(super) fail: &'static [&'static str],
    pub(super) kill: &'static [&'static str],
    pub(super) fault: &'static [&'static str],
}

impl Scripted {
    pub(super) fn new() -> Self {
        Self {
            log: RefCell::new(Vec::new()),
            fail: &[],
            kill: &[],
            fault: &[],
        }
    }
}

impl ToolExecutor for Scripted {
    fn execute(
        &self,
        invocation: ToolCall<'_>,
        _step_dir: &std::path::Path,
        _stop: &AtomicBool,
        _bound: Option<crate::config::ToolOutputBound>,
    ) -> Result<ToolOutcome, ExecError> {
        self.log.borrow_mut().push((
            invocation.id.to_string(),
            invocation.name.to_string(),
            invocation.input.clone(),
        ));
        if self.kill.contains(&invocation.name) {
            return Err(ExecError::KilledBySignal {
                name: invocation.name.to_string(),
                signal: 15,
            });
        }
        if self.fault.contains(&invocation.name) {
            return Err(ExecError::Spawn {
                name: invocation.name.to_string(),
                source: std::io::Error::other("no such binary"),
            });
        }
        Ok(ToolOutcome {
            content: format!("ran {}", invocation.name).into_bytes(),
            is_error: self.fail.contains(&invocation.name),
        })
    }
}

/// The step machinery shared by every fixture here: real git for the
/// integration path, a scratch config root, and the never-reached stubs.
pub(super) struct Fixture {
    pub(super) git: RealGit,
    clock: SystemClock,
    id_gen: crate::prompt::NanoIdGen,
    cfg: TempDir,
}

impl Fixture {
    pub(super) fn new() -> Self {
        Self {
            git: RealGit::new(),
            clock: SystemClock,
            id_gen: crate::prompt::NanoIdGen,
            cfg: TempDir::new().unwrap(),
        }
    }

    pub(super) fn deps<'a>(
        &'a self,
        exec: &'a dyn ToolExecutor,
        stop: &'a AtomicBool,
    ) -> crate::prompt::Deps<'a> {
        crate::prompt::Deps {
            adapter: &NoAdapter,
            sleeper: &NoSleeper,
            git: &self.git,
            clock: &self.clock,
            id_gen: &self.id_gen,
            tool_executor: exec,
            config_root: self.cfg.path(),
            adapter_target: None,
            stop,
            launcher: &NoLauncher,
            rng: crate::workspace::agent_name::mint::test_rng(),
        }
    }
}

/// The result text of a completed fan-out, or a panic on any other exit.
pub(super) fn outcome_text(fanout: Fanout) -> (String, bool) {
    let Fanout::Outcome(o) = fanout else {
        panic!("expected an aggregated outcome");
    };
    (String::from_utf8(o.content).unwrap(), o.is_error)
}

#[test]
fn an_envelope_fans_out_serially_and_commits_one_aggregated_entry() {
    // Two inner invocations through the whole tool window: derived ids
    // `<outer>-<k>` reach the executor in list order (the second's
    // omitted `input` as `{}`), and exactly one transcript entry lands,
    // attributed per entry under the envelope's own wire id.
    let agent_id = "agent-8ee7";
    let ws = TempDir::new().unwrap();
    let fx = Fixture::new();
    let (worktree, step_dir_rel) = branch_with_step(&ws, agent_id, &fx.git);
    let exec = Scripted::new();
    let stop = AtomicBool::new(false);
    let deps = fx.deps(&exec, &stop);
    let content = vec![Content::ToolUse {
        id: "t1".into(),
        name: "multi_tool".into(),
        input: json!({"invocations": [
            {"name": "alpha", "input": {"x": 1}},
            {"name": "beta"},
        ]}),
        signature: None,
    }];
    let grant = ["multi_tool".into(), "alpha".into(), "beta".into()];
    let resolution = Resolution::new();
    let window = run_tool_calls(
        ws.path(),
        &worktree,
        agent_id,
        &resolution.of(crate::prompt::WORKER_ROLE, &grant),
        &step_dir_rel,
        &content,
        &deps,
    )
    .unwrap();
    assert!(matches!(window, ToolWindow::Completed));
    let want = vec![
        ("t1-1".to_string(), "alpha".to_string(), json!({"x": 1})),
        ("t1-2".to_string(), "beta".to_string(), json!({})),
    ];
    assert_eq!(*exec.log.borrow(), want);
    // One committed entry for the whole envelope — block-on-all.
    let entry = std::fs::read_to_string(worktree.join("messages/002-tool.json")).unwrap();
    assert!(!worktree.join("messages/003-tool.json").exists());
    let blocks: Vec<Content> = serde_json::from_str(&entry).unwrap();
    let Content::ToolResult {
        tool_use_id,
        content,
        is_error,
    } = &blocks[0]
    else {
        panic!("expected a tool_result, got {:?}", blocks[0]);
    };
    assert_eq!(tool_use_id, "t1");
    assert!(!is_error);
    let Content::Text(text) = &content[0] else {
        panic!("the aggregate is text");
    };
    assert!(
        text.starts_with("2 invocations: 2 ok, 0 failed, 0 skipped"),
        "{text}"
    );
    assert!(
        text.contains("=== [1/2] alpha: ok ===\nran alpha"),
        "{text}"
    );
    assert!(text.contains("=== [2/2] beta: ok ===\nran beta"), "{text}");
}

#[test]
fn abort_is_the_default_and_skips_every_entry_after_a_failure() {
    let fx = Fixture::new();
    let exec = Scripted {
        fail: &["bad"],
        ..Scripted::new()
    };
    let stop = AtomicBool::new(false);
    let deps = fx.deps(&exec, &stop);
    let resolution = Resolution::new();
    let grant = [
        "multi_tool".into(),
        "alpha".into(),
        "bad".into(),
        "gamma".into(),
    ];
    let input = json!({"invocations": [
        {"name": "alpha"}, {"name": "bad"}, {"name": "gamma"},
    ]});
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
    // The third invocation never reached the executor.
    assert_eq!(exec.log.borrow().len(), 2);
    let (text, is_error) = outcome_text(fanout);
    assert!(is_error, "a failed entry marks the aggregate: {text}");
    assert!(
        text.starts_with("3 invocations: 1 ok, 1 failed, 1 skipped"),
        "{text}"
    );
    assert!(text.contains("=== [2/3] bad: failed ==="), "{text}");
    assert!(text.contains("=== [3/3] gamma: skipped ==="), "{text}");
    assert!(
        text.contains("on_failure \"abort\" ended the envelope after [2/3] failed"),
        "{text}"
    );
}

#[test]
fn run_all_runs_every_entry_despite_a_failure() {
    let fx = Fixture::new();
    let exec = Scripted {
        fail: &["bad"],
        ..Scripted::new()
    };
    let stop = AtomicBool::new(false);
    let deps = fx.deps(&exec, &stop);
    let resolution = Resolution::new();
    let grant = [
        "multi_tool".into(),
        "alpha".into(),
        "bad".into(),
        "gamma".into(),
    ];
    let input = json!({"invocations": [
        {"name": "alpha"}, {"name": "bad"}, {"name": "gamma"},
    ], "on_failure": "run_all"});
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
    assert_eq!(exec.log.borrow().len(), 3);
    let (text, is_error) = outcome_text(fanout);
    assert!(is_error);
    assert!(
        text.starts_with("3 invocations: 2 ok, 1 failed, 0 skipped"),
        "{text}"
    );
    assert!(text.contains("=== [3/3] gamma: ok ==="), "{text}");
}

#[test]
fn an_empty_envelope_is_the_general_path_with_empty_inputs() {
    let fx = Fixture::new();
    let exec = Scripted::new();
    let stop = AtomicBool::new(false);
    let deps = fx.deps(&exec, &stop);
    let resolution = Resolution::new();
    let grant = ["multi_tool".into()];
    let input = json!({"invocations": []});
    let step_dir = TempDir::new().unwrap();
    let fanout = fan_out(
        "t0",
        &input,
        step_dir.path(),
        &resolution.of(crate::prompt::WORKER_ROLE, &grant),
        step_dir.path(),
        "agent-multi",
        &deps,
    )
    .unwrap();
    assert!(exec.log.borrow().is_empty());
    let (text, is_error) = outcome_text(fanout);
    assert!(!is_error);
    assert_eq!(text, "0 invocations: 0 ok, 0 failed, 0 skipped\n");
}
