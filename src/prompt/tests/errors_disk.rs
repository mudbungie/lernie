//! Disk and git error paths for [`crate::prompt::run`].
//!
//! Covers the branch-life failures inside `prompt::run`: `git
//! worktree add`, the I/O writes for the dispatch (worktree dir,
//! goal, soul) and for the diagnostic step record (request, response,
//! meta), the dispatch commit's `git add` / `git commit`, the
//! branch-tip capture (`git rev-parse`), the model-output transcript
//! entry commit, and the terminal result-deposit's branch-tip read
//! (§2.6). Merge-back is gone (§2.6), so its rebase / merge / remove
//! arms are gone with it, and terminal compaction is deleted (§2.7), so
//! no compactor dispatch follows a final response. Config and adapter
//! failure paths live in [`super::errors`].

use super::fixtures::*;
use crate::prompt::Error;

/// Indexes on the StubGit's run log. Control resolution runs first
/// (§2.2): 0 config-head rev-parse, 1-3 the three `show` control reads.
/// Branch work follows: 4 worktree add, 5 the dispatch commit's
/// control-file removal (§2.3 step 2), 6 dispatch add, 7 dispatch
/// commit, 8 the step-1 drain stray-probe (`git status`, §2.11), 9
/// user-message delivery add, 10 user-message delivery commit (§2.11 —
/// the initial message is delivered through the front door before step
/// 1's read state is captured), 11 rev-parse. Pinned as constants so
/// the transcript/terminal op-index labels stay readable.
pub(super) const WORKTREE_ADD_INDEX: usize = 4;
const REV_PARSE_INDEX: usize = 11;
/// After the model call settles, the transcript writer (§2.3) commits
/// the model-output entry — `git add` then `commit` — before the loop
/// terminates (no tool_use on the happy stream).
const TRANSCRIPT_ADD_INDEX: usize = REV_PARSE_INDEX + 1;
const TRANSCRIPT_COMMIT_INDEX: usize = TRANSCRIPT_ADD_INDEX + 1;
/// The terminal event deposits a result message (§2.6, §2.3 step 5),
/// which reads the branch tip (`git rev-parse HEAD`) as its terminal
/// ref. It is the last git op before the loop breaks; the deposit itself
/// is a no-op for a root (no parent inbox, §2.4).
const TERMINAL_REV_PARSE_INDEX: usize = TRANSCRIPT_COMMIT_INDEX + 1;

#[test]
fn run_surfaces_worktree_create_failure() {
    // Pre-create the worktree path as a regular file so
    // `write_dispatch_files`'s create_dir_all on the worktree fails
    // on a file-not-dir component.
    let repo = scaffold_repo(VALID_PER_REPO_PROVIDERS_YAML, Some("body"));
    let wt = worktree_path(repo.path());
    std::fs::create_dir_all(wt.parent().unwrap()).unwrap();
    std::fs::write(&wt, b"blocker").unwrap();
    let adapter = StubAdapter::happy(&happy_response_bytes());
    let err = run_with_stubs(repo.path(), "hi", &adapter, &StubGit::ok()).unwrap_err();
    assert!(matches!(err, Error::Io(_)), "got {err:?}");
}

#[test]
fn run_surfaces_goal_write_failure() {
    // Worktree dir exists but goal.md is already a directory.
    let repo = scaffold_repo(VALID_PER_REPO_PROVIDERS_YAML, Some("body"));
    let wt = worktree_path(repo.path());
    std::fs::create_dir_all(wt.join("goal.md")).unwrap();
    let adapter = StubAdapter::happy(&happy_response_bytes());
    let err = run_with_stubs(repo.path(), "hi", &adapter, &StubGit::ok()).unwrap_err();
    assert!(matches!(err, Error::Io(_)), "got {err:?}");
}

