//! Disk and git error paths for [`crate::prompt::run`].
//!
//! Covers the branch-life failures: `git rev-parse main`, `git
//! worktree add`, the four I/O writes in the snapshot phase, each
//! `git add` / `git commit` along the two-commit flow, and
//! `branches.json` load/save. Config and adapter failure paths live
//! in [`super::errors`].

use super::fixtures::*;
use crate::prompt::{Error, branches, run};

#[test]
fn run_surfaces_rev_parse_failure() {
    // `git rev-parse main` is the first git call (index 0). When it
    // fails, no branch has been created yet.
    let repo = scaffold_repo(VALID_PROVIDERS_YAML, VALID_AGENTS_YAML, Some("body"));
    let adapter = StubAdapter::happy(HAPPY_RESPONSE_JSON.as_bytes());
    let git = StubGit::failing_at(0);
    let clock = FixedClock::new();
    let id = FixedIdGen;
    let err = run(repo.path(), "hi", &valid_deps(&adapter, &git, &clock, &id)).unwrap_err();
    assert!(
        matches!(
            err,
            Error::Git {
                op: "rev-parse",
                ..
            }
        ),
        "got {err:?}"
    );
}

#[test]
fn run_surfaces_worktree_add_failure() {
    // rev-parse succeeds; `git worktree add` fails (index 1).
    let repo = scaffold_repo(VALID_PROVIDERS_YAML, VALID_AGENTS_YAML, Some("body"));
    let adapter = StubAdapter::happy(HAPPY_RESPONSE_JSON.as_bytes());
    let git = StubGit::failing_at(1);
    let clock = FixedClock::new();
    let id = FixedIdGen;
    let err = run(repo.path(), "hi", &valid_deps(&adapter, &git, &clock, &id)).unwrap_err();
    assert!(
        matches!(
            err,
            Error::Git {
                op: "worktree add",
                ..
            }
        ),
        "got {err:?}"
    );
}

#[test]
fn run_surfaces_branches_save_failure() {
    // Pre-create `.agent/state/branches.json` as a directory so the
    // atomic rename over it fails.
    let repo = scaffold_repo(VALID_PROVIDERS_YAML, VALID_AGENTS_YAML, Some("body"));
    std::fs::create_dir_all(branches::path(repo.path())).unwrap();
    let adapter = StubAdapter::happy(HAPPY_RESPONSE_JSON.as_bytes());
    let git = StubGit::ok();
    let clock = FixedClock::new();
    let id = FixedIdGen;
    let err = run(repo.path(), "hi", &valid_deps(&adapter, &git, &clock, &id)).unwrap_err();
    assert!(matches!(err, Error::Io(_)), "got {err:?}");
}

#[test]
fn run_surfaces_agent_dir_create_failure() {
    // Pre-create the worktree path as a regular file so
    // `write_snapshot`'s create_dir_all fails on a file-not-dir
    // component.
    let repo = scaffold_repo(VALID_PROVIDERS_YAML, VALID_AGENTS_YAML, Some("body"));
    let wt = worktree_path(repo.path());
    std::fs::create_dir_all(wt.parent().unwrap()).unwrap();
    std::fs::write(&wt, b"blocker").unwrap();
    let adapter = StubAdapter::happy(HAPPY_RESPONSE_JSON.as_bytes());
    let git = StubGit::ok();
    let clock = FixedClock::new();
    let id = FixedIdGen;
    let err = run(repo.path(), "hi", &valid_deps(&adapter, &git, &clock, &id)).unwrap_err();
    assert!(matches!(err, Error::Io(_)), "got {err:?}");
}

#[test]
fn run_surfaces_goal_write_failure() {
    // Agent dir exists but goal.md is already a directory.
    let repo = scaffold_repo(VALID_PROVIDERS_YAML, VALID_AGENTS_YAML, Some("body"));
    let wt = worktree_path(repo.path());
    std::fs::create_dir_all(wt.join(".agent/goal.md")).unwrap();
    let adapter = StubAdapter::happy(HAPPY_RESPONSE_JSON.as_bytes());
    let git = StubGit::ok();
    let clock = FixedClock::new();
    let id = FixedIdGen;
    let err = run(repo.path(), "hi", &valid_deps(&adapter, &git, &clock, &id)).unwrap_err();
    assert!(matches!(err, Error::Io(_)), "got {err:?}");
}

