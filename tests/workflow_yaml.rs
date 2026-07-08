//! Integration tests for `workflow.yaml` parsing and validation.

use lernie::config::action::{Action, DispatchMode};
use lernie::config::error::LoadError;
use lernie::config::workflow::{Backoff, Budgets, CompactionTrigger, Event, RetryConfig, Workflow};
use std::io::Write;
use std::path::Path;
use std::time::Duration;
use tempfile::NamedTempFile;

fn write_yaml(s: &str) -> NamedTempFile {
    let mut f = NamedTempFile::new().unwrap();
    f.write_all(s.as_bytes()).unwrap();
    f
}

// YAML interprets `key: value` inside a plain scalar as a map entry,
// so any action with named arguments must be quoted. Unambiguous actions
// (no `: ` inside) may stay bare.
const ARCH_EXAMPLE: &str = r#"
events:
  user_message:
    - spawn_exchange
    - dispatch(worker)
  worker_return:
    - dispatch(verifier)
    - gate_merge_on(verifier.approve)
  verifier_approve:
    - dispatch(compactor)
    - merge
  verifier_reject:
    - "dispatch(worker, with: verifier.feedback)"
  worker_flush:
    - "dispatch(compactor, mode: intermediate)"
  branch_stopped:
    - mark_abandoned
    - notify_ui
  pre_step: []
  post_step: []
  on_tool_return: []

compaction:
  intermediate:
    trigger: every_n_commits
    n: 10
"#;

#[test]
fn parses_arch_example() {
    let f = write_yaml(ARCH_EXAMPLE);
    let w = Workflow::load(f.path()).unwrap();
    let typed = w.typed_events();
    assert_eq!(typed[&Event::UserMessage].len(), 2);
    assert_eq!(
        typed[&Event::WorkerFlush][0],
        Action::Dispatch {
            role: "compactor".into(),
            with: None,
            mode: Some(DispatchMode::Intermediate)
        }
    );
    assert_eq!(
        w.compaction.as_ref().unwrap().intermediate.trigger,
        CompactionTrigger::EveryNCommits
    );
}

#[test]
fn parses_explicit_retry_block() {
    // Covers the RetryConfig + Backoff deserialize path (ARCH §6, §2.10).
    let f = write_yaml(
        "events:\n  user_message:\n    - merge\nretry:\n  max_attempts: 5\n  backoff: exponential\n",
    );
    let w = Workflow::load(f.path()).unwrap();
    assert_eq!(w.retry.max_attempts, 5);
    assert_eq!(w.retry.backoff, Backoff::Exponential);
    // Exponential backoff doubles from the first rung.
    let d1 = w.retry.backoff.delay(1);
    let d2 = w.retry.backoff.delay(2);
    assert!(d2 > d1 && d1 > Duration::ZERO);
}

#[test]
fn omitted_retry_block_uses_the_default() {
    // No `retry:` → RetryConfig::default (3 attempts, exponential).
    let f = write_yaml("events:\n  user_message:\n    - merge\n");
    let w = Workflow::load(f.path()).unwrap();
    assert_eq!(w.retry, RetryConfig::default());
    assert_eq!(w.retry.max_attempts, 3);
    assert_eq!(w.retry.backoff, Backoff::Exponential);
}

#[test]
fn parses_explicit_budgets_block() {
    // ARCH §6 budgets example: all three limits declared.
    let f = write_yaml(
        "events: {}\nbudgets:\n  max_total_tokens: 2000000\n  max_wall_seconds: 3600\n  max_depth: 4\n",
    );
    let w = Workflow::load(f.path()).unwrap();
    assert_eq!(w.budgets.max_total_tokens, Some(2_000_000));
    assert_eq!(w.budgets.max_wall_seconds, Some(3600));
    assert_eq!(w.budgets.max_depth, Some(4));
}

#[test]
fn omitted_budgets_block_is_all_unbounded() {
    // No `budgets:` → Budgets::default (every axis None = unbounded).
    let f = write_yaml("events:\n  user_message:\n    - merge\n");
    let w = Workflow::load(f.path()).unwrap();
    assert_eq!(w.budgets, Budgets::default());
    assert!(w.budgets.max_total_tokens.is_none());
    assert!(w.budgets.max_wall_seconds.is_none());
    assert!(w.budgets.max_depth.is_none());
}

