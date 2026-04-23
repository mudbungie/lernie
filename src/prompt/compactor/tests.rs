//! Unit tests for `compactor::run` and helpers.
//!
//! Lives in a sibling file rather than an inline `mod tests` so the
//! production module stays under the 300-line repo cap. Test coverage
//! for the cmp git sequence sits here because it lives behind the
//! [`crate::prompt::Dispatcher`] boundary in production and is not
//! reachable through `prompt::run` with a stub dispatcher.

use super::*;
use crate::prompt::step::Usage;
use std::cell::RefCell;
use std::path::PathBuf;

fn tmpdir() -> tempfile::TempDir {
    tempfile::TempDir::new().unwrap()
}

/// Compactor-local stubs. Mirror the prompt::tests::fixtures shapes;
/// kept inline because those fixtures are private to the prompt::tests
/// module.
struct FixedClock;
impl Clock for FixedClock {
    fn now_iso8601(&self) -> String {
        unreachable!("compactor::run never reads iso clock; v0.2 stub has no model call")
    }
    fn now_compact(&self) -> String {
        "ct-2".into()
    }
}
struct FixedIdGen;
impl IdGen for FixedIdGen {
    fn short(&self) -> String {
        "deadbeef".into()
    }
}
struct StubGit {
    runs: RefCell<Vec<(PathBuf, Vec<String>)>>,
    fail_at: Option<usize>,
}
impl StubGit {
    fn ok() -> Self {
        Self {
            runs: RefCell::new(Vec::new()),
            fail_at: None,
        }
    }
    fn failing_at(idx: usize) -> Self {
        Self {
            runs: RefCell::new(Vec::new()),
            fail_at: Some(idx),
        }
    }
}
impl GitRunner for StubGit {
    fn run(&self, dest: &Path, args: &[&str]) -> std::io::Result<()> {
        let mut runs = self.runs.borrow_mut();
        let idx = runs.len();
        let entry = (
            dest.to_path_buf(),
            args.iter().map(|s| (*s).to_owned()).collect(),
        );
        runs.push(entry);
        if self.fail_at == Some(idx) {
            Err(std::io::Error::other(format!("stub git fail at {idx}")))
        } else {
            Ok(())
        }
    }
    fn run_capture(&self, _: &Path, _: &[&str]) -> std::io::Result<String> {
        unreachable!("compactor never calls run_capture")
    }
}

/// Lay out a parent worktree with one terminal-step response so
/// `build_summary` succeeds. Returns (TempDir holding repo,
/// parent_worktree_path).
fn parent_with_response(exchange_id: &str, text: &str) -> (tempfile::TempDir, PathBuf) {
    let repo = tmpdir();
    let parent_wt = repo.path().join("parent-wt");
    let step_dir = parent_wt.join(step_dir_rel(exchange_id, TERMINAL_STEP_SEQ));
    std::fs::create_dir_all(&step_dir).unwrap();
    let response = StepResponse {
        assistant_response: text.into(),
        model_id: "m".into(),
        provider: "p".into(),
        usage: Usage {
            input_tokens: 0,
            output_tokens: 0,
        },
        stop_reason: "end_turn".into(),
        started_at: "s".into(),
        ended_at: "e".into(),
    };
    std::fs::write(
        step_dir.join(RESPONSE_FILE),
        serde_json::to_vec(&response).unwrap(),
    )
    .unwrap();
    (repo, parent_wt)
}

fn req<'a>(repo: &'a Path, parent_wt: &'a Path) -> CompactorRequest<'a> {
    CompactorRequest {
        repo,
        parent_branch: "ex/ex1",
        parent_worktree: parent_wt,
        exchange_id: "ex1",
    }
}

#[test]
fn build_summary_happy_path_folds_response_text_with_id() {
    let (_repo, parent_wt) = parent_with_response("ex1", "pong");
    let summary = build_summary(&parent_wt, "ex1").unwrap();
    assert_eq!(summary, "exchange ex1: pong\n");
}

#[test]
fn build_summary_surfaces_missing_response_as_io() {
    let wt = tmpdir();
    let err = build_summary(wt.path(), "ex1").unwrap_err();
    assert!(matches!(err, Error::Io(_)), "got {err:?}");
}

#[test]
fn build_summary_surfaces_malformed_response_as_adapter_json() {
    let wt = tmpdir();
    let step_dir = wt.path().join(step_dir_rel("ex1", TERMINAL_STEP_SEQ));
    std::fs::create_dir_all(&step_dir).unwrap();
    std::fs::write(step_dir.join(RESPONSE_FILE), b"{ not json").unwrap();
    let err = build_summary(wt.path(), "ex1").unwrap_err();
    assert!(matches!(err, Error::AdapterJson(_)), "got {err:?}");
}

#[test]
fn compactor_goal_text_names_parent_branch() {
    let g = compactor_goal("ex/ex1");
    assert!(g.contains("`ex/ex1`"), "{g}");
    assert!(g.contains("write_summary"));
    assert!(g.contains("mark_for_deletion"));
    assert!(g.contains(".agent/goal.md"));
}

