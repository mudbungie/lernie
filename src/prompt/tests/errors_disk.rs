//! Disk and git error paths for [`crate::prompt::run`].
//!
//! Covers the branch-life failures inside `prompt::run`: `git
//! worktree add`, the I/O writes for the dispatch (worktree dir,
//! goal, soul) and for the diagnostic step record (request, response,
//! meta), the dispatch commit's `git add` / `git commit`, the
//! branch-tip capture (`git rev-parse`), and the merge-back-to-main
//! rebase / merge / remove. Compactor-internal failures live in
//! `compactor::tests` — they sit behind the [`crate::prompt::Dispatcher`]
//! boundary, so are not reachable through `prompt::run` with a stub
//! dispatcher. Config and adapter failure paths live in
//! [`super::errors`].

use super::fixtures::*;
use crate::prompt::{Deps, Error, run};

/// Index of `git rev-parse HEAD` on the StubGit's run log: 0 worktree
/// add, 1 dispatch add, 2 dispatch commit, 3 rev-parse. Pinned as a
/// constant so the merge-back op-index labels stay readable.
const REV_PARSE_INDEX: usize = 3;
const REBASE_INDEX: usize = REV_PARSE_INDEX + 1;
const MERGE_OURS_RM_INDEX: usize = REBASE_INDEX + 1;
const MERGE_OURS_LS_TREE_INDEX: usize = MERGE_OURS_RM_INDEX + 1;
const MERGE_OURS_DIFF_INDEX: usize = MERGE_OURS_LS_TREE_INDEX + 1;
const MERGE_INDEX: usize = MERGE_OURS_DIFF_INDEX + 1;
const WORKTREE_REMOVE_INDEX: usize = MERGE_INDEX + 1;

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
    // `write_dispatch_files`'s create_dir_all on the worktree fails
    // on a file-not-dir component.
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
fn run_surfaces_dispatch_add_failure() {
    // git add for the dispatch commit fails (index 1).
    let repo = scaffold_repo(VALID_PER_REPO_PROVIDERS_YAML, Some("body"));
    let adapter = StubAdapter::happy(HAPPY_RESPONSE_JSON.as_bytes());
    let err = run_with_stubs(repo.path(), "hi", &adapter, &StubGit::failing_at(1)).unwrap_err();
    assert!(matches!(err, Error::Git { op: "add", .. }));
}

#[test]
fn run_surfaces_dispatch_commit_failure() {
    let repo = scaffold_repo(VALID_PER_REPO_PROVIDERS_YAML, Some("body"));
    let adapter = StubAdapter::happy(HAPPY_RESPONSE_JSON.as_bytes());
    let err = run_with_stubs(repo.path(), "hi", &adapter, &StubGit::failing_at(2)).unwrap_err();
    assert!(matches!(err, Error::Git { op: "commit", .. }));
}

