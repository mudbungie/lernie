//! `workflow.yaml` `compaction:` block validation (ARCH §2.6, §2.7,
//! §6) — split from [`workflow_yaml`](super::workflow_yaml) to hold the
//! per-file line cap.

use crate::config::error::LoadError;
use crate::config::workflow::Workflow;
use std::path::Path;

/// Same origin-labelled parse as `workflow_yaml`'s.
fn parse(raw: &str) -> Result<Workflow, LoadError> {
    Workflow::parse(raw, Path::new("<commit>:workflow.yaml"))
}

#[test]
fn workflow_without_compaction_is_ok() {
    let w = parse("events:\n  user_message:\n    - notify_ui\n").unwrap();
    assert!(w.compaction.is_none());
}

#[test]
fn rejects_a_retained_tail_at_or_over_the_commit_trigger() {
    // §2.6/§6: keep_recent >= n under every_n_commits would leave the
    // clock over threshold at every landing — refused at load. Below n
    // parses; the time trigger is unconstrained (the clock is seconds).
    let yaml = "events: {}\ncompaction:\n  intermediate:\n    trigger: every_n_commits\n    n: 5\n    keep_recent: 5\n";
    match parse(yaml).unwrap_err() {
        LoadError::Invalid { key, message, .. } => {
            assert_eq!(key, "compaction.intermediate.keep_recent");
            assert!(message.contains("smaller than n"), "{message}");
        }
        other => panic!("expected Invalid, got {other:?}"),
    }
    let ok = "events: {}\ncompaction:\n  intermediate:\n    trigger: every_n_commits\n    n: 5\n    keep_recent: 4\n";
    assert_eq!(
        parse(ok)
            .unwrap()
            .compaction
            .unwrap()
            .intermediate
            .keep_recent,
        Some(4)
    );
    let time = "events: {}\ncompaction:\n  intermediate:\n    trigger: every_t_seconds\n    n: 5\n    keep_recent: 50\n";
    assert!(parse(time).is_ok());
}

#[test]
fn rejects_compaction_missing_n_for_count_trigger() {
    let yaml = "events: {}\ncompaction:\n  intermediate:\n    trigger: every_n_commits\n";
    let err = parse(yaml).unwrap_err();
    match err {
        LoadError::Invalid { key, .. } => assert_eq!(key, "compaction.intermediate.n"),
        other => panic!("expected Invalid, got {other:?}"),
    }
}

#[test]
fn rejects_compaction_missing_n_for_seconds_trigger() {
    let yaml = "events: {}\ncompaction:\n  intermediate:\n    trigger: every_t_seconds\n";
    assert!(matches!(parse(yaml), Err(LoadError::Invalid { .. })));
}

#[test]
fn on_flush_trigger_does_not_need_n() {
    let yaml = "events: {}\ncompaction:\n  intermediate:\n    trigger: on_flush\n";
    assert!(parse(yaml).is_ok());
}

#[test]
fn rejects_compaction_zero_n() {
    let yaml = "events: {}\ncompaction:\n  intermediate:\n    trigger: every_n_commits\n    n: 0\n";
    assert!(matches!(parse(yaml), Err(LoadError::Invalid { .. })));
}
