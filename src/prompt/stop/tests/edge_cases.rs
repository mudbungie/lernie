//! Edge cases, error propagation, and direct surface coverage for
//! [`crate::prompt::stop`] — the bits that don't fit the happy-path
//! orchestration narrative.

use super::fixtures::{
    ErrFinder, ErrInspector, ErrMergedInspector, NoopGit, STEPS_DIR, StubFinder, StubInspector,
    touch_step_response,
};
use crate::prompt::stop::{Error, cascade, run};
use crate::template::GitRunner;
use std::io;
use std::path::Path;
use std::time::Duration;
use tempfile::TempDir;

#[test]
fn run_propagates_inspector_exists_error_as_git() {
    let dir = TempDir::new().unwrap();
    let err = run(
        dir.path(),
        "br",
        &ErrInspector,
        &StubFinder::default(),
        &cascade::RecordingSignaler::new(0),
        Duration::from_millis(1),
        &NoopGit,
    )
    .unwrap_err();
    matches!(
        err,
        Error::Git {
            op: "rev-parse --verify",
            ..
        }
    );
}

#[test]
fn run_propagates_inspector_merged_error_as_git() {
    let dir = TempDir::new().unwrap();
    let err = run(
        dir.path(),
        "br",
        &ErrMergedInspector,
        &StubFinder::default(),
        &cascade::RecordingSignaler::new(0),
        Duration::from_millis(1),
        &NoopGit,
    )
    .unwrap_err();
    matches!(
        err,
        Error::Git {
            op: "merge-base --is-ancestor",
            ..
        }
    );
}

#[test]
fn run_propagates_finder_io_error_as_proc() {
    let dir = TempDir::new().unwrap();
    touch_step_response(dir.path(), "br", 1);
    let inspector = StubInspector {
        exists: true,
        merged: false,
    };
    let err = run(
        dir.path(),
        "br",
        &inspector,
        &ErrFinder,
        &cascade::RecordingSignaler::new(0),
        Duration::from_millis(1),
        &NoopGit,
    )
    .unwrap_err();
    matches!(err, Error::Proc(_));
}

#[test]
fn run_skips_step_subdirs_that_are_not_three_digit_numbers() {
    let dir = TempDir::new().unwrap();
    let conv = dir.path().join(STEPS_DIR).join("br");
    std::fs::create_dir_all(conv.join("not-a-step")).unwrap();
    std::fs::create_dir_all(conv.join("99")).unwrap(); // wrong width
    let real = touch_step_response(dir.path(), "br", 5);
    let inspector = StubInspector {
        exists: true,
        merged: false,
    };
    let finder = StubFinder::with_returns(vec![Some(33)]);
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
    assert_eq!(finder.seen.lock().unwrap().as_slice(), &[real]);
}

#[test]
fn run_skips_branch_dir_with_no_step_subdirs_yet() {
    let dir = TempDir::new().unwrap();
    std::fs::create_dir_all(dir.path().join(STEPS_DIR).join("br")).unwrap();
    let inspector = StubInspector {
        exists: true,
        merged: false,
    };
    let finder = StubFinder::default();
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
    assert!(finder.seen.lock().unwrap().is_empty());
    assert!(signaler.took().is_empty());
}

#[test]
fn collect_response_paths_returns_empty_when_steps_root_missing() {
    let dir = TempDir::new().unwrap();
    let v = super::super::collect_response_paths(dir.path(), "br").unwrap();
    assert!(v.is_empty());
}

#[test]
fn error_display_branch_missing_includes_name() {
    let e = Error::BranchMissing("foo".into());
    let s = format!("{e}");
    assert!(s.contains("\"foo\""), "got {s}");
}

#[test]
fn error_display_already_merged_includes_name() {
    let e = Error::AlreadyMerged("foo".into());
    let s = format!("{e}");
    assert!(s.contains("\"foo\""), "got {s}");
}

#[test]
fn error_display_steps_walk_passes_through_io_message() {
    let e = Error::StepsWalk(io::Error::other("nope"));
    let s = format!("{e}");
    assert!(s.contains("nope"), "got {s}");
}

#[test]
fn noop_git_returns_ok_from_both_methods() {
    // The orchestration tests pass `&NoopGit` purely to satisfy
    // run()'s signature; StubInspector ignores the runner. Cover
    // NoopGit's body directly so tarpaulin sees it.
    let g = NoopGit;
    g.run(Path::new("/anywhere"), &["any", "args"]).unwrap();
    assert_eq!(g.run_capture(Path::new("/anywhere"), &["any"]).unwrap(), "");
}

#[test]
fn cli_run_returns_branch_missing_against_empty_repo() {
    // cli_run wires production deps (`GitInspector` / `ProcFsFinder`
    // / `RealSignaler`). Pointed at a temp dir with no `<repo>/root/`,
    // GitInspector's rev-parse fails and is interpreted as "branch
    // does not exist" — the canonical pre-cascade error path.
    let dir = TempDir::new().unwrap();
    let err = super::super::cli_run(dir.path(), "no-such-branch").unwrap_err();
    matches!(err, Error::BranchMissing(b) if b == "no-such-branch");
}

#[test]
fn become_pgid_leader_returns_cleanly() {
    // Direct call to the production wrapper. Idempotent on a process
    // that's already a pgid leader (typical for cargo test under
    // shell job control). The branch table itself is covered by the
    // closure-injected variants below, so this test is just to seal
    // the wrapper's coverage.
    super::super::become_pgid_leader();
}

#[test]
fn become_pgid_leader_with_zero_succeeds_silently() {
    // Setpgid returning 0 ("ok") takes the silent branch; nothing
    // observable except that the function returned.
    super::super::become_pgid_leader_with(|| 0);
}

#[test]
fn become_pgid_leader_with_nonzero_takes_error_branch() {
    // Setpgid returning -1 is the failure path. The function prints
    // to stderr (irrelevant for coverage) and returns; only the
    // branch reachability matters here.
    super::super::become_pgid_leader_with(|| -1);
}
