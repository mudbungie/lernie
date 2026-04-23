//! Disk and git error paths for [`crate::prompt::run`].
//!
//! Covers the branch-life failures: `git worktree add`, the four I/O
//! writes (agent dir, goal, step dir, request, response), and each
//! `git add` / `git commit` along the two-commit flow. Config and
//! adapter failure paths live in [`super::errors`].

use super::fixtures::*;
use crate::prompt::{Error, run};

#[test]
fn run_surfaces_worktree_add_failure() {
    // describe succeeds; `git worktree add` fails (index 0).
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
                op: "worktree add",
                ..
            }
        ),
        "got {err:?}"
    );
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
    // `git add` of the snapshot files (index 1) fails.
    let repo = scaffold_repo(VALID_PROVIDERS_YAML, VALID_AGENTS_YAML, Some("body"));
    let adapter = StubAdapter::happy(HAPPY_RESPONSE_JSON.as_bytes());
    let git = StubGit::failing_at(1);
    let clock = FixedClock::new();
    let id = FixedIdGen;
    let err = run(repo.path(), "hi", &valid_deps(&adapter, &git, &clock, &id)).unwrap_err();
    assert!(matches!(err, Error::Git { op: "add", .. }));
}

#[test]
fn run_surfaces_snapshot_commit_failure() {
    // `git commit` of the snapshot (index 2) fails.
    let repo = scaffold_repo(VALID_PROVIDERS_YAML, VALID_AGENTS_YAML, Some("body"));
    let adapter = StubAdapter::happy(HAPPY_RESPONSE_JSON.as_bytes());
    let git = StubGit::failing_at(2);
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
    // `git add` of the response file (index 3) fails.
    let repo = scaffold_repo(VALID_PROVIDERS_YAML, VALID_AGENTS_YAML, Some("body"));
    let adapter = StubAdapter::happy(HAPPY_RESPONSE_JSON.as_bytes());
    let git = StubGit::failing_at(3);
    let clock = FixedClock::new();
    let id = FixedIdGen;
    let err = run(repo.path(), "hi", &valid_deps(&adapter, &git, &clock, &id)).unwrap_err();
    assert!(matches!(err, Error::Git { op: "add", .. }));
}

#[test]
fn run_surfaces_response_commit_failure() {
    // `git commit` of the response (index 4) fails.
    let repo = scaffold_repo(VALID_PROVIDERS_YAML, VALID_AGENTS_YAML, Some("body"));
    let adapter = StubAdapter::happy(HAPPY_RESPONSE_JSON.as_bytes());
    let git = StubGit::failing_at(4);
    let clock = FixedClock::new();
    let id = FixedIdGen;
    let err = run(repo.path(), "hi", &valid_deps(&adapter, &git, &clock, &id)).unwrap_err();
    assert!(matches!(err, Error::Git { op: "commit", .. }));
}

#[test]
fn run_surfaces_compactor_branch_spawn_failure() {
    // index 5: worktree add -b inv/<ex-id>/<cmp-id>
    let repo = scaffold_repo(VALID_PROVIDERS_YAML, VALID_AGENTS_YAML, Some("body"));
    let adapter = StubAdapter::happy(HAPPY_RESPONSE_JSON.as_bytes());
    let git = StubGit::failing_at(5);
    let clock = FixedClock::new();
    let id = FixedIdGen;
    let err = run(repo.path(), "hi", &valid_deps(&adapter, &git, &clock, &id)).unwrap_err();
    assert!(matches!(
        err,
        Error::Git {
            op: "worktree add",
            ..
        }
    ));
}

#[test]
fn run_surfaces_compactor_summary_add_failure() {
    // index 6: git add .agent/compactions/001.md in cmp wt
    let repo = scaffold_repo(VALID_PROVIDERS_YAML, VALID_AGENTS_YAML, Some("body"));
    let adapter = StubAdapter::happy(HAPPY_RESPONSE_JSON.as_bytes());
    let git = StubGit::failing_at(6);
    let clock = FixedClock::new();
    let id = FixedIdGen;
    let err = run(repo.path(), "hi", &valid_deps(&adapter, &git, &clock, &id)).unwrap_err();
    assert!(matches!(err, Error::Git { op: "add", .. }));
}

