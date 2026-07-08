//! Stopped-path tests. Two on-disk signatures both surface as
//! `{"status":"stopped"}` (ARCH §2.9 — kill / crash / explicit stop
//! are indistinguishable on disk):
//!
//! 1. The latest step's `response.json` ended in a §4.4 `error`
//!    event (clean failure).
//! 2. The latest step's `response.json` has no terminal event line
//!    AND no process holds its fd open — the kill-mid-stream
//!    signature, detected via the [`super::fixtures::StubPgidFinder`]
//!    in tests and via `/proc/<pid>/fd/*` (`ProcFsFinder`) in
//!    production.
//!
//! Tests covering "in-flight states that must keep polling" use
//! [`super::fixtures::ConflictOnFirstSleep`]: the sleeper writes a
//! conflicted ref on its first call, so the next poll resolves to
//! `conflicted` and the loop terminates deterministically.

use super::super::*;
use super::fixtures::{
    ConflictOnFirstSleep, LiveRepo, NoopSleeper, StubPgidFinder, env, input_for,
};
use std::io::Cursor;

const ERROR_EVENT: &str = r#"{"type":"message_start"}
{"type":"error","kind":"fatal","message":"boom"}
"#;

const MESSAGE_STOP: &str = r#"{"type":"message_start"}
{"type":"message_stop","api_calls":1}
"#;

// brazen v=1 fixtures (bl-507a dual vocabulary): terminal is `end`,
// with `finish`/`error` carried inside the segment.
const BRAZEN_ERROR: &str = r#"{"type":"message_start","v":1,"role":"assistant"}
{"type":"error","kind":"transport","message":"reset"}
{"type":"end"}
"#;

const BRAZEN_FINISH: &str = r#"{"type":"message_start","v":1,"role":"assistant"}
{"type":"finish","reason":"stop"}
{"type":"end"}
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
fn stopped_when_latest_response_ends_in_error_event() {
    let live = fixture_with_unmerged_sub();
    live.write_response("p1-sub", 1, ERROR_EVENT);
    let payload = run_stopped(
        &live,
        &StubPgidFinder::writer_present(),
        &NoopSleeper::new(),
    );
    assert_eq!(payload["status"], "stopped");
    assert!(payload.get("summary").is_none());
}

#[test]
fn stopped_when_latest_response_ends_in_brazen_error_segment() {
    // brazen v=1 failed attempt: `end` terminal, `error` in the last
    // segment → Failed → stopped (bl-507a).
    let live = fixture_with_unmerged_sub();
    live.write_response("p1-sub", 1, BRAZEN_ERROR);
    let payload = run_stopped(
        &live,
        &StubPgidFinder::writer_present(),
        &NoopSleeper::new(),
    );
    assert_eq!(payload["status"], "stopped");
}

#[test]
fn brazen_finish_alone_keeps_loop_in_flight() {
    // A brazen `finish`+`end` step (Complete) is not terminal for the
    // subagent — the harness may still advance. Mirrors the legacy
    // `message_stop` case; /proc is NOT consulted (bl-507a).
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
    // Earlier step ended cleanly; the latest step is the one with
    // the `error` event. The classifier reads the latest, not all
    // history.
    let live = fixture_with_unmerged_sub();
    live.write_response("p1-sub", 1, MESSAGE_STOP);
    live.write_response("p1-sub", 2, ERROR_EVENT);
    assert_eq!(
        run_stopped(
            &live,
            &StubPgidFinder::writer_present(),
            &NoopSleeper::new()
        )["status"],
        "stopped"
    );
}

#[test]
fn no_steps_dir_is_in_flight_until_terminal_state_appears() {
    // Subagent has zero step records yet — latest-step lookup
    // returns None and the loop polls again.
    let live = fixture_with_unmerged_sub();
    let sleeper = ConflictOnFirstSleep::new(&live, "p1-sub");
    assert_eq!(
        run_stopped(&live, &StubPgidFinder::writer_present(), &sleeper)["status"],
        "conflicted"
    );
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
    assert_eq!(
        run_stopped(&live, &StubPgidFinder::writer_present(), &sleeper)["status"],
        "conflicted"
    );
}

#[test]
fn latest_step_without_response_json_is_in_flight() {
    // Step dir exists but no response.json yet (file write not
    // landed). `fs::read` returns NotFound; the classifier resolves
    // to `Absent` and skips the writer probe.
    let live = fixture_with_unmerged_sub();
    std::fs::create_dir_all(live.repo().join("steps").join("p1-sub").join("001")).unwrap();
    let sleeper = ConflictOnFirstSleep::new(&live, "p1-sub");
    let finder = StubPgidFinder::writer_present();
    assert_eq!(
        run_stopped(&live, &finder, &sleeper)["status"],
        "conflicted"
    );
    // No response.json on disk — the loop must NOT have probed
    // /proc, since there's no path to scan.
    assert!(finder.calls.borrow().is_empty());
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
fn malformed_jsonl_lines_with_writer_present_keep_loop_in_flight() {
    // Last completed line is non-JSON. With a writer holding the fd
    // open, this is mid-stream garbage and the loop keeps polling —
    // ConflictOnFirstSleep then resolves it.
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
    // Mid-write very early — writer has not flushed any complete
    // line yet. No `\n` in the buffer means no completed line, but
    // /proc shows the writer holds the fd, so we keep polling.
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
    // Only `\n\n\n` — every split line is empty; classifier reports
    // `NonTerminal` and the writer-present /proc probe keeps the
    // loop spinning.
    let live = fixture_with_unmerged_sub();
    live.write_response("p1-sub", 1, "\n\n\n");
    let sleeper = ConflictOnFirstSleep::new(&live, "p1-sub");
    assert_eq!(
        run_stopped(&live, &StubPgidFinder::writer_present(), &sleeper)["status"],
        "conflicted"
    );
}

#[test]
fn message_stop_alone_keeps_loop_in_flight_until_terminal_state_appears() {
    // A subagent step ending in `message_stop` is NOT terminal for
    // the subagent (the harness still has terminal compaction +
    // merge-back to do). The poll loop must spin until something
    // terminal lands. /proc is NOT consulted — the message_stop arm
    // short-circuits ahead of the writer probe.
    let live = fixture_with_unmerged_sub();
    live.write_response("p1-sub", 1, MESSAGE_STOP);
    let sleeper = ConflictOnFirstSleep::new(&live, "p1-sub");
    let finder = StubPgidFinder::writer_present();
    assert_eq!(
        run_stopped(&live, &finder, &sleeper)["status"],
        "conflicted"
    );
    assert_eq!(*sleeper.count.borrow(), 1);
    assert!(
        finder.calls.borrow().is_empty(),
        "message_stop must not consult /proc"
    );
}
