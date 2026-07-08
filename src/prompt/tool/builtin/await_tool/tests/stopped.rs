//! Stopped-path + fd-close-gate tests (ARCH §2.9, §3.5, §4.4).
//!
//! A terminal on-disk signature surfaces as `{"status":"stopped"}`
//! ONLY once the writer has closed the `response.json` fd. Two
//! signatures both surface through it (kill / crash / explicit stop are
//! indistinguishable on disk, §2.9):
//!
//! 1. The latest step's `response.json` last segment carries a brazen
//!    `error` (Failed) AND no process holds its fd open.
//! 2. No trailing `end` line (NoTerminal) AND no writer holds the fd.
//!
//! The fd-close gate is the load-bearing §3.5/§4.4 rule: the harness
//! holds ONE fd across every retry attempt and the backoff sleeps
//! between them, so a mid-retry `error` segment with a writer still
//! present reads `in_flight`, never `stopped`.

use super::super::*;
use super::fixtures::{
    ConflictOnFirstSleep, LiveRepo, NoopSleeper, StubPgidFinder, env, input_for,
};
use std::io::Cursor;

// brazen v=1: terminal is `end`, with `finish`/`error` inside the
// segment (§4.4).
const BRAZEN_ERROR: &str = r#"{"type":"message_start","v":1,"role":"assistant"}
{"type":"error","kind":"transport","message":"reset"}
{"type":"end"}
"#;

const BRAZEN_FINISH: &str = r#"{"type":"message_start","v":1,"role":"assistant"}
{"type":"finish","reason":"stop"}
{"type":"end"}
"#;

// Two segments: a failed attempt (error+end) then a clean retry
// (finish+end). The last segment is authoritative → Complete.
const BRAZEN_RETRY_THEN_CLEAN: &str = r#"{"type":"message_start","v":1,"role":"assistant"}
{"type":"error","kind":"provider","message":"429"}
{"type":"end"}
{"type":"message_start","v":1,"role":"assistant"}
{"type":"finish","reason":"stop"}
{"type":"end"}
"#;

fn fixture_with_unmerged_sub() -> LiveRepo {
    let live = LiveRepo::new();
    live.run_git(&["checkout", "-b", "p1"]);
    live.run_git(&["commit", "--allow-empty", "-m", "p1 base"]);
    live.branch_and_commit("p1", "p1-sub", "marker.txt");
    live.run_git(&["checkout", "p1"]);
    live
}

fn run_stopped(
    live: &LiveRepo,
    finder: &dyn crate::prompt::stop::PgidFinder,
    sleeper: &dyn Sleeper,
) -> serde_json::Value {
    let mut stdin = Cursor::new(input_for("p1-sub"));
    let mut stdout = Vec::new();
    let env_stub = env(live.repo(), "p1");
    run(
        &mut stdin,
        &mut stdout,
        &env_stub,
        &live.git,
        finder,
        sleeper,
    )
    .unwrap();
    serde_json::from_slice(&stdout).unwrap()
}

#[test]
fn stopped_when_error_segment_and_fd_closed() {
    // Failed segment + no writer → the retry loop settled failed (§2.10).
    let live = fixture_with_unmerged_sub();
    live.write_response("p1-sub", 1, BRAZEN_ERROR);
    let payload = run_stopped(&live, &StubPgidFinder::no_writer(), &NoopSleeper::new());
    assert_eq!(payload["status"], "stopped");
    assert!(payload.get("summary").is_none());
}

#[test]
fn mid_retry_error_segment_with_writer_open_is_in_flight() {
    // THE fd-close gate (§3.5/§4.4): a trailing `error` segment while a
    // writer still holds the fd open is mid-retry, not failed. Must
    // read in_flight — the loop keeps polling (ConflictOnFirstSleep
    // resolves it to `conflicted`).
    let live = fixture_with_unmerged_sub();
    live.write_response("p1-sub", 1, BRAZEN_ERROR);
    let sleeper = ConflictOnFirstSleep::new(&live, "p1-sub");
    assert_eq!(
        run_stopped(&live, &StubPgidFinder::writer_present(), &sleeper)["status"],
        "conflicted"
    );
}

#[test]
fn two_segment_failed_then_clean_with_fd_open_is_in_flight() {
    // A failed-then-clean two-segment file: the last segment is Complete
    // (§4.4 last-segment-authoritative), so even with the fd open it is
    // in_flight, never stopped.
    let live = fixture_with_unmerged_sub();
    live.write_response("p1-sub", 1, BRAZEN_RETRY_THEN_CLEAN);
    let sleeper = ConflictOnFirstSleep::new(&live, "p1-sub");
    let finder = StubPgidFinder::writer_present();
    assert_eq!(
        run_stopped(&live, &finder, &sleeper)["status"],
        "conflicted"
    );
    assert!(
        finder.calls.borrow().is_empty(),
        "a complete segment must not consult /proc"
    );
}