#[test]
fn run_surfaces_compactor_summary_commit_failure() {
    // index 7: git commit of summary in cmp wt
    let repo = scaffold_repo(VALID_PROVIDERS_YAML, VALID_AGENTS_YAML, Some("body"));
    let adapter = StubAdapter::happy(HAPPY_RESPONSE_JSON.as_bytes());
    let git = StubGit::failing_at(7);
    let clock = FixedClock::new();
    let id = FixedIdGen;
    let err = run(repo.path(), "hi", &valid_deps(&adapter, &git, &clock, &id)).unwrap_err();
    assert!(matches!(err, Error::Git { op: "commit", .. }));
}

#[test]
fn run_surfaces_compactor_rebase_failure() {
    // index 8: rebase cmp onto ex
    let repo = scaffold_repo(VALID_PROVIDERS_YAML, VALID_AGENTS_YAML, Some("body"));
    let adapter = StubAdapter::happy(HAPPY_RESPONSE_JSON.as_bytes());
    let git = StubGit::failing_at(8);
    let clock = FixedClock::new();
    let id = FixedIdGen;
    let err = run(repo.path(), "hi", &valid_deps(&adapter, &git, &clock, &id)).unwrap_err();
    assert!(matches!(err, Error::Git { op: "rebase", .. }));
}

#[test]
fn run_surfaces_compactor_merge_failure() {
    // index 9: merge --no-ff cmp into ex (the abort from index 8 is
    // not hit here because rebase ok; merge is the next call and ok
    // too; the index shifts). Actually with failing_at(9), only call
    // 9 fails, so rebase ok, merge fails.
    let repo = scaffold_repo(VALID_PROVIDERS_YAML, VALID_AGENTS_YAML, Some("body"));
    let adapter = StubAdapter::happy(HAPPY_RESPONSE_JSON.as_bytes());
    let git = StubGit::failing_at(9);
    let clock = FixedClock::new();
    let id = FixedIdGen;
    let err = run(repo.path(), "hi", &valid_deps(&adapter, &git, &clock, &id)).unwrap_err();
    assert!(matches!(err, Error::Git { op: "merge", .. }));
}

#[test]
fn run_surfaces_compactor_worktree_remove_failure() {
    // index 10: worktree remove cmp
    let repo = scaffold_repo(VALID_PROVIDERS_YAML, VALID_AGENTS_YAML, Some("body"));
    let adapter = StubAdapter::happy(HAPPY_RESPONSE_JSON.as_bytes());
    let git = StubGit::failing_at(10);
    let clock = FixedClock::new();
    let id = FixedIdGen;
    let err = run(repo.path(), "hi", &valid_deps(&adapter, &git, &clock, &id)).unwrap_err();
    assert!(matches!(
        err,
        Error::Git {
            op: "worktree remove",
            ..
        }
    ));
}

#[test]
fn run_surfaces_merge_to_main_rebase_failure() {
    // index 11: rebase ex onto main
    let repo = scaffold_repo(VALID_PROVIDERS_YAML, VALID_AGENTS_YAML, Some("body"));
    let adapter = StubAdapter::happy(HAPPY_RESPONSE_JSON.as_bytes());
    let git = StubGit::failing_at(11);
    let clock = FixedClock::new();
    let id = FixedIdGen;
    let err = run(repo.path(), "hi", &valid_deps(&adapter, &git, &clock, &id)).unwrap_err();
    assert!(matches!(err, Error::Git { op: "rebase", .. }));
}

#[test]
fn run_surfaces_merge_to_main_merge_failure() {
    // index 12: merge --no-ff ex into main
    let repo = scaffold_repo(VALID_PROVIDERS_YAML, VALID_AGENTS_YAML, Some("body"));
    let adapter = StubAdapter::happy(HAPPY_RESPONSE_JSON.as_bytes());
    let git = StubGit::failing_at(12);
    let clock = FixedClock::new();
    let id = FixedIdGen;
    let err = run(repo.path(), "hi", &valid_deps(&adapter, &git, &clock, &id)).unwrap_err();
    assert!(matches!(err, Error::Git { op: "merge", .. }));
}

#[test]
fn run_surfaces_ex_worktree_remove_failure() {
    // index 13: worktree remove ex
    let repo = scaffold_repo(VALID_PROVIDERS_YAML, VALID_AGENTS_YAML, Some("body"));
    let adapter = StubAdapter::happy(HAPPY_RESPONSE_JSON.as_bytes());
    let git = StubGit::failing_at(13);
    let clock = FixedClock::new();
    let id = FixedIdGen;
    let err = run(repo.path(), "hi", &valid_deps(&adapter, &git, &clock, &id)).unwrap_err();
    assert!(matches!(
        err,
        Error::Git {
            op: "worktree remove",
            ..
        }
    ));
}

