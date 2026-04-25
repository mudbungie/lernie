//! Disk and git error paths for [`crate::prompt::run`].
//!
//! Covers the branch-life failures inside `prompt::run`: `git worktree
//! add`, the four I/O writes (worktree dir, goal, soul, step dir,
//! request, response), and each `git add` / `git commit` along the
//! two-commit flow, plus the merge-back-to-main rebase/merge/remove.
//! Compactor-internal failures (worktree add for the compactor
//! branch, summary write/commit, cmp rebase/merge/remove) live in
//! `compactor::tests` — they sit behind the [`crate::prompt::Dispatcher`]
//! boundary now, so are not reachable through `prompt::run` with a
//! stub dispatcher. Config and adapter failure paths live in
//! [`super::errors`].

use super::fixtures::*;
use crate::prompt::{Deps, Error, run};

#[test]
fn run_surfaces_worktree_add_failure() {
    // describe succeeds; `git worktree add` fails (index 0).
    let repo = scaffold_repo(VALID_PER_REPO_PROVIDERS_YAML, Some("body"));
    let adapter = StubAdapter::happy(HAPPY_RESPONSE_JSON.as_bytes());
    let err = run_with_stubs(repo.path(), "hi", &adapter, &StubGit::failing_at(0)).unwrap_err();
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
fn run_surfaces_worktree_create_failure() {
    // Pre-create the worktree path as a regular file so
    // `write_snapshot`'s create_dir_all on the worktree fails on a
    // file-not-dir component.
    let repo = scaffold_repo(VALID_PER_REPO_PROVIDERS_YAML, Some("body"));
    let wt = worktree_path(repo.path());
    std::fs::create_dir_all(wt.parent().unwrap()).unwrap();
    std::fs::write(&wt, b"blocker").unwrap();
    let adapter = StubAdapter::happy(HAPPY_RESPONSE_JSON.as_bytes());
    let err = run_with_stubs(repo.path(), "hi", &adapter, &StubGit::ok()).unwrap_err();
    assert!(matches!(err, Error::Io(_)), "got {err:?}");
}

#[test]
fn run_surfaces_goal_write_failure() {
    // Worktree dir exists but goal.md is already a directory.
    let repo = scaffold_repo(VALID_PER_REPO_PROVIDERS_YAML, Some("body"));
    let wt = worktree_path(repo.path());
    std::fs::create_dir_all(wt.join("goal.md")).unwrap();
    let adapter = StubAdapter::happy(HAPPY_RESPONSE_JSON.as_bytes());
    let err = run_with_stubs(repo.path(), "hi", &adapter, &StubGit::ok()).unwrap_err();
    assert!(matches!(err, Error::Io(_)), "got {err:?}");
}

#[test]
fn run_surfaces_soul_write_failure() {
    // Worktree dir + goal.md writeable, but soul.md is a directory.
    let repo = scaffold_repo(VALID_PER_REPO_PROVIDERS_YAML, Some("body"));
    let wt = worktree_path(repo.path());
    std::fs::create_dir_all(wt.join("soul.md")).unwrap();
    let adapter = StubAdapter::happy(HAPPY_RESPONSE_JSON.as_bytes());
    let err = run_with_stubs(repo.path(), "hi", &adapter, &StubGit::ok()).unwrap_err();
    assert!(matches!(err, Error::Io(_)), "got {err:?}");
}

#[test]
fn run_surfaces_step_dir_create_failure() {
    // Worktree dir + goal/soul writeable, but the steps/ path
    // component is a regular file so step_dir create_dir_all fails.
    let repo = scaffold_repo(VALID_PER_REPO_PROVIDERS_YAML, Some("body"));
    let wt = worktree_path(repo.path());
    std::fs::create_dir_all(&wt).unwrap();
    std::fs::write(wt.join("steps"), b"blocker").unwrap();
    let adapter = StubAdapter::happy(HAPPY_RESPONSE_JSON.as_bytes());
    let err = run_with_stubs(repo.path(), "hi", &adapter, &StubGit::ok()).unwrap_err();
    assert!(matches!(err, Error::Io(_)), "got {err:?}");
}

#[test]
fn run_surfaces_request_write_failure() {
    // Pre-create request.json as a directory so the file write fails.
    let repo = scaffold_repo(VALID_PER_REPO_PROVIDERS_YAML, Some("body"));
    let wt = worktree_path(repo.path());
    let step_dir = wt.join("steps/ct-1-deadbeef/001");
    std::fs::create_dir_all(step_dir.join("request.json")).unwrap();
    let adapter = StubAdapter::happy(HAPPY_RESPONSE_JSON.as_bytes());
    let err = run_with_stubs(repo.path(), "hi", &adapter, &StubGit::ok()).unwrap_err();
    assert!(matches!(err, Error::Io(_)), "got {err:?}");
}

#[test]
fn run_surfaces_snapshot_add_failure() {
    let repo = scaffold_repo(VALID_PER_REPO_PROVIDERS_YAML, Some("body"));
    let adapter = StubAdapter::happy(HAPPY_RESPONSE_JSON.as_bytes());
    let err = run_with_stubs(repo.path(), "hi", &adapter, &StubGit::failing_at(1)).unwrap_err();
    assert!(matches!(err, Error::Git { op: "add", .. }));
}

#[test]
fn run_surfaces_snapshot_commit_failure() {
    let repo = scaffold_repo(VALID_PER_REPO_PROVIDERS_YAML, Some("body"));
    let adapter = StubAdapter::happy(HAPPY_RESPONSE_JSON.as_bytes());
    let err = run_with_stubs(repo.path(), "hi", &adapter, &StubGit::failing_at(2)).unwrap_err();
    assert!(matches!(err, Error::Git { op: "commit", .. }));
}

#[test]
fn run_surfaces_response_write_failure() {
    let repo = scaffold_repo(VALID_PER_REPO_PROVIDERS_YAML, Some("body"));
    let wt = worktree_path(repo.path());
    let step_dir = wt.join("steps/ct-1-deadbeef/001");
    std::fs::create_dir_all(step_dir.join("response.json")).unwrap();
    let adapter = StubAdapter::happy(HAPPY_RESPONSE_JSON.as_bytes());
    let err = run_with_stubs(repo.path(), "hi", &adapter, &StubGit::ok()).unwrap_err();
    assert!(matches!(err, Error::Io(_)), "got {err:?}");
}

#[test]
fn run_surfaces_response_add_failure() {
    let repo = scaffold_repo(VALID_PER_REPO_PROVIDERS_YAML, Some("body"));
    let adapter = StubAdapter::happy(HAPPY_RESPONSE_JSON.as_bytes());
    let err = run_with_stubs(repo.path(), "hi", &adapter, &StubGit::failing_at(3)).unwrap_err();
    assert!(matches!(err, Error::Git { op: "add", .. }));
}

#[test]
fn run_surfaces_response_commit_failure() {
    let repo = scaffold_repo(VALID_PER_REPO_PROVIDERS_YAML, Some("body"));
    let adapter = StubAdapter::happy(HAPPY_RESPONSE_JSON.as_bytes());
    let err = run_with_stubs(repo.path(), "hi", &adapter, &StubGit::failing_at(4)).unwrap_err();
    assert!(matches!(err, Error::Git { op: "commit", .. }));
}

#[test]
fn run_surfaces_dispatcher_failure() {
    // Dispatcher returns an error — surfaces as DispatchFailed and
    // skips the merge-to-main step. Built inline because the helper's
    // default dispatcher is always-ok.
    let repo = scaffold_repo(VALID_PER_REPO_PROVIDERS_YAML, Some("body"));
    let harness = scaffold_harness_root();
    let adapter = StubAdapter::happy(HAPPY_RESPONSE_JSON.as_bytes());
    let git = StubGit::ok();
    let clock = FixedClock::default();
    let id = FixedIdGen;
    let dispatcher = StubDispatcher::failing(std::io::ErrorKind::Other, "lernie binary missing");
    let deps = Deps {
        adapter: &adapter,
        git: &git,
        clock: &clock,
        id_gen: &id,
        dispatcher: &dispatcher,
        harness_root: harness.path(),
    };
    let err = run(repo.path(), "hi", &deps).unwrap_err();
    assert!(
        matches!(
            err,
            Error::DispatchFailed {
                role: "compactor",
                ..
            }
        ),
        "got {err:?}"
    );
    assert_eq!(git.runs.borrow().len(), 5, "merge-to-main never starts");
}

#[test]
fn run_surfaces_merge_to_main_rebase_failure() {
    // Index 5: rebase conv onto main. Cmp internals are behind the
    // dispatcher boundary, so index 5 is the next git call after the
    // response commit.
    let repo = scaffold_repo(VALID_PER_REPO_PROVIDERS_YAML, Some("body"));
    let adapter = StubAdapter::happy(HAPPY_RESPONSE_JSON.as_bytes());
    let err = run_with_stubs(repo.path(), "hi", &adapter, &StubGit::failing_at(5)).unwrap_err();
    assert!(matches!(err, Error::Git { op: "rebase", .. }));
}

#[test]
fn run_surfaces_merge_ours_rm_failure() {
    // Index 6 is the merge=ours rm: first call after the rebase
    // succeeds. Surfaces the alignment-step error op label.
    let repo = scaffold_repo(VALID_PER_REPO_PROVIDERS_YAML, Some("body"));
    let adapter = StubAdapter::happy(HAPPY_RESPONSE_JSON.as_bytes());
    let err = run_with_stubs(repo.path(), "hi", &adapter, &StubGit::failing_at(6)).unwrap_err();
    assert!(matches!(
        err,
        Error::Git {
            op: "merge=ours rm",
            ..
        }
    ));
}

#[test]
fn run_surfaces_merge_ours_ls_tree_failure() {
    let repo = scaffold_repo(VALID_PER_REPO_PROVIDERS_YAML, Some("body"));
    let adapter = StubAdapter::happy(HAPPY_RESPONSE_JSON.as_bytes());
    let err = run_with_stubs(repo.path(), "hi", &adapter, &StubGit::failing_at(7)).unwrap_err();
    assert!(matches!(
        err,
        Error::Git {
            op: "merge=ours ls-tree",
            ..
        }
    ));
}

#[test]
fn run_surfaces_merge_ours_diff_failure() {
    // Index 8 is the diff --cached --name-only capture. With an empty
    // ls-tree the conditional checkout is skipped, so diff is the
    // next call after ls-tree.
    let repo = scaffold_repo(VALID_PER_REPO_PROVIDERS_YAML, Some("body"));
    let adapter = StubAdapter::happy(HAPPY_RESPONSE_JSON.as_bytes());
    let err = run_with_stubs(repo.path(), "hi", &adapter, &StubGit::failing_at(8)).unwrap_err();
    assert!(matches!(
        err,
        Error::Git {
            op: "merge=ours diff",
            ..
        }
    ));
}

#[test]
fn run_surfaces_merge_to_main_merge_failure() {
    // With both alignment captures returning empty (no checkout, no
    // commit), index 9 is the merge --no-ff.
    let repo = scaffold_repo(VALID_PER_REPO_PROVIDERS_YAML, Some("body"));
    let adapter = StubAdapter::happy(HAPPY_RESPONSE_JSON.as_bytes());
    let err = run_with_stubs(repo.path(), "hi", &adapter, &StubGit::failing_at(9)).unwrap_err();
    assert!(matches!(err, Error::Git { op: "merge", .. }));
}

#[test]
fn run_surfaces_conv_worktree_remove_failure() {
    let repo = scaffold_repo(VALID_PER_REPO_PROVIDERS_YAML, Some("body"));
    let adapter = StubAdapter::happy(HAPPY_RESPONSE_JSON.as_bytes());
    let err = run_with_stubs(repo.path(), "hi", &adapter, &StubGit::failing_at(10)).unwrap_err();
    assert!(
        matches!(
            err,
            Error::Git {
                op: "worktree remove",
                ..
            }
        ),
        "got {err:?}"
    );
}