#[test]
fn run_surfaces_step_dir_create_failure() {
    // Agent dir + goal.md writeable, but the exchanges/ path
    // component is a regular file so step_dir create_dir_all fails.
    let repo = scaffold_repo(VALID_PROVIDERS_YAML, VALID_AGENTS_YAML, Some("body"));
    let wt = worktree_path(repo.path());
    std::fs::create_dir_all(wt.join(".agent")).unwrap();
    std::fs::write(wt.join("exchanges"), b"blocker").unwrap();
    let adapter = StubAdapter::happy(HAPPY_RESPONSE_JSON.as_bytes());
    let git = StubGit::ok();
    let clock = FixedClock::new();
    let id = FixedIdGen;
    let err = run(repo.path(), "hi", &valid_deps(&adapter, &git, &clock, &id)).unwrap_err();
    assert!(matches!(err, Error::Io(_)), "got {err:?}");
}

#[test]
fn run_surfaces_request_write_failure() {
    // Pre-create request.json as a directory so the file write fails.
    let repo = scaffold_repo(VALID_PROVIDERS_YAML, VALID_AGENTS_YAML, Some("body"));
    let wt = worktree_path(repo.path());
    let step_dir = wt.join("exchanges/ct-1-deadbeef/steps/001");
    std::fs::create_dir_all(step_dir.join("request.json")).unwrap();
    let adapter = StubAdapter::happy(HAPPY_RESPONSE_JSON.as_bytes());
    let git = StubGit::ok();
    let clock = FixedClock::new();
    let id = FixedIdGen;
    let err = run(repo.path(), "hi", &valid_deps(&adapter, &git, &clock, &id)).unwrap_err();
    assert!(matches!(err, Error::Io(_)), "got {err:?}");
}

#[test]
fn run_surfaces_snapshot_add_failure() {
    // `git add` of the snapshot files fails (index 2).
    let repo = scaffold_repo(VALID_PROVIDERS_YAML, VALID_AGENTS_YAML, Some("body"));
    let adapter = StubAdapter::happy(HAPPY_RESPONSE_JSON.as_bytes());
    let git = StubGit::failing_at(2);
    let clock = FixedClock::new();
    let id = FixedIdGen;
    let err = run(repo.path(), "hi", &valid_deps(&adapter, &git, &clock, &id)).unwrap_err();
    assert!(matches!(err, Error::Git { op: "add", .. }));
}

#[test]
fn run_surfaces_snapshot_commit_failure() {
    // `git commit` of the snapshot fails (index 3).
    let repo = scaffold_repo(VALID_PROVIDERS_YAML, VALID_AGENTS_YAML, Some("body"));
    let adapter = StubAdapter::happy(HAPPY_RESPONSE_JSON.as_bytes());
    let git = StubGit::failing_at(3);
    let clock = FixedClock::new();
    let id = FixedIdGen;
    let err = run(repo.path(), "hi", &valid_deps(&adapter, &git, &clock, &id)).unwrap_err();
    assert!(matches!(err, Error::Git { op: "commit", .. }));
}

#[test]
fn run_surfaces_response_write_failure() {
    // Pre-create response.json as a directory so the post-complete
    // write fails. The snapshot commit is already ok.
    let repo = scaffold_repo(VALID_PROVIDERS_YAML, VALID_AGENTS_YAML, Some("body"));
    let wt = worktree_path(repo.path());
    let step_dir = wt.join("exchanges/ct-1-deadbeef/steps/001");
    std::fs::create_dir_all(step_dir.join("response.json")).unwrap();
    let adapter = StubAdapter::happy(HAPPY_RESPONSE_JSON.as_bytes());
    let git = StubGit::ok();
    let clock = FixedClock::new();
    let id = FixedIdGen;
    let err = run(repo.path(), "hi", &valid_deps(&adapter, &git, &clock, &id)).unwrap_err();
    assert!(matches!(err, Error::Io(_)), "got {err:?}");
}

#[test]
fn run_surfaces_response_add_failure() {
    // `git add` of the response file fails (index 4).
    let repo = scaffold_repo(VALID_PROVIDERS_YAML, VALID_AGENTS_YAML, Some("body"));
    let adapter = StubAdapter::happy(HAPPY_RESPONSE_JSON.as_bytes());
    let git = StubGit::failing_at(4);
    let clock = FixedClock::new();
    let id = FixedIdGen;
    let err = run(repo.path(), "hi", &valid_deps(&adapter, &git, &clock, &id)).unwrap_err();
    assert!(matches!(err, Error::Git { op: "add", .. }));
}

#[test]
fn run_surfaces_response_commit_failure() {
    // `git commit` of the response fails (index 5).
    let repo = scaffold_repo(VALID_PROVIDERS_YAML, VALID_AGENTS_YAML, Some("body"));
    let adapter = StubAdapter::happy(HAPPY_RESPONSE_JSON.as_bytes());
    let git = StubGit::failing_at(5);
    let clock = FixedClock::new();
    let id = FixedIdGen;
    let err = run(repo.path(), "hi", &valid_deps(&adapter, &git, &clock, &id)).unwrap_err();
    assert!(matches!(err, Error::Git { op: "commit", .. }));
}
