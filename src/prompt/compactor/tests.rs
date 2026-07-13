//! Unit tests for `compactor::run` and helpers.
//!
//! Lives in a sibling file rather than an inline `mod tests` so the
//! production module stays under the 300-line repo cap. Test coverage
//! for the cmp git sequence sits here because it lives behind the
//! [`crate::prompt::Dispatcher`] boundary in production and is not
//! reachable through `prompt::run` with a stub dispatcher.

use super::*;
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
        unreachable!("compactor::run never reads iso clock; v0.3 stub has no model call")
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
    fn run_capture(&self, dest: &Path, args: &[&str]) -> std::io::Result<String> {
        // The compaction merge (ARCH §2.6) issues only plain `run` calls
        // (merge + worktree remove), so this only exists to satisfy the
        // trait; it records like `run` and captures nothing. Exercised by
        // `stub_run_capture_records_and_returns_empty`.
        self.run(dest, args)?;
        Ok(String::new())
    }
}

/// Lay out a parent worktree (no step records — the compactor stub
/// no longer reads them per §2.3 diagnostic-only). Returns (TempDir
/// holding repo, parent_worktree_path).
fn parent_layout(parent_conv_id: &str) -> (tempfile::TempDir, PathBuf) {
    let repo = tmpdir();
    let parent_wt = repo.path().join(parent_conv_id);
    std::fs::create_dir_all(&parent_wt).unwrap();
    (repo, parent_wt)
}

fn req<'a>(repo: &'a Path, parent_wt: &'a Path) -> CompactorRequest<'a> {
    CompactorRequest {
        repo,
        parent_conv_id: "p1",
        parent_worktree: parent_wt,
    }
}

#[test]
fn build_summary_identifies_parent_conversation() {
    // Stub summary: no read of `response.json` (§2.3 diagnostic-only
    // contract bans harness reads). Body just identifies the parent
    // conversation; v0.4+ replaces this with model-driven output
    // delivered through the dispatch contract.
    assert_eq!(
        build_summary("p1"),
        "conversation p1: terminal compaction\n"
    );
    assert_eq!(
        build_summary("conv-id-x"),
        "conversation conv-id-x: terminal compaction\n"
    );
}

#[test]
fn compactor_goal_text_names_parent_branch() {
    let g = compactor_goal("p1");
    assert!(g.contains("`p1`"), "{g}");
    assert!(g.contains("write_summary"));
    assert!(g.contains("mark_for_deletion"));
    assert!(g.contains("goal.md"));
    assert!(g.contains("summary/<NNN>.md"));
}

#[test]
fn run_happy_path_writes_goal_summary_and_merges() {
    let (repo, parent_wt) = parent_layout("p1");
    let git = StubGit::ok();
    // Hyphenated descent (ARCH §2.2): cmp id + worktree directory share
    // the same name `<parent>-<cmp-id>`; the ref is `agents/<id>` (§2.3).
    let cmp_branch = "p1-ct-2-deadbeef";
    let cmp_worktree = crate::workspace::agent_worktree(repo.path(), cmp_branch);

    run(
        &req(repo.path(), &parent_wt),
        &git,
        &FixedClock,
        &FixedIdGen,
    )
    .unwrap();

    let runs = git.runs.borrow();
    // 0: worktree add -b cmp_branch cmp_worktree parent_branch (run
    // inside parent worktree, which shares the .git dir).
    assert_eq!(runs[0].0, parent_wt);
    assert_eq!(
        runs[0].1[..4],
        ["worktree", "add", "-b", "agents/p1-ct-2-deadbeef"]
    );
    assert_eq!(runs[0].1[4], cmp_worktree.to_string_lossy().to_string());
    assert_eq!(runs[0].1[5], "agents/p1");

    // 1: control-file removal (total no-op off a parent tip, §2.3
    // step 2).
    assert_eq!(runs[1].0, cmp_worktree);
    assert_eq!(runs[1].1[..5], ["rm", "-r", "-q", "--ignore-unmatch", "--"]);

    // 2: add goal.md (in cmp wt)
    assert_eq!(runs[2].0, cmp_worktree);
    assert_eq!(runs[2].1, vec!["add", "goal.md"]);

    // 3: commit dispatch (in cmp wt)
    assert_eq!(runs[3].0, cmp_worktree);
    assert_eq!(runs[3].1[0], "commit");
    assert!(runs[3].1[2].contains("compaction: dispatch"));
    assert!(runs[3].1[2].contains("[p1]"));

    // 4: add summary/001.md (in cmp wt)
    assert_eq!(runs[4].0, cmp_worktree);
    assert_eq!(runs[4].1, vec!["add", "summary/001.md"]);

    // 5: commit summary (in cmp wt)
    assert_eq!(runs[5].0, cmp_worktree);
    assert_eq!(runs[5].1[0], "commit");
    assert!(runs[5].1[2].contains("compaction: terminal summary"));

    // 6..7: the compaction merge (ARCH §2.6) — `--no-ff` merge of the
    // compactor branch (its `agents/*` ref) into the dispatching
    // branch, then worktree remove. No rebase and no merge=ours
    // alignment: the merge is conflict-free by construction. Both run
    // inside the parent worktree — any access point onto the one
    // workspace repository (§2.2).
    assert_eq!(runs[6].0, parent_wt);
    assert_eq!(
        runs[6].1,
        vec!["merge", "--no-ff", "agents/p1-ct-2-deadbeef"]
    );
    assert_eq!(runs[7].0, parent_wt);
    assert_eq!(runs[7].1[..2], ["worktree", "remove"]);
    assert_eq!(runs[7].1[2], cmp_worktree.to_string_lossy().to_string());
    assert_eq!(runs.len(), 8, "no rebase/align steps remain");

    // Disk-side: goal.md and summary are present in the cmp wt tree
    // (they were physically written before each git add).
    let goal = std::fs::read_to_string(cmp_worktree.join("goal.md")).unwrap();
    assert!(goal.contains("`p1`"));
    let summary = std::fs::read_to_string(cmp_worktree.join("summary/001.md")).unwrap();
    assert_eq!(summary, "conversation p1: terminal compaction\n");
}

/// Asserts that failing the git call at `idx` surfaces as
/// `Error::Git { op: $op, .. }`. Each `run()` call exercises the same
/// pre-failure setup; the only thing that varies is which git call is
/// the one to fail and what op label the harness wraps it in.
macro_rules! run_failing_at_test {
    ($name:ident, $idx:expr, $op:literal) => {
        #[test]
        fn $name() {
            let (repo, parent_wt) = parent_layout("p1");
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
run_failing_at_test!(run_surfaces_control_rm_failure, 1, "rm control files");
run_failing_at_test!(run_surfaces_goal_add_failure, 2, "add");
run_failing_at_test!(run_surfaces_goal_commit_failure, 3, "commit");
run_failing_at_test!(run_surfaces_summary_add_failure, 4, "add");
run_failing_at_test!(run_surfaces_summary_commit_failure, 5, "commit");
run_failing_at_test!(run_surfaces_merge_failure, 6, "merge");
run_failing_at_test!(run_surfaces_worktree_remove_failure, 7, "worktree remove");

#[test]
fn stub_run_capture_records_and_returns_empty() {
    // The compaction merge issues no capture (§2.6), so the stub's
    // `run_capture` is trait-only; assert its recording behavior directly.
    let git = StubGit::ok();
    let out = git
        .run_capture(std::path::Path::new("/x"), &["rev-parse"])
        .unwrap();
    assert!(out.is_empty());
    assert_eq!(git.runs.borrow().len(), 1);
}