#[test]
fn partial_budgets_leaves_the_other_axes_unbounded() {
    // A single declared limit; the rest stay unbounded (§6).
    let f = write_yaml("events: {}\nbudgets:\n  max_total_tokens: 500\n");
    let w = Workflow::load(f.path()).unwrap();
    assert_eq!(w.budgets.max_total_tokens, Some(500));
    assert!(w.budgets.max_wall_seconds.is_none());
    assert!(w.budgets.max_depth.is_none());
}

#[test]
fn workflow_without_compaction_is_ok() {
    let f = write_yaml("events:\n  user_message:\n    - spawn_exchange\n");
    let w = Workflow::load(f.path()).unwrap();
    assert!(w.compaction.is_none());
}

#[test]
fn rejects_unknown_event() {
    let f = write_yaml("events:\n  user_request:\n    - spawn_exchange\n");
    let err = Workflow::load(f.path()).unwrap_err();
    assert!(matches!(err, LoadError::Yaml { .. }));
}

#[test]
fn rejects_unknown_action() {
    let f = write_yaml("events:\n  user_message:\n    - teleport(worker)\n");
    let err = Workflow::load(f.path()).unwrap_err();
    match err {
        LoadError::Invalid { key, message, .. } => {
            assert_eq!(key, "events.user_message[0]");
            assert!(message.contains("unknown action"));
        }
        other => panic!("expected Invalid, got {other:?}"),
    }
}

#[test]
fn rejects_compaction_missing_n_for_count_trigger() {
    let yaml = "events: {}\ncompaction:\n  intermediate:\n    trigger: every_n_commits\n";
    let f = write_yaml(yaml);
    let err = Workflow::load(f.path()).unwrap_err();
    match err {
        LoadError::Invalid { key, .. } => assert_eq!(key, "compaction.intermediate.n"),
        other => panic!("expected Invalid, got {other:?}"),
    }
}

#[test]
fn rejects_compaction_missing_n_for_seconds_trigger() {
    let yaml = "events: {}\ncompaction:\n  intermediate:\n    trigger: every_t_seconds\n";
    let f = write_yaml(yaml);
    assert!(matches!(
        Workflow::load(f.path()),
        Err(LoadError::Invalid { .. })
    ));
}

#[test]
fn on_flush_trigger_does_not_need_n() {
    let yaml = "events: {}\ncompaction:\n  intermediate:\n    trigger: on_flush\n";
    let f = write_yaml(yaml);
    assert!(Workflow::load(f.path()).is_ok());
}

#[test]
fn rejects_compaction_zero_n() {
    let yaml = "events: {}\ncompaction:\n  intermediate:\n    trigger: every_n_commits\n    n: 0\n";
    let f = write_yaml(yaml);
    assert!(matches!(
        Workflow::load(f.path()),
        Err(LoadError::Invalid { .. })
    ));
}

#[test]
fn surfaces_io_errors() {
    assert!(matches!(
        Workflow::load(Path::new("/no/such/workflow.yaml")),
        Err(LoadError::Io { .. })
    ));
}

const EVENT_NAMES: &[&str] = &[
    "user_message",
    "worker_return",
    "verifier_approve",
    "verifier_reject",
    "worker_flush",
    "branch_stopped",
    "pre_step",
    "post_step",
    "on_tool_return",
];

#[test]
fn each_event_name_round_trips() {
    for name in EVENT_NAMES {
        let yaml = format!("events:\n  {name}: []\n");
        let f = write_yaml(&yaml);
        Workflow::load(f.path()).unwrap_or_else(|e| panic!("event {name} did not round-trip: {e}"));
    }
}

// Exercises the per-event branches of the internal `event_name` map by
// triggering an invalid-action error under every event.
#[test]
fn invalid_action_error_message_names_each_event() {
    for name in EVENT_NAMES {
        let yaml = format!("events:\n  {name}:\n    - bogus_action\n");
        let f = write_yaml(&yaml);
        let err = Workflow::load(f.path()).unwrap_err();
        match err {
            LoadError::Invalid { key, .. } => {
                assert!(
                    key.contains(name),
                    "expected key to contain {name}, got {key}"
                );
            }
            other => panic!("expected Invalid for {name}, got {other:?}"),
        }
    }
}
