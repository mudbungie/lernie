//! Unit tests for `subagent::spawn_subagent_branch`.
//!
//! Lives in a sibling file rather than an inline `mod tests` so the
//! production module stays under the 300-line repo cap.

use super::*;
use std::cell::RefCell;
use std::io;
use std::path::PathBuf;

#[derive(Default)]
struct StubGit {
    runs: RefCell<Vec<(PathBuf, Vec<String>)>>,
    fail_at: Option<usize>,
}
impl StubGit {
    fn ok() -> Self {
        Self::default()
    }
    fn failing_at(idx: usize) -> Self {
        Self {
            fail_at: Some(idx),
            ..Self::default()
        }
    }
}
impl GitRunner for StubGit {
    fn run(&self, dest: &Path, args: &[&str]) -> io::Result<()> {
        let mut runs = self.runs.borrow_mut();
        let idx = runs.len();
        runs.push((
            dest.to_path_buf(),
            args.iter().map(|s| (*s).to_owned()).collect(),
        ));
        if self.fail_at == Some(idx) {
            Err(io::Error::other(format!("stub git fail at {idx}")))
        } else {
            Ok(())
        }
    }
    fn run_capture(&self, _dest: &Path, _args: &[&str]) -> io::Result<String> {
        // The subagent spawn helper never calls run_capture — every git
        // op it issues is fire-and-forget. The trait still requires an
        // impl; panicking documents the assumption and tarpaulin's
        // `ignore-panics` excludes the branch from the coverage floor.
        unreachable!("spawn_subagent_branch never issues capturing git ops")
    }
}

/// A grant of nothing against the stub's fixed config commit — the
/// compactor's shape (§2.7), and all these stub trees can honour.
const EMPTY_GRANT: crate::prompt::dispatch::Grant<'static> = crate::prompt::dispatch::Grant {
    role: "worker",
    tools: &[],
    config_commit: "c0ffee",
};

fn tmpdir() -> tempfile::TempDir {
    tempfile::TempDir::new().unwrap()
}

fn req<'a>(parent_wt: &'a Path, sub_wt: &'a Path, soul: Option<&'a str>) -> SpawnRequest<'a> {
    SpawnRequest {
        parent_worktree: parent_wt,
        parent_branch: "p1",
        sub_branch: "p1-ct-2-deadbeef",
        sub_worktree: sub_wt,
        fork_point: None,
        goal_text: "do the thing\n",
        soul_text: soul,
        name: None,
        // The stub worktrees carry no `descriptions/**` and the grant is
        // empty, so the descriptor half of the trim is a no-op here
        // (§3.3) — exercised on its own in
        // `dispatch::step_commit::descriptors::tests`.
        grant: &EMPTY_GRANT,
        commit_subject: "dispatch: worker [p1-ct-2-deadbeef]",
    }
}

#[test]
fn writes_goal_and_soul_when_soul_present() {
    let parent_dir = tmpdir();
    let sub_dir = tmpdir();
    let parent_wt = parent_dir.path();
    let sub_wt = sub_dir.path();
    let git = StubGit::ok();

    spawn_subagent_branch(&req(parent_wt, sub_wt, Some("you are worker\n")), &git).unwrap();

    let runs = git.runs.borrow();
    // 0: worktree add (in parent worktree) — ids map to agents/* refs
    // at the git boundary (§2.3).
    assert_eq!(runs[0].0, parent_wt);
    assert_eq!(
        runs[0].1[..4],
        ["worktree", "add", "-b", "agents/p1-ct-2-deadbeef"]
    );
    assert_eq!(runs[0].1[4], sub_wt.to_string_lossy().to_string());
    assert_eq!(runs[0].1[5], "agents/p1");
    // 1: control-file removal (total, --ignore-unmatch; §2.3 step 2)
    assert_eq!(runs[1].0, sub_wt);
    assert_eq!(runs[1].1[..5], ["rm", "-r", "-q", "--ignore-unmatch", "--"]);
    // 2: stage the settled name — the trim's fourth part (§2.3)
    assert_eq!(runs[2].0, sub_wt);
    assert_eq!(runs[2].1, vec!["add", "name"]);
    // 3: add goal.md soul.md (in sub worktree)
    assert_eq!(runs[3].0, sub_wt);
    assert_eq!(runs[3].1, vec!["add", "goal.md", "soul.md"]);
    // 4: commit (in sub worktree)
    assert_eq!(runs[4].0, sub_wt);
    assert_eq!(runs[4].1[0], "commit");
    assert_eq!(runs[4].1[2], "dispatch: worker [p1-ct-2-deadbeef]");

    assert_eq!(
        std::fs::read_to_string(sub_wt.join("goal.md")).unwrap(),
        "do the thing\n"
    );
    assert_eq!(
        std::fs::read_to_string(sub_wt.join("soul.md")).unwrap(),
        "you are worker\n"
    );
    // An unnamed child still carries the fact's file, empty (§2.3 —
    // one shape, so a fork never inherits its parent's name).
    assert_eq!(std::fs::read_to_string(sub_wt.join("name")).unwrap(), "");
}

