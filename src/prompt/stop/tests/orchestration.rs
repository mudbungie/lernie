//! Happy-path orchestration tests for [`crate::prompt::stop::run`].

use super::fixtures::{NoopGit, StubFinder, StubInspector, touch_step_response};
use crate::prompt::stop::{Error, cascade, run};
use std::time::Duration;
use tempfile::TempDir;

#[test]
fn run_returns_branch_missing_when_inspector_says_no() {
    let dir = TempDir::new().unwrap();
    let inspector = StubInspector {
        exists: false,
        merged: false,
    };
    let err = run(
        dir.path(),
        "br",
        &inspector,
        &StubFinder::default(),
        &cascade::RecordingSignaler::new(0),
        Duration::from_millis(1),
        &NoopGit,
    )
    .unwrap_err();
    matches!(err, Error::BranchMissing(b) if b == "br");
}

#[test]
fn run_returns_already_merged_when_inspector_says_so() {
    let dir = TempDir::new().unwrap();
    let inspector = StubInspector {
        exists: true,
        merged: true,
    };
    let err = run(
        dir.path(),
        "br",
        &inspector,
        &StubFinder::default(),
        &cascade::RecordingSignaler::new(0),
        Duration::from_millis(1),
        &NoopGit,
    )
    .unwrap_err();
    matches!(err, Error::AlreadyMerged(b) if b == "br");
}

#[test]
fn run_idempotent_when_no_writers_found() {
    let dir = TempDir::new().unwrap();
    touch_step_response(dir.path(), "br", 1);
    let inspector = StubInspector {
        exists: true,
        merged: false,
    };
    let signaler = cascade::RecordingSignaler::new(0);
    run(
        dir.path(),
        "br",
        &inspector,
        &StubFinder::default(),
        &signaler,
        Duration::from_millis(1),
        &NoopGit,
    )
    .unwrap();
    assert!(
        signaler.took().is_empty(),
        "no writer → no signals sent (idempotent)"
    );
}

#[test]
fn run_idempotent_when_no_steps_dir_at_all() {
    let dir = TempDir::new().unwrap();
    let inspector = StubInspector {
        exists: true,
        merged: false,
    };
    let signaler = cascade::RecordingSignaler::new(0);
    run(
        dir.path(),
        "br",
        &inspector,
        &StubFinder::default(),
        &signaler,
        Duration::from_millis(1),
        &NoopGit,
    )
    .unwrap();
    assert!(signaler.took().is_empty());
}

#[test]
fn run_picks_highest_numbered_step() {
    let dir = TempDir::new().unwrap();
    let _early = touch_step_response(dir.path(), "br", 1);
    let latest = touch_step_response(dir.path(), "br", 7);
    let _mid = touch_step_response(dir.path(), "br", 3);
    let inspector = StubInspector {
        exists: true,
        merged: false,
    };
    let finder = StubFinder::with_returns(vec![Some(123)]);
    let signaler = cascade::RecordingSignaler::new(0);
    run(
        dir.path(),
        "br",
        &inspector,
        &finder,
        &signaler,
        Duration::from_millis(1),
        &NoopGit,
    )
    .unwrap();
    assert_eq!(finder.seen.lock().unwrap().as_slice(), &[latest]);
    assert_eq!(signaler.took(), vec![("term", 123)]);
}

#[test]
fn run_covers_descended_subagent_conv_ids_via_hyphen_prefix() {
    let dir = TempDir::new().unwrap();
    touch_step_response(dir.path(), "br", 1);
    touch_step_response(dir.path(), "br-sub", 2);
    let inspector = StubInspector {
        exists: true,
        merged: false,
    };
    let finder = StubFinder::with_returns(vec![Some(11), Some(22)]);
    let signaler = cascade::RecordingSignaler::new(0);
    run(
        dir.path(),
        "br",
        &inspector,
        &finder,
        &signaler,
        Duration::from_millis(1),
        &NoopGit,
    )
    .unwrap();
    let mut targets: Vec<i32> = signaler
        .took()
        .into_iter()
        .filter(|(s, _)| *s == "term")
        .map(|(_, t)| t)
        .collect();
    targets.sort();
    assert_eq!(targets, vec![11, 22]);
}

#[test]
fn run_dedupes_pgid_when_multiple_writers_share_one() {
    let dir = TempDir::new().unwrap();
    touch_step_response(dir.path(), "br", 1);
    touch_step_response(dir.path(), "br-sub", 2);
    let inspector = StubInspector {
        exists: true,
        merged: false,
    };
    let finder = StubFinder::with_returns(vec![Some(7), Some(7)]);
    let signaler = cascade::RecordingSignaler::new(0);
    run(
        dir.path(),
        "br",
        &inspector,
        &finder,
        &signaler,
        Duration::from_millis(1),
        &NoopGit,
    )
    .unwrap();
    assert_eq!(signaler.took(), vec![("term", 7)]);
}

#[test]
fn run_skips_unrelated_conv_id_branches() {
    let dir = TempDir::new().unwrap();
    touch_step_response(dir.path(), "br", 1);
    touch_step_response(dir.path(), "different", 1);
    touch_step_response(dir.path(), "br2-not-descended", 1);
    let inspector = StubInspector {
        exists: true,
        merged: false,
    };
    let finder = StubFinder::with_returns(vec![Some(99)]);
    let signaler = cascade::RecordingSignaler::new(0);
    run(
        dir.path(),
        "br",
        &inspector,
        &finder,
        &signaler,
        Duration::from_millis(1),
        &NoopGit,
    )
    .unwrap();
    let seen = finder.seen.lock().unwrap().clone();
    assert_eq!(seen.len(), 1, "only `br` should match: got {seen:?}");
    assert!(seen[0].to_string_lossy().contains("/steps/br/"));
}