#[test]
fn run_happy_path_writes_goal_summary_and_merges() {
    let (repo, parent_wt) = parent_with_response("ex1", "pong");
    let git = StubGit::ok();
    let cmp_branch = "inv/ex1/ct-2-deadbeef";
    let cmp_worktree = repo.path().join(".lernie/worktrees/inv/ex1/ct-2-deadbeef");

    run(
        &req(repo.path(), &parent_wt),
        &git,
        &FixedClock,
        &FixedIdGen,
    )
    .unwrap();

    let runs = git.runs.borrow();
    // 0: worktree add -b cmp_branch cmp_worktree parent_branch (in repo)
    assert_eq!(runs[0].0, repo.path());
    assert_eq!(runs[0].1[..4], ["worktree", "add", "-b", cmp_branch]);
    assert_eq!(runs[0].1[4], cmp_worktree.to_string_lossy().to_string());
    assert_eq!(runs[0].1[5], "ex/ex1");

    // 1: add .agent/goal.md (in cmp wt)
    assert_eq!(runs[1].0, cmp_worktree);
    assert_eq!(runs[1].1, vec!["add", ".agent/goal.md"]);

    // 2: commit dispatch (in cmp wt)
    assert_eq!(runs[2].0, cmp_worktree);
    assert_eq!(runs[2].1[0], "commit");
    assert!(runs[2].1[2].contains("compaction: dispatch"));
    assert!(runs[2].1[2].contains("ex ex1"));

    // 3: add .agent/compactions/001.md (in cmp wt)
    assert_eq!(runs[3].0, cmp_worktree);
    assert_eq!(runs[3].1, vec!["add", ".agent/compactions/001.md"]);

    // 4: commit summary (in cmp wt)
    assert_eq!(runs[4].0, cmp_worktree);
    assert_eq!(runs[4].1[0], "commit");
    assert!(runs[4].1[2].contains("compaction: terminal summary"));

    // 5..8: rebase + merge --no-ff + worktree remove (cmp into ex)
    assert_eq!(runs[5].0, cmp_worktree);
    assert_eq!(runs[5].1, vec!["rebase", "ex/ex1"]);
    assert_eq!(runs[6].0, parent_wt);
    assert_eq!(runs[6].1, vec!["merge", "--no-ff", cmp_branch]);
    assert_eq!(runs[7].0, repo.path());
    assert_eq!(runs[7].1[..2], ["worktree", "remove"]);
    assert_eq!(runs[7].1[2], cmp_worktree.to_string_lossy().to_string());

    // Disk-side: goal.md and summary are present in the cmp wt tree
    // (they were physically written before each git add).
    let goal = std::fs::read_to_string(cmp_worktree.join(".agent/goal.md")).unwrap();
    assert!(goal.contains("`ex/ex1`"));
    let summary = std::fs::read_to_string(cmp_worktree.join(".agent/compactions/001.md")).unwrap();
    assert_eq!(summary, "exchange ex1: pong\n");
}

#[test]
fn run_surfaces_missing_terminal_response_as_io() {
    let repo = tmpdir();
    let parent_wt = repo.path().join("parent-wt");
    std::fs::create_dir_all(&parent_wt).unwrap();
    let git = StubGit::ok();
    let err = run(
        &req(repo.path(), &parent_wt),
        &git,
        &FixedClock,
        &FixedIdGen,
    )
    .unwrap_err();
    assert!(matches!(err, Error::Io(_)), "got {err:?}");
}

/// Asserts that failing the git call at `idx` surfaces as
/// `Error::Git { op: $op, .. }`. Each `run()` call exercises the same
/// pre-failure setup; the only thing that varies is which git call is
/// the one to fail and what op label the harness wraps it in.
macro_rules! run_failing_at_test {
    ($name:ident, $idx:expr, $op:literal) => {
        #[test]
        fn $name() {
            let (repo, parent_wt) = parent_with_response("ex1", "pong");
            let git = StubGit::failing_at($idx);
            let err = run(
                &req(repo.path(), &parent_wt),
                &git,
                &FixedClock,
                &FixedIdGen,
            )
            .unwrap_err();
            assert!(matches!(err, Error::Git { op: $op, .. }), "got {err:?}");
        }
    };
}

run_failing_at_test!(run_surfaces_worktree_add_failure, 0, "worktree add");
run_failing_at_test!(run_surfaces_goal_add_failure, 1, "add");
run_failing_at_test!(run_surfaces_goal_commit_failure, 2, "commit");
run_failing_at_test!(run_surfaces_summary_add_failure, 3, "add");
run_failing_at_test!(run_surfaces_summary_commit_failure, 4, "commit");
run_failing_at_test!(run_surfaces_rebase_failure, 5, "rebase");
run_failing_at_test!(run_surfaces_merge_failure, 6, "merge");
run_failing_at_test!(run_surfaces_worktree_remove_failure, 7, "worktree remove");
