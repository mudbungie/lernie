//! The git-op error arms of the compaction merge, routed through a stub
//! whose `run_capture` reports `MERGE_HEAD` present so control reaches
//! everything after merge setup — the `add`, the filter, and the commit.
//! The behavioral arms live in [`super`], against real git.

use super::*;

/// Stub git reporting a merge in progress (non-empty `MERGE_HEAD`) so the
/// arms after merge setup are reachable; `run` fails at a chosen call
/// index, and the filter's `diff --cached` capture answers `diff` —
/// `Some(paths)` for the classes it reports, `None` to fail the capture.
struct StubGit {
    invocations: RefCell<Vec<Vec<String>>>,
    fail_at: usize,
    diff: Option<String>,
}
impl StubGit {
    /// Nothing to filter: every diff class is empty, so the call
    /// sequence is merge, add, commit.
    fn failing_at(idx: usize) -> Self {
        Self {
            invocations: RefCell::new(Vec::new()),
            fail_at: idx,
            diff: Some(String::new()),
        }
    }
    /// Every diff class reports `paths`, so the filter runs its `rm` and
    /// `checkout`: merge, add, rm, checkout, commit.
    fn with_filtered(idx: usize, paths: &str) -> Self {
        Self {
            diff: Some(paths.into()),
            ..Self::failing_at(idx)
        }
    }
    fn diff_capture_failing() -> Self {
        Self {
            diff: None,
            ..Self::failing_at(usize::MAX)
        }
    }
}
impl GitRunner for StubGit {
    fn run(&self, _dest: &Path, args: &[&str]) -> std::io::Result<()> {
        let idx = self.invocations.borrow().len();
        self.invocations
            .borrow_mut()
            .push(args.iter().map(|s| (*s).to_owned()).collect());
        if idx == self.fail_at {
            Err(std::io::Error::other("stub fail"))
        } else {
            Ok(())
        }
    }
    fn run_capture(&self, _dest: &Path, args: &[&str]) -> std::io::Result<String> {
        if args.first() == Some(&"diff") {
            return self
                .diff
                .clone()
                .ok_or_else(|| std::io::Error::other("stub fail"));
        }
        // MERGE_HEAD present → merge_in_progress is true, so control
        // reaches the add/filter/commit arms.
        Ok("deadbeefsha".into())
    }
}

#[test]
fn add_failure_surfaces_as_git_error() {
    // invocations: 0=merge, 1=add(fail).
    let err = merge(&PathBuf::from("/x"), "p1-cmp", &StubGit::failing_at(1)).unwrap_err();
    assert_git_op(err, "compaction merge add");
}

#[test]
fn commit_failure_surfaces_as_git_error() {
    // invocations: 0=merge, 1=add, 2=commit(fail) — nothing to filter.
    let err = merge(&PathBuf::from("/x"), "p1-cmp", &StubGit::failing_at(2)).unwrap_err();
    assert_git_op(err, "compaction merge commit");
}

#[test]
fn filter_diff_failure_surfaces_as_git_error() {
    let err = merge(
        &PathBuf::from("/x"),
        "p1-cmp",
        &StubGit::diff_capture_failing(),
    )
    .unwrap_err();
    assert_git_op(err, "compaction merge filter");
}

#[test]
fn filter_restore_failure_surfaces_as_git_error() {
    // invocations: 0=merge, 1=add, 2=rm(fail) — the filter's restore of an
    // addition the compactor's dialog contributed.
    let stub = StubGit::with_filtered(2, "messages/003-goal.md\n");
    let err = merge(&PathBuf::from("/x"), "p1-cmp", &stub).unwrap_err();
    assert_git_op(err, "compaction merge filter");
}

/// Stub git for the **decline** path: `MERGE_HEAD` present and
/// `ls-files -u` reporting one path at stages 1/2/3 — a content conflict.
/// `run` fails at a chosen index (0 = merge, 1 = merge --abort,
/// 2 = update-ref); `unmerged: None` fails the `ls-files` capture itself.
struct ConflictGit {
    invocations: RefCell<usize>,
    fail_at: usize,
    unmerged: Option<&'static str>,
}
impl ConflictGit {
    fn failing_at(idx: usize) -> Self {
        Self {
            invocations: RefCell::new(0),
            fail_at: idx,
            unmerged: Some(
                "100644 aaa 1\tsummary/001.md\n\
                 100644 bbb 2\tsummary/001.md\n\
                 100644 ccc 3\tsummary/001.md\n",
            ),
        }
    }
    fn unmerged_capture_failing() -> Self {
        Self {
            unmerged: None,
            ..Self::failing_at(usize::MAX)
        }
    }
}
impl GitRunner for ConflictGit {
    fn run(&self, _dest: &Path, _args: &[&str]) -> std::io::Result<()> {
        let idx = *self.invocations.borrow();
        *self.invocations.borrow_mut() = idx + 1;
        if idx == self.fail_at {
            Err(std::io::Error::other("stub fail"))
        } else {
            Ok(())
        }
    }
    fn run_capture(&self, _dest: &Path, args: &[&str]) -> std::io::Result<String> {
        if args.first() == Some(&"ls-files") {
            return self
                .unmerged
                .map(str::to_owned)
                .ok_or_else(|| std::io::Error::other("stub fail"));
        }
        Ok("deadbeefsha".into())
    }
}

#[test]
fn an_unmerged_listing_failure_surfaces_as_git_error() {
    let err = merge(
        &PathBuf::from("/x"),
        "p1-cmp",
        &ConflictGit::unmerged_capture_failing(),
    )
    .unwrap_err();
    assert_git_op(err, "compaction merge unmerged");
}

#[test]
fn an_abort_failure_surfaces_as_git_error() {
    // invocations: 0=merge, 1=merge --abort(fail).
    let err = merge(&PathBuf::from("/x"), "p1-cmp", &ConflictGit::failing_at(1)).unwrap_err();
    assert_git_op(err, "compaction merge abort");
}

#[test]
fn a_decline_mark_failure_surfaces_as_git_error() {
    // invocations: 0=merge, 1=merge --abort, 2=update-ref(fail).
    let err = merge(&PathBuf::from("/x"), "p1-cmp", &ConflictGit::failing_at(2)).unwrap_err();
    assert_git_op(err, "compaction merge decline update-ref");
}

#[test]
fn a_declined_merge_reports_the_conflicted_paths() {
    let outcome = merge(
        &PathBuf::from("/x"),
        "p1-cmp",
        &ConflictGit::failing_at(usize::MAX),
    )
    .unwrap();
    assert_eq!(
        outcome,
        MergeOutcome::Conflicted(vec!["summary/001.md".to_string()])
    );
}
