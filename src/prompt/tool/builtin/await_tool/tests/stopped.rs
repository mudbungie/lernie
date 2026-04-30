//! Stopped-path tests: subagent's latest step's `response.json`
//! ended in a §4.4 `error` event. v0.4 deliberately scopes stopped
//! detection to the `error` signature; killed-mid-stream is filed
//! as a follow-on (see SKILL.md).
//!
//! Tests covering "in-flight states that must keep polling" use
//! [`super::fixtures::ConflictOnFirstSleep`]: the sleeper writes a
//! conflicted ref on its first call, so the next poll resolves to
//! `conflicted` and the loop terminates deterministically.

use super::super::*;
use super::fixtures::{ConflictOnFirstSleep, LiveRepo, NoopSleeper, env, input_for};
use std::io::Cursor;

const ERROR_EVENT: &str = r#"{"type":"message_start"}
{"type":"error","kind":"fatal","message":"boom"}
"#;

const MESSAGE_STOP: &str = r#"{"type":"message_start"}
{"type":"message_stop","api_calls":1}
"#;

fn fixture_with_unmerged_sub() -> LiveRepo {
    let live = LiveRepo::new();
    live.run_git(&["checkout", "-b", "p1"]);
    live.run_git(&["commit", "--allow-empty", "-m", "p1 base"]);
    live.branch_and_commit("p1", "p1-sub", "marker.txt");
    live.run_git(&["checkout", "p1"]);
    // Sub stays unmerged.
    live
}

fn run_stopped(live: &LiveRepo, sleeper: &dyn Sleeper) -> serde_json::Value {
    let mut stdin = Cursor::new(input_for("p1-sub"));
    let mut stdout = Vec::new();
    let env_stub = env(live.repo(), "p1");
    run(&mut stdin, &mut stdout, &env_stub, &live.git, sleeper).unwrap();
    serde_json::from_slice(&stdout).unwrap()
}

#[test]
fn stopped_when_latest_response_ends_in_error_event() {
    let live = fixture_with_unmerged_sub();
    live.write_response("p1-sub", 1, ERROR_EVENT);
    let payload = run_stopped(&live, &NoopSleeper::new());
    assert_eq!(payload["status"], "stopped");
    assert!(payload.get("summary").is_none());
}

#[test]
fn stopped_uses_latest_step_when_earlier_step_was_clean() {
    // Earlier step ended cleanly; the latest step is the one with
    // the `error` event. The classifier reads the latest, not all
    // history.
    let live = fixture_with_unmerged_sub();
    live.write_response("p1-sub", 1, MESSAGE_STOP);
    live.write_response("p1-sub", 2, ERROR_EVENT);
    assert_eq!(run_stopped(&live, &NoopSleeper::new())["status"], "stopped");
}

#[test]
fn no_steps_dir_is_in_flight_until_terminal_state_appears() {
    // Subagent has zero step records yet — latest-step lookup
    // returns None and the loop polls again.
    let live = fixture_with_unmerged_sub();
    let sleeper = ConflictOnFirstSleep::new(&live, "p1-sub");
    assert_eq!(run_stopped(&live, &sleeper)["status"], "conflicted");
}

#[test]
fn empty_steps_dir_is_in_flight() {
    // steps/<handle>/ exists but has no numeric subdirs (e.g. a
    // foreign file) — `latest_step_dir` returns None.
    let live = fixture_with_unmerged_sub();
    std::fs::create_dir_all(live.repo().join("steps").join("p1-sub")).unwrap();
    std::fs::write(
        live.repo().join("steps").join("p1-sub").join("README"),
        b"not a step\n",
    )
    .unwrap();
    let sleeper = ConflictOnFirstSleep::new(&live, "p1-sub");
    assert_eq!(run_stopped(&live, &sleeper)["status"], "conflicted");
}

#[test]
fn latest_step_without_response_json_is_in_flight() {
    // Step dir exists but no response.json yet (file write not
    // landed). `fs::read` returns NotFound, classifier returns
    // false.
    let live = fixture_with_unmerged_sub();
    std::fs::create_dir_all(live.repo().join("steps").join("p1-sub").join("001")).unwrap();
    let sleeper = ConflictOnFirstSleep::new(&live, "p1-sub");
    assert_eq!(run_stopped(&live, &sleeper)["status"], "conflicted");
}

#[test]
fn response_json_read_failure_surfaces_as_typed_error() {
    // `response.json` is a directory, not a file — `fs::read` fails
    // with an io error that is *not* NotFound, surfacing as
    // [`Error::Git { op: "read response.json", .. }`].
    let live = fixture_with_unmerged_sub();
    let bogus = live
        .repo()
        .join("steps")
        .join("p1-sub")
        .join("001")
        .join("response.json");
    std::fs::create_dir_all(&bogus).unwrap();
    let mut stdin = Cursor::new(input_for("p1-sub"));
    let mut stdout = Vec::new();
    let env_stub = env(live.repo(), "p1");
    let err = run(
        &mut stdin,
        &mut stdout,
        &env_stub,
        &live.git,
        &NoopSleeper::new(),
    )
    .unwrap_err();
    assert!(
        matches!(
            err,
            Error::Git {
                op: "read response.json",
                ..
            }
        ),
        "{err}"
    );
}

#[test]
fn malformed_jsonl_lines_keep_loop_in_flight() {
    // Last completed line is non-JSON; the classifier returns false
    // and the loop polls again.
    let live = fixture_with_unmerged_sub();
    live.write_response("p1-sub", 1, "garbage line\n");
    let sleeper = ConflictOnFirstSleep::new(&live, "p1-sub");
    assert_eq!(run_stopped(&live, &sleeper)["status"], "conflicted");
}

#[test]
fn response_json_with_no_newlines_keeps_loop_in_flight() {
    // Mid-write very early — writer has not flushed any complete
    // line yet. No `\n` in the buffer means no completed line, and
    // the classifier returns false.
    let live = fixture_with_unmerged_sub();
    live.write_response("p1-sub", 1, "{\"type\":\"message_start\"}");
    let sleeper = ConflictOnFirstSleep::new(&live, "p1-sub");
    assert_eq!(run_stopped(&live, &sleeper)["status"], "conflicted");
}

#[test]
fn response_json_only_blank_lines_keeps_loop_in_flight() {
    // Only `\n\n\n` — every split line is empty; the rfind-non-empty
    // check returns None and the classifier returns false.
    let live = fixture_with_unmerged_sub();
    live.write_response("p1-sub", 1, "\n\n\n");
    let sleeper = ConflictOnFirstSleep::new(&live, "p1-sub");
    assert_eq!(run_stopped(&live, &sleeper)["status"], "conflicted");
}

#[test]
fn message_stop_alone_keeps_loop_in_flight_until_terminal_state_appears() {
    // A subagent step ending in `message_stop` is NOT terminal for
    // the subagent (the harness still has terminal compaction +
    // merge-back to do). The poll loop must spin until something
    // terminal lands.
    let live = fixture_with_unmerged_sub();
    live.write_response("p1-sub", 1, MESSAGE_STOP);
    let sleeper = ConflictOnFirstSleep::new(&live, "p1-sub");
    assert_eq!(run_stopped(&live, &sleeper)["status"], "conflicted");
    assert_eq!(*sleeper.count.borrow(), 1);
}
