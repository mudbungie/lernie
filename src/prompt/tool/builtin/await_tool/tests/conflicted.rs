//! Conflicted-path tests: the merge protocol wrote
//! `refs/lernie/conflicted/<handle>` on rebase failure (ARCH §2.6
//! step 6). await reads that ref directly via `git for-each-ref`.

use super::super::*;
use super::fixtures::{LiveRepo, NoopSleeper, StubPgidFinder, env, input_for};
use std::io::Cursor;

#[test]
fn conflicted_when_marker_ref_present_at_sub_branch_tip() {
    let live = LiveRepo::new();
    live.run_git(&["checkout", "-b", "p1"]);
    live.run_git(&["commit", "--allow-empty", "-m", "p1 base"]);
    live.branch_and_commit("p1", "p1-sub", "marker.txt");
    live.run_git(&["checkout", "p1"]);
    // The merge protocol writes this ref on rebase failure
    // (prompt::merge::tests asserts the write); here we install it
    // directly to keep the test scoped to await.
    live.run_git(&["update-ref", "refs/lernie/conflicted/p1-sub", "p1-sub"]);

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
    assert!(payload.get("summary").is_none());
}

#[test]
fn conflicted_takes_precedence_over_merged() {
    // Edge case: the conflicted marker exists AND the branch is
    // (somehow) merged. The marker means an operator hasn't yet
    // cleaned up after the merge protocol's failure surface, so the
    // status is `conflicted` — the safer signal.
    let live = LiveRepo::new();
    live.run_git(&["checkout", "-b", "p1"]);
    live.run_git(&["commit", "--allow-empty", "-m", "p1 base"]);
    live.branch_and_commit("p1", "p1-sub", "marker.txt");
    live.run_git(&["checkout", "p1"]);
    live.run_git(&["update-ref", "refs/lernie/conflicted/p1-sub", "p1-sub"]);
    live.run_git(&["merge", "--no-ff", "-m", "merge sub", "p1-sub"]);

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

#[test]
fn unrelated_conflicted_ref_does_not_match_handle() {
    // Marker ref is for a different sub-branch; await must not pick
    // it up. Validates the ref-name match is exact.
    let live = LiveRepo::new();
    live.run_git(&["checkout", "-b", "p1"]);
    live.run_git(&["commit", "--allow-empty", "-m", "p1 base"]);
    live.branch_and_commit("p1", "p1-sub", "m1.txt");
    live.branch_and_commit("p1", "p1-other", "m2.txt");
    live.run_git(&["checkout", "p1"]);
    live.run_git(&["update-ref", "refs/lernie/conflicted/p1-other", "p1-other"]);
    live.write_response(
        "p1-sub",
        1,
        r#"{"type":"error","kind":"fatal","message":"x"}
"#,
    );

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
    // p1-sub is not the conflicted one; its own latest response.json
    // ended in error → stopped.
    let payload: serde_json::Value = serde_json::from_slice(&stdout).unwrap();
    assert_eq!(payload["status"], "stopped");
}