#[test]
fn run_surfaces_soul_write_failure() {
    // Worktree dir + goal.md writeable, but soul.md is a directory.
    let repo = scaffold_repo(VALID_PER_REPO_PROVIDERS_YAML, Some("body"));
    let wt = worktree_path(repo.path());
    std::fs::create_dir_all(wt.join("soul.md")).unwrap();
    let adapter = StubAdapter::happy(&happy_response_bytes());
    let err = run_with_stubs(repo.path(), "hi", &adapter, &StubGit::ok()).unwrap_err();
    assert!(matches!(err, Error::Io(_)), "got {err:?}");
}

#[test]
fn run_surfaces_rev_parse_failure() {
    // Branch-tip capture for meta.json's `commit` field (§2.10) is
    // [`REV_PARSE_INDEX`]; failing it surfaces as Error::Git { op:
    // "rev-parse" }.
    let repo = scaffold_repo(VALID_PER_REPO_PROVIDERS_YAML, Some("body"));
    let adapter = StubAdapter::happy(&happy_response_bytes());
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
    let adapter = StubAdapter::happy(&happy_response_bytes());
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
    let adapter = StubAdapter::happy(&happy_response_bytes());
    let err = run_with_stubs(repo.path(), "hi", &adapter, &StubGit::ok()).unwrap_err();
    assert!(matches!(err, Error::Io(_)), "got {err:?}");
}

#[test]
fn run_surfaces_response_write_failure() {
    let repo = scaffold_repo(VALID_PER_REPO_PROVIDERS_YAML, Some("body"));
    let step_dir = repo.path().join("steps/ct-1-deadbeef/001");
    std::fs::create_dir_all(step_dir.join("response.json")).unwrap();
    let adapter = StubAdapter::happy(&happy_response_bytes());
    let err = run_with_stubs(repo.path(), "hi", &adapter, &StubGit::ok()).unwrap_err();
    assert!(matches!(err, Error::Io(_)), "got {err:?}");
}

#[test]
fn run_surfaces_meta_write_failure() {
    let repo = scaffold_repo(VALID_PER_REPO_PROVIDERS_YAML, Some("body"));
    let step_dir = repo.path().join("steps/ct-1-deadbeef/001");
    std::fs::create_dir_all(step_dir.join("meta.json")).unwrap();
    let adapter = StubAdapter::happy(&happy_response_bytes());
    let err = run_with_stubs(repo.path(), "hi", &adapter, &StubGit::ok()).unwrap_err();
    assert!(matches!(err, Error::Io(_)), "got {err:?}");
}

/// Failing the git call at `idx` surfaces as `Error::Git { op: $op,
/// .. }`. Shared helper so each op-index test stays one line — the macro
/// path tarpaulin trips on otherwise.
fn assert_run_fails_with_git_op(idx: usize, expected_op: &'static str) {
    let repo = scaffold_repo(VALID_PER_REPO_PROVIDERS_YAML, Some("body"));
    let adapter = StubAdapter::happy(&happy_response_bytes());
    let err = run_with_stubs(repo.path(), "hi", &adapter, &StubGit::failing_at(idx)).unwrap_err();
    match err {
        Error::Git { op, .. } => assert_eq!(op, expected_op),
        other => panic!("expected Error::Git op={expected_op}, got {other:?}"),
    }
}

macro_rules! git_op_failure_test {
    ($name:ident, $idx:expr, $op:literal) => {
        #[test]
        fn $name() {
            assert_run_fails_with_git_op($idx, $op);
        }
    };
}

git_op_failure_test!(
    run_surfaces_transcript_add_failure,
    TRANSCRIPT_ADD_INDEX,
    "transcript add"
);
git_op_failure_test!(
    run_surfaces_transcript_commit_failure,
    TRANSCRIPT_COMMIT_INDEX,
    "transcript commit"
);
// The terminal result-deposit's branch-tip read (§2.6, §2.3 step 5).
git_op_failure_test!(
    run_surfaces_terminal_deposit_rev_parse_failure,
    TERMINAL_REV_PARSE_INDEX,
    "rev-parse"
);
