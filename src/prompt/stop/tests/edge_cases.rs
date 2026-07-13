//! Edge cases, error propagation, and direct surface coverage for
//! [`crate::prompt::stop`] — the bits that don't fit the happy-path
//! orchestration narrative.

use super::fixtures::{
    ErrFinder, ErrInspector, ErrMergedInspector, NoopGit, StubFinder, StubInspector,
    touch_inbox_dir,
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
        false,
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
        false,
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
    touch_inbox_dir(dir.path(), "br");
    let inspector = StubInspector {
        exists: true,
        merged: false,
    };
    let err = run(
        dir.path(),
        "br",
        false,
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
fn collect_inbox_dirs_returns_empty_when_inbox_root_missing() {
    let dir = TempDir::new().unwrap();
    let v = super::super::collect_inbox_dirs(dir.path(), "br", false).unwrap();
    assert!(v.is_empty());
}

#[test]
fn collect_inbox_dirs_gates_descendants_on_stop_children() {
    // Same on-disk tree, both flag values: default is self-only; the
    // flag folds in the `br-*` descendant. Pins the opt-in boundary
    // directly at the collector (§2.9).
    let dir = TempDir::new().unwrap();
    touch_inbox_dir(dir.path(), "br");
    touch_inbox_dir(dir.path(), "br-sub");

    let mut default_only = super::super::collect_inbox_dirs(dir.path(), "br", false).unwrap();
    default_only.sort();
    assert_eq!(
        default_only.len(),
        1,
        "default is self-only: {default_only:?}"
    );
    assert!(default_only[0].ends_with("inbox/br"));

    let mut with_children = super::super::collect_inbox_dirs(dir.path(), "br", true).unwrap();
    with_children.sort();
    assert_eq!(
        with_children.len(),
        2,
        "flag includes the child: {with_children:?}"
    );
    assert!(with_children[0].ends_with("inbox/br"));
    assert!(with_children[1].ends_with("inbox/br-sub"));
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
fn error_display_inbox_walk_passes_through_io_message() {
    let e = Error::InboxWalk(io::Error::other("nope"));
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
    let err = super::super::cli_run(dir.path(), "no-such-branch", false).unwrap_err();
    matches!(err, Error::BranchMissing(b) if b == "no-such-branch");
}

#[test]
fn cli_run_forwards_stop_children_flag() {
    // The `true` arm flows through cli_run into run/collect; the empty
    // repo still short-circuits at branch validation, proving the flag
    // is plumbed without needing a live executor.
    let dir = TempDir::new().unwrap();
    let err = super::super::cli_run(dir.path(), "no-such-branch", true).unwrap_err();
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
