//! Real-git tests for the launched driver's own-branch entry (§2.11):
//! the pin-1 silent no-op, acquire-or-exit, delivery of a late deposit
//! (the exit-race payoff: what an exit-launched driver finds and does),
//! and worktree rematerialization off the persistent ref (§2.3 step 6).

use super::{DriveOutcome, drive};
use crate::prompt::SystemClock;
use crate::prompt::inbox::{deposit, inbox_dir, try_acquire};
use crate::template::{GitRunner, ROOT_WORKTREE, RealGit};
use std::path::Path;
use tempfile::TempDir;

/// A root-shaped agent id (two hyphen-free tokens, §2.3).
const AGENT: &str = "20260101-a1";

/// A workspace with a `root` primary worktree on `main` and one agent
/// branch + worktree — the §2.2 shipped layout, real git throughout.
fn workspace() -> TempDir {
    let ws = TempDir::new().unwrap();
    let root = ws.path().join(ROOT_WORKTREE);
    std::fs::create_dir_all(&root).unwrap();
    let g = RealGit::new();
    g.run(&root, &["init", "-b", "main"]).unwrap();
    g.run(&root, &["config", "user.email", "t@test.invalid"])
        .unwrap();
    g.run(&root, &["config", "user.name", "t"]).unwrap();
    g.run(&root, &["commit", "--allow-empty", "-m", "init"])
        .unwrap();
    let wt = ws.path().join(AGENT);
    let wt_str = wt.to_string_lossy().to_string();
    g.run(
        &root,
        &["worktree", "add", "-b", AGENT, wt_str.as_str(), "main"],
    )
    .unwrap();
    ws
}

fn tip(ws: &Path) -> String {
    RealGit::new()
        .run_capture(&ws.join(ROOT_WORKTREE), &["rev-parse", AGENT])
        .unwrap()
}

#[test]
fn a_held_lock_means_already_driven() {
    let ws = workspace();
    let _held = try_acquire(&inbox_dir(ws.path(), AGENT))
        .unwrap()
        .expect("free");
    let outcome = drive(ws.path(), AGENT, &RealGit::new()).unwrap();
    assert_eq!(outcome, DriveOutcome::AlreadyDriven);
}

#[test]
fn empty_inbox_exits_silently_without_stepping_or_relaunching() {
    // §2.11 pin 1: acquire, find nothing, exit — no step (the branch tip
    // does not move), no epitaph (no deposit appears anywhere), no
    // further launch (structural: `drive` takes no launcher at all).
    let ws = workspace();
    let before = tip(ws.path());
    let outcome = drive(ws.path(), AGENT, &RealGit::new()).unwrap();
    assert_eq!(outcome, DriveOutcome::NothingToDeliver);
    assert_eq!(tip(ws.path()), before, "no step: the tip must not move");
    // No deposit anywhere: the workspace inbox tree holds only the
    // agent's own (empty) inbox dir the probe created.
    let entries: Vec<_> = std::fs::read_dir(ws.path().join("inbox"))
        .unwrap()
        .flatten()
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .collect();
    assert_eq!(entries, vec![AGENT.to_string()]);
    assert_eq!(
        std::fs::read_dir(inbox_dir(ws.path(), AGENT))
            .unwrap()
            .count(),
        0
    );
}

#[test]
fn a_late_deposit_is_delivered_by_the_launched_driver() {
    // The exit-race payoff: a deposit that landed after an executor's
    // final drain sits pending; the exit-launched driver acquires the
    // released lock and delivers it as a delivery commit.
    let ws = workspace();
    deposit(ws.path(), AGENT, "user", "late mail", &SystemClock).unwrap();
    let outcome = drive(ws.path(), AGENT, &RealGit::new()).unwrap();
    assert_eq!(outcome, DriveOutcome::Delivered(1));
    // The file left the inbox (rename semantics, §2.11)…
    assert_eq!(
        std::fs::read_dir(inbox_dir(ws.path(), AGENT))
            .unwrap()
            .count(),
        0
    );
    // …and landed committed in the transcript.
    let shown = RealGit::new()
        .run_capture(
            &ws.path().join(ROOT_WORKTREE),
            &["show", &format!("{AGENT}:messages/001-user.md")],
        )
        .unwrap();
    assert!(shown.contains("late mail"), "got {shown:?}");
}

#[test]
fn a_torn_down_worktree_is_rematerialized_before_delivery() {
    // §2.3 step 6: quiescence may tear the worktree down; the branch ref
    // persists and the next driver rematerializes off it.
    let ws = workspace();
    let root = ws.path().join(ROOT_WORKTREE);
    let wt = ws.path().join(AGENT);
    let wt_str = wt.to_string_lossy().to_string();
    RealGit::new()
        .run(&root, &["worktree", "remove", wt_str.as_str()])
        .unwrap();
    assert!(!wt.exists());
    deposit(ws.path(), AGENT, "user", "wake up", &SystemClock).unwrap();
    let outcome = drive(ws.path(), AGENT, &RealGit::new()).unwrap();
    assert_eq!(outcome, DriveOutcome::Delivered(1));
    assert!(wt.join("messages").join("001-user.md").exists());
}

#[test]
fn rematerialize_failure_is_surfaced_as_a_git_error() {
    let ws = workspace();
    let root = ws.path().join(ROOT_WORKTREE);
    let wt = ws.path().join(AGENT);
    let wt_str = wt.to_string_lossy().to_string();
    let g = RealGit::new();
    g.run(&root, &["worktree", "remove", wt_str.as_str()])
        .unwrap();
    g.run(&root, &["branch", "-D", AGENT]).unwrap();
    deposit(ws.path(), AGENT, "user", "orphaned", &SystemClock).unwrap();
    let err = drive(ws.path(), AGENT, &RealGit::new()).unwrap_err();
    assert!(
        matches!(
            err,
            crate::prompt::Error::Git {
                op: "worktree add (rematerialize)",
                ..
            }
        ),
        "{err}"
    );
}

#[test]
fn a_broken_inbox_surfaces_as_an_executor_lock_error() {
    // A file where the agent's inbox dir should be makes the acquire
    // fail with an I/O error rather than a clean no-op.
    let ws = workspace();
    std::fs::create_dir_all(ws.path().join("inbox")).unwrap();
    std::fs::write(inbox_dir(ws.path(), AGENT), b"not a dir").unwrap();
    let err = drive(ws.path(), AGENT, &RealGit::new()).unwrap_err();
    assert!(
        matches!(err, crate::prompt::Error::ExecutorLock { .. }),
        "{err}"
    );
}
