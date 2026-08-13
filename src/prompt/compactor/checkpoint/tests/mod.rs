//! Tests for checkpoint trigger evaluation (ARCH §6).
//!
//! [`due`] is pure — driven by constructed states, and that is this
//! module. [`state`]'s git derivation runs against a real repo in
//! [`derive`], so the origin grep and the commit-count / elapsed-time
//! measures are exercised end-to-end.

use super::*;
use crate::config::workflow::{CompactionConfig, compaction::IntermediateCompaction};

mod derive;

fn cfg(trigger: CompactionTrigger, n: Option<u32>) -> CompactionConfig {
    CompactionConfig {
        intermediate: IntermediateCompaction {
            trigger,
            n,
            keep_recent: None,
        },
    }
}

fn st(commits: u32, seconds: u64, flush: bool) -> CheckpointState {
    CheckpointState {
        commits_since_checkpoint: commits,
        seconds_since_checkpoint: seconds,
        flush_requested: flush,
        is_compactor: false,
    }
}

#[test]
fn no_config_never_compacts() {
    assert!(!due(None, &st(1000, 1000, true)));
}

#[test]
fn every_n_commits_fires_at_or_past_the_threshold() {
    let c = cfg(CompactionTrigger::EveryNCommits, Some(3));
    assert!(!due(Some(&c), &st(2, 0, false)));
    assert!(due(Some(&c), &st(3, 0, false)));
    assert!(due(Some(&c), &st(4, 0, false)));
}

#[test]
fn every_t_seconds_fires_at_or_past_the_threshold() {
    let c = cfg(CompactionTrigger::EveryTSeconds, Some(10));
    assert!(!due(Some(&c), &st(0, 9, false)));
    assert!(due(Some(&c), &st(0, 10, false)));
}

#[test]
fn on_flush_fires_only_when_the_agent_elects_it() {
    let c = cfg(CompactionTrigger::OnFlush, None);
    assert!(!due(Some(&c), &st(9999, 9999, false)));
    assert!(due(Some(&c), &st(0, 0, true)));
}

#[test]
fn a_malformed_threshold_fails_closed() {
    // n absent or zero (guarded at config load, §6) is never due — a bad
    // config does not compact every step.
    assert!(!due(
        Some(&cfg(CompactionTrigger::EveryNCommits, None)),
        &st(100, 0, false)
    ));
    assert!(!due(
        Some(&cfg(CompactionTrigger::EveryTSeconds, Some(0))),
        &st(0, 100, false)
    ));
}

#[test]
fn a_compactor_is_never_compaction_eligible() {
    // The invariant: a compactor *is* the compaction, not a subject of
    // one (§2.7). No trigger, at any count/elapsed/elected flush, admits
    // it to the eligible set — this is what stops a compactor from
    // dispatching a compactor (bl-a9eb / yog bl-ebbd).
    let compactor = CheckpointState {
        is_compactor: true,
        ..st(9999, 9999, true)
    };
    for c in [
        cfg(CompactionTrigger::EveryNCommits, Some(1)),
        cfg(CompactionTrigger::EveryTSeconds, Some(1)),
        cfg(CompactionTrigger::OnFlush, None),
    ] {
        assert!(!due(Some(&c), &compactor), "{:?}", c.intermediate.trigger);
        // The same state on a non-compactor branch *is* due, so the
        // exclusion is the only thing suppressing it.
        assert!(due(Some(&c), &st(9999, 9999, true)));
    }
}