#[test]
fn writes_only_goal_when_soul_is_none() {
    let parent_dir = tmpdir();
    let sub_dir = tmpdir();
    let git = StubGit::ok();

    spawn_subagent_branch(&req(parent_dir.path(), sub_dir.path(), None), &git).unwrap();

    let runs = git.runs.borrow();
    // The stage step adds only goal.md.
    assert_eq!(runs[3].1, vec!["add", "goal.md"]);
    assert!(
        !sub_dir.path().join("soul.md").exists(),
        "soul.md should not be written"
    );
}

#[test]
fn surfaces_worktree_add_failure() {
    let parent_dir = tmpdir();
    let sub_dir = tmpdir();
    let git = StubGit::failing_at(0);
    let err =
        spawn_subagent_branch(&req(parent_dir.path(), sub_dir.path(), None), &git).unwrap_err();
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
fn surfaces_control_rm_failure() {
    let parent_dir = tmpdir();
    let sub_dir = tmpdir();
    let git = StubGit::failing_at(1);
    let err = spawn_subagent_branch(
        &req(parent_dir.path(), sub_dir.path(), Some("soul\n")),
        &git,
    )
    .unwrap_err();
    assert!(
        matches!(
            err,
            Error::Git {
                op: "rm control files",
                ..
            }
        ),
        "got {err:?}"
    );
}

#[test]
fn surfaces_add_failure() {
    let parent_dir = tmpdir();
    let sub_dir = tmpdir();
    let git = StubGit::failing_at(3);
    let err = spawn_subagent_branch(
        &req(parent_dir.path(), sub_dir.path(), Some("soul\n")),
        &git,
    )
    .unwrap_err();
    assert!(matches!(err, Error::Git { op: "add", .. }), "got {err:?}");
}

#[test]
fn surfaces_commit_failure() {
    let parent_dir = tmpdir();
    let sub_dir = tmpdir();
    let git = StubGit::failing_at(4);
    let err =
        spawn_subagent_branch(&req(parent_dir.path(), sub_dir.path(), None), &git).unwrap_err();
    assert!(
        matches!(err, Error::Git { op: "commit", .. }),
        "got {err:?}"
    );
}

#[test]
fn surfaces_io_failure_when_sub_worktree_is_a_file() {
    // Production `git worktree add` creates the directory; in the
    // stub-git test path we lean on `create_dir_all` for the same.
    // If the target path already exists as a regular file (e.g.
    // because of a stale remnant), that fails — the helper surfaces
    // the io::Error unchanged via the Error::Io conversion.
    let parent_dir = tmpdir();
    let sub_wt = parent_dir.path().join("collision");
    std::fs::write(&sub_wt, b"existing file").unwrap();
    let git = StubGit::ok();
    let r = SpawnRequest {
        parent_worktree: parent_dir.path(),
        parent_branch: "p1",
        sub_branch: "p1-x",
        sub_worktree: &sub_wt,
        fork_point: None,
        goal_text: "g",
        soul_text: None,
        name: None,
        grant: &EMPTY_GRANT,
        commit_subject: "x",
    };
    let err = spawn_subagent_branch(&r, &git).unwrap_err();
    assert!(matches!(err, Error::Io(_)), "got {err:?}");
}

#[test]
fn surfaces_name_settle_failure() {
    // The trim's fourth part (§2.3): staging the settled `name` fails,
    // and the dispatch commit reports it in its own voice rather than
    // as an anonymous git error.
    let parent_dir = tmpdir();
    let sub_dir = tmpdir();
    let git = StubGit::failing_at(2);
    let err =
        spawn_subagent_branch(&req(parent_dir.path(), sub_dir.path(), None), &git).unwrap_err();
    assert!(
        matches!(
            err,
            Error::Git {
                op: "settle the agent name",
                ..
            }
        ),
        "got {err:?}"
    );
}
