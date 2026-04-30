//! Happy-path tests for the merged terminal state — sub-branch
//! reachable from parent's tip, summary readable from sub's tree.

use super::super::*;
use super::fixtures::{LiveRepo, NoopSleeper, StubPgidFinder, env, input_for};
use std::io::Cursor;

#[test]
fn merged_path_returns_status_and_summary_from_sub_branch_tip() {
    let live = LiveRepo::new();
    // Root conversation lives on `p1`; subagent on `p1-sub`. The
    // subagent commits its summary, then the parent merges --no-ff.
    live.run_git(&["checkout", "-b", "p1"]);
    live.run_git(&["commit", "--allow-empty", "-m", "p1 base"]);
    live.branch_and_commit("p1", "p1-sub", "marker.txt");
    live.write_summary_on("p1-sub", 1, "terminal summary text\n");
    live.run_git(&["checkout", "p1"]);
    live.run_git(&["merge", "--no-ff", "-m", "merge sub", "p1-sub"]);

    let mut stdin = Cursor::new(input_for("p1-sub"));
    let mut stdout = Vec::new();
    let env_stub = env(live.repo(), "p1");
    let finder = StubPgidFinder::writer_present();
    let sleeper = NoopSleeper::new();

    run(
        &mut stdin,
        &mut stdout,
        &env_stub,
        &live.git,
        &finder,
        &sleeper,
    )
    .unwrap();

    let payload: serde_json::Value = serde_json::from_slice(&stdout).unwrap();
    assert_eq!(payload["status"], "merged");
    assert_eq!(payload["summary"], "terminal summary text");
    // Loop terminated on the first poll — no sleep.
    assert_eq!(sleeper.count.get(), 0);
    // Merged path resolves on git refs alone — no /proc probe.
    assert!(finder.calls.borrow().is_empty());
}

#[test]
fn merged_path_picks_highest_seq_summary() {
    // Multiple summaries on the sub-branch (intermediate +
    // terminal); await reads the highest seq.
    let live = LiveRepo::new();
    live.run_git(&["checkout", "-b", "p1"]);
    live.run_git(&["commit", "--allow-empty", "-m", "p1 base"]);
    live.branch_and_commit("p1", "p1-sub", "marker.txt");
    live.write_summary_on("p1-sub", 1, "first\n");
    live.write_summary_on("p1-sub", 2, "second\n");
    live.write_summary_on("p1-sub", 3, "terminal\n");
    live.run_git(&["checkout", "p1"]);
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
    assert_eq!(payload["status"], "merged");
    assert_eq!(payload["summary"], "terminal");
}

#[test]
fn merged_without_summary_surfaces_typed_error() {
    // Sub-branch merged but never wrote a `summary/<NNN>.md`. Per
    // ARCH §2.7 the terminal compactor must produce one, so this is
    // a regression surface — typed error, not a panic.
    let live = LiveRepo::new();
    live.run_git(&["checkout", "-b", "p1"]);
    live.run_git(&["commit", "--allow-empty", "-m", "p1 base"]);
    live.branch_and_commit("p1", "p1-sub", "marker.txt");
    live.run_git(&["checkout", "p1"]);
    live.run_git(&["merge", "--no-ff", "-m", "merge sub", "p1-sub"]);

    let mut stdin = Cursor::new(input_for("p1-sub"));
    let mut stdout = Vec::new();
    let env_stub = env(live.repo(), "p1");
    let err = run(
        &mut stdin,
        &mut stdout,
        &env_stub,
        &live.git,
        &StubPgidFinder::writer_present(),
        &NoopSleeper::new(),
    )
    .unwrap_err();
    assert!(matches!(err, Error::MergedWithoutSummary), "{err}");
}
