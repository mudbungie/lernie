//! Budget-exhausted path: budget enforcement wrote
//! `refs/lernie/budget-exhausted/<handle>` when the conversation crossed
//! a `workflow.yaml` limit (ARCH §6). await reads that ref directly via
//! `git for-each-ref`, the same git-native pattern as the conflicted ref.

use super::super::*;
use super::fixtures::{LiveRepo, NoopSleeper, StubPgidFinder, env, input_for};
use std::io::Cursor;

#[test]
fn budget_exhausted_when_marker_ref_present() {
    let live = LiveRepo::new();
    live.run_git(&["checkout", "-b", "p1"]);
    live.run_git(&["commit", "--allow-empty", "-m", "p1 base"]);
    live.branch_and_commit("p1", "p1-sub", "marker.txt");
    live.run_git(&["checkout", "p1"]);
    // Budget enforcement writes this ref on exhaustion (§6, via
    // `budget::mark_exhausted`); install it directly to keep the test
    // scoped to await.
    live.run_git(&[
        "update-ref",
        "refs/lernie/budget-exhausted/p1-sub",
        "p1-sub",
    ]);

    let mut stdin = Cursor::new(input_for("p1-sub"));
    let mut stdout = Vec::new();
    let env_stub = env(live.repo(), "p1");
    run(
        &mut stdin,
        &mut stdout,
        &env_stub,
        &live.git,
        &StubPgidFinder::writer_present(),
        &NoopSleeper::new(),
    )
    .unwrap();

    let payload: serde_json::Value = serde_json::from_slice(&stdout).unwrap();
    assert_eq!(payload["status"], "budget_exhausted");
    assert!(payload.get("summary").is_none());
}

#[test]
fn conflicted_takes_precedence_over_budget_exhausted() {
    // Both markers present: conflicted (a harness defect, §2.6 step 6) is
    // the higher-priority signal, so it wins the precedence order.
    let live = LiveRepo::new();
    live.run_git(&["checkout", "-b", "p1"]);
    live.run_git(&["commit", "--allow-empty", "-m", "p1 base"]);
    live.branch_and_commit("p1", "p1-sub", "marker.txt");
    live.run_git(&["checkout", "p1"]);
    live.run_git(&["update-ref", "refs/lernie/conflicted/p1-sub", "p1-sub"]);
    live.run_git(&[
        "update-ref",
        "refs/lernie/budget-exhausted/p1-sub",
        "p1-sub",
    ]);

    let mut stdin = Cursor::new(input_for("p1-sub"));
    let mut stdout = Vec::new();
    let env_stub = env(live.repo(), "p1");
    run(
        &mut stdin,
        &mut stdout,
        &env_stub,
        &live.git,
        &StubPgidFinder::writer_present(),
        &NoopSleeper::new(),
    )
    .unwrap();
    let payload: serde_json::Value = serde_json::from_slice(&stdout).unwrap();
    assert_eq!(payload["status"], "conflicted");
}