#[test]
fn brazen_finish_alone_keeps_loop_in_flight() {
    // A `finish`+`end` step (Complete) is not terminal for the
    // subagent — the harness may still advance; /proc is NOT consulted.
    let live = fixture_with_unmerged_sub();
    live.write_response("p1-sub", 1, BRAZEN_FINISH);
    let sleeper = ConflictOnFirstSleep::new(&live, "p1-sub");
    let finder = StubPgidFinder::writer_present();
    assert_eq!(
        run_stopped(&live, &finder, &sleeper)["status"],
        "conflicted"
    );
    assert!(
        finder.calls.borrow().is_empty(),
        "a complete segment must not consult /proc"
    );
}

#[test]
fn stopped_uses_latest_step_when_earlier_step_was_clean() {
    // Earlier step finished cleanly; the latest step carries the error.
    let live = fixture_with_unmerged_sub();
    live.write_response("p1-sub", 1, BRAZEN_FINISH);
    live.write_response("p1-sub", 2, BRAZEN_ERROR);
    assert_eq!(
        run_stopped(&live, &StubPgidFinder::no_writer(), &NoopSleeper::new())["status"],
        "stopped"
    );
}

#[test]
fn no_steps_dir_is_in_flight_until_terminal_state_appears() {
    let live = fixture_with_unmerged_sub();
    let sleeper = ConflictOnFirstSleep::new(&live, "p1-sub");
    assert_eq!(
        run_stopped(&live, &StubPgidFinder::writer_present(), &sleeper)["status"],
        "conflicted"
    );
}

#[test]
fn empty_steps_dir_is_in_flight() {
    let live = fixture_with_unmerged_sub();
    std::fs::create_dir_all(live.repo().join("steps").join("p1-sub")).unwrap();
    std::fs::write(
        live.repo().join("steps").join("p1-sub").join("README"),
        b"not a step\n",
    )
    .unwrap();
    let sleeper = ConflictOnFirstSleep::new(&live, "p1-sub");
    assert_eq!(
        run_stopped(&live, &StubPgidFinder::writer_present(), &sleeper)["status"],
        "conflicted"
    );
}

#[test]
fn latest_step_without_response_json_is_in_flight() {
    let live = fixture_with_unmerged_sub();
    std::fs::create_dir_all(live.repo().join("steps").join("p1-sub").join("001")).unwrap();
    let sleeper = ConflictOnFirstSleep::new(&live, "p1-sub");
    let finder = StubPgidFinder::writer_present();
    assert_eq!(
        run_stopped(&live, &finder, &sleeper)["status"],
        "conflicted"
    );
    assert!(finder.calls.borrow().is_empty());
}

#[test]
fn response_json_read_failure_surfaces_as_typed_error() {
    // `response.json` is a directory — `fs::read` fails with a non-
    // NotFound io error → Error::Git { op: "read response.json" }.
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
        &StubPgidFinder::writer_present(),
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
fn malformed_jsonl_with_writer_present_keeps_loop_in_flight() {
    // Last completed line is non-JSON (NoTerminal). With a writer, the
    // fd-close gate keeps it in_flight.
    let live = fixture_with_unmerged_sub();
    live.write_response("p1-sub", 1, "garbage line\n");
    let sleeper = ConflictOnFirstSleep::new(&live, "p1-sub");
    assert_eq!(
        run_stopped(&live, &StubPgidFinder::writer_present(), &sleeper)["status"],
        "conflicted"
    );
}

#[test]
fn response_json_with_no_newlines_keeps_loop_in_flight_when_writer_present() {
    let live = fixture_with_unmerged_sub();
    live.write_response("p1-sub", 1, "{\"type\":\"message_start\"}");
    let sleeper = ConflictOnFirstSleep::new(&live, "p1-sub");
    assert_eq!(
        run_stopped(&live, &StubPgidFinder::writer_present(), &sleeper)["status"],
        "conflicted"
    );
}

#[test]
fn response_json_only_blank_lines_keeps_loop_in_flight_when_writer_present() {
    let live = fixture_with_unmerged_sub();
    live.write_response("p1-sub", 1, "\n\n\n");
    let sleeper = ConflictOnFirstSleep::new(&live, "p1-sub");
    assert_eq!(
        run_stopped(&live, &StubPgidFinder::writer_present(), &sleeper)["status"],
        "conflicted"
    );
}
