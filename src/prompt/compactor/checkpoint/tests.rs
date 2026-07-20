//! Tests for checkpoint trigger evaluation (ARCH §6).
//!
//! [`due`] is pure — driven by constructed states. [`state`]'s git
//! derivation runs against a real repo so the last-checkpoint grep and the
//! commit-count / elapsed-time measures are exercised end-to-end.

use super::*;
use crate::config::workflow::{CompactionConfig, IntermediateCompaction};
use crate::template::RealGit;
use tempfile::TempDir;

fn cfg(trigger: CompactionTrigger, n: Option<u32>) -> CompactionConfig {
    CompactionConfig {
        intermediate: IntermediateCompaction { trigger, n },
    }
}

fn st(commits: u32, seconds: u64, flush: bool) -> CheckpointState {
    CheckpointState {
        commits_since_checkpoint: commits,
        seconds_since_checkpoint: seconds,
        flush_requested: flush,
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

// ---- real-git state derivation ---------------------------------------

fn init(wt: &Path) {
    let g = RealGit::new();
    g.run(wt, &["init", "-b", "agents/p1"]).unwrap();
    g.run(wt, &["config", "user.email", "t@t"]).unwrap();
    g.run(wt, &["config", "user.name", "t"]).unwrap();
}

fn commit(wt: &Path, subject: &str, rel: &str, content: &str) {
    let g = RealGit::new();
    let f = wt.join(rel);
    std::fs::create_dir_all(f.parent().unwrap()).unwrap();
    std::fs::write(&f, content).unwrap();
    g.run(wt, &["add", "-A"]).unwrap();
    g.run(wt, &["commit", "-m", subject]).unwrap();
}

fn now_of(wt: &Path) -> u64 {
    RealGit::new()
        .run_capture(wt, &["log", "-n", "1", "--format=%ct", "HEAD"])
        .unwrap()
        .trim()
        .parse()
        .unwrap()
}

#[test]
fn state_counts_the_whole_branch_when_no_checkpoint_landed() {
    let dir = TempDir::new().unwrap();
    let wt = dir.path();
    init(wt);
    commit(wt, "root", "a.txt", "1");
    commit(wt, "step", "b.txt", "2");
    let s = state(wt, now_of(wt), false, &RealGit::new()).unwrap();
    assert_eq!(s.commits_since_checkpoint, 2, "root + one step");
    // now == the HEAD commit time; the root is the same wall second, so
    // elapsed rounds to ~0 (≤1 guards a second-boundary straddle).
    assert!(s.seconds_since_checkpoint <= 1);
    assert!(!s.flush_requested);
}

#[test]
fn state_measures_from_the_last_compaction_merge() {
    let dir = TempDir::new().unwrap();
    let wt = dir.path();
    init(wt);
    commit(wt, "root", "a.txt", "1");
    commit(wt, "compaction merge [p1-cmp]", "summary/001.md", "x");
    commit(wt, "step after", "b.txt", "2");
    let s = state(wt, now_of(wt) + 42, true, &RealGit::new()).unwrap();
    assert_eq!(s.commits_since_checkpoint, 1, "only the post-merge step");
    // Elapsed is measured from the checkpoint commit, not the root.
    assert!(s.seconds_since_checkpoint >= 42);
    assert!(s.flush_requested);
}

#[test]
fn state_saturates_when_now_precedes_the_checkpoint() {
    let dir = TempDir::new().unwrap();
    let wt = dir.path();
    init(wt);
    commit(wt, "root", "a.txt", "1");
    let s = state(wt, 0, false, &RealGit::new()).unwrap();
    assert_eq!(s.seconds_since_checkpoint, 0, "no negative elapsed time");
}

#[test]
fn state_surfaces_a_git_failure() {
    // A non-repo directory: the first rev-parse/log fails loudly.
    let dir = TempDir::new().unwrap();
    let err = state(dir.path(), 0, false, &RealGit::new()).unwrap_err();
    assert!(matches!(err, Error::Git { .. }), "{err:?}");
}