#[test]
fn run_surfaces_rev_parse_failure() {
    // Branch-tip capture for meta.json's `commit` field (§2.10) is
    // index 3; failing it surfaces as Error::Git { op: "rev-parse" }.
    let repo = scaffold_repo(VALID_PER_REPO_PROVIDERS_YAML, Some("body"));
    let adapter = StubAdapter::happy(HAPPY_RESPONSE_JSON.as_bytes());
    let err = run_with_stubs(
        repo.path(),
        "hi",
        &adapter,
        &StubGit::failing_at(REV_PARSE_INDEX),
    )
    .unwrap_err();
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
fn run_surfaces_step_dir_create_failure() {
    // Step records live at the conv-repo root (§2.2). Pre-create
    // <repo>/steps as a regular file so write_request's
    // create_dir_all on <repo>/steps/<conv-id>/<NNN>/ fails on a
    // file-not-dir component.
    let repo = scaffold_repo(VALID_PER_REPO_PROVIDERS_YAML, Some("body"));
    std::fs::write(repo.path().join("steps"), b"blocker").unwrap();
    let adapter = StubAdapter::happy(HAPPY_RESPONSE_JSON.as_bytes());
    let err = run_with_stubs(repo.path(), "hi", &adapter, &StubGit::ok()).unwrap_err();
    assert!(matches!(err, Error::Io(_)), "got {err:?}");
}

#[test]
fn run_surfaces_request_write_failure() {
    // Pre-create request.json under the *conv-repo's* step dir as a
    // directory so the file write fails (step records relocated out
    // of the worktree per §2.3).
    let repo = scaffold_repo(VALID_PER_REPO_PROVIDERS_YAML, Some("body"));
    let step_dir = repo.path().join("steps/ct-1-deadbeef/001");
    std::fs::create_dir_all(step_dir.join("request.json")).unwrap();
    let adapter = StubAdapter::happy(HAPPY_RESPONSE_JSON.as_bytes());
    let err = run_with_stubs(repo.path(), "hi", &adapter, &StubGit::ok()).unwrap_err();
    assert!(matches!(err, Error::Io(_)), "got {err:?}");
}

#[test]
fn run_surfaces_response_write_failure() {
    let repo = scaffold_repo(VALID_PER_REPO_PROVIDERS_YAML, Some("body"));
    let step_dir = repo.path().join("steps/ct-1-deadbeef/001");
    std::fs::create_dir_all(step_dir.join("response.json")).unwrap();
    let adapter = StubAdapter::happy(HAPPY_RESPONSE_JSON.as_bytes());
    let err = run_with_stubs(repo.path(), "hi", &adapter, &StubGit::ok()).unwrap_err();
    assert!(matches!(err, Error::Io(_)), "got {err:?}");
}

#[test]
fn run_surfaces_meta_write_failure() {
    let repo = scaffold_repo(VALID_PER_REPO_PROVIDERS_YAML, Some("body"));
    let step_dir = repo.path().join("steps/ct-1-deadbeef/001");
    std::fs::create_dir_all(step_dir.join("meta.json")).unwrap();
    let adapter = StubAdapter::happy(HAPPY_RESPONSE_JSON.as_bytes());
    let err = run_with_stubs(repo.path(), "hi", &adapter, &StubGit::ok()).unwrap_err();
    assert!(matches!(err, Error::Io(_)), "got {err:?}");
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
    let tool_executor = StubToolExecutor::ok();
    let deps = Deps {
        adapter: &adapter,
        git: &git,
        clock: &clock,
        id_gen: &id,
        dispatcher: &dispatcher,
        tool_executor: &tool_executor,
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
    // Pre-dispatcher git op count: worktree add, dispatch add,
    // dispatch commit, rev-parse for meta = 4.
    assert_eq!(git.runs.borrow().len(), 4, "merge-to-main never starts");
}

/// Failing the git call at `idx` surfaces as `Error::Git { op: $op,
/// .. }`. Shared helper so each merge-back op-index test stays one
/// line — the macro path tarpaulin trips on otherwise.
fn assert_run_fails_with_git_op(idx: usize, expected_op: &'static str) {
    let repo = scaffold_repo(VALID_PER_REPO_PROVIDERS_YAML, Some("body"));
    let adapter = StubAdapter::happy(HAPPY_RESPONSE_JSON.as_bytes());
    let err = run_with_stubs(repo.path(), "hi", &adapter, &StubGit::failing_at(idx)).unwrap_err();
    match err {
        Error::Git { op, .. } => assert_eq!(op, expected_op),
        other => panic!("expected Error::Git op={expected_op}, got {other:?}"),
    }
}

macro_rules! merge_back_failure_test {
    ($name:ident, $idx:expr, $op:literal) => {
        #[test]
        fn $name() {
            assert_run_fails_with_git_op($idx, $op);
        }
    };
}

merge_back_failure_test!(
    run_surfaces_merge_to_main_rebase_failure,
    REBASE_INDEX,
    "rebase"
);
merge_back_failure_test!(
    run_surfaces_merge_ours_rm_failure,
    MERGE_OURS_RM_INDEX,
    "merge=ours rm"
);
merge_back_failure_test!(
    run_surfaces_merge_ours_ls_tree_failure,
    MERGE_OURS_LS_TREE_INDEX,
    "merge=ours ls-tree"
);
merge_back_failure_test!(
    run_surfaces_merge_ours_diff_failure,
    MERGE_OURS_DIFF_INDEX,
    "merge=ours diff"
);
merge_back_failure_test!(
    run_surfaces_merge_to_main_merge_failure,
    MERGE_INDEX,
    "merge"
);
merge_back_failure_test!(
    run_surfaces_conv_worktree_remove_failure,
    WORKTREE_REMOVE_INDEX,
    "worktree remove"
);
