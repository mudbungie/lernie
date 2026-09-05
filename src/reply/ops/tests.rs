//! The trail's reading: every field, the strictness that names one, and the
//! standings that ride verbatim.

use serde_json::json;

use super::{CLEAN, row};

/// A whole row, read field for field — the shape REMOTE §9.17 spells.
#[test]
fn a_row_carries_what_ran_how_it_ended_and_where_its_alarm_stands() {
    let read = row(&json!({
        "argv": "bl close x",
        "cwd": "/ws/home",
        "exit": 1,
        "exit_label": "exit 1",
        "failed": true,
        "origin": "balls",
        "standing": "live",
        "stderr": "the gate said no",
        "stdout": "",
        "ts": "1700"
    }))
    .expect("a whole row reads");
    assert_eq!(read.argv, "bl close x");
    assert_eq!(read.cwd, "/ws/home");
    assert_eq!(read.exit, 1);
    assert_eq!(read.exit_label, "exit 1");
    assert!(read.failed);
    assert_eq!(read.origin, "balls");
    assert_eq!(read.standing, "live");
    assert_eq!(read.stderr, "the gate said no");
    assert_eq!(read.stdout, "");
    assert_eq!(read.ts, "1700");
}

/// **The handoff is not a failure**, and the sentinel exit that says so is
/// carried rather than classified: the label is the engine's and the integer
/// is beside it (REMOTE §9.17's *"total, never absent"*).
#[test]
fn a_detached_row_is_not_failed_and_keeps_its_sentinel() {
    let read = row(&json!({
        "argv": "litany prompt c-1",
        "cwd": "/ws/home",
        "exit": -2,
        "exit_label": "detached — handed off, no exit to observe",
        "failed": false,
        "origin": "conversation",
        "standing": "detached",
        "stderr": "",
        "stdout": "",
        "ts": "1705"
    }))
    .expect("a detached row reads");
    assert!(!read.failed);
    assert_eq!(read.exit, -2);
    assert_ne!(read.standing, CLEAN);
}

/// **A standing this build has never seen paints as itself** — rung 3, in the
/// one place this row spends it. A vocabulary that grew upstream costs a badge
/// and not a decode.
#[test]
fn a_standing_this_build_does_not_know_rides_verbatim() {
    let read = row(&json!({
        "argv": "bl close x", "cwd": "/ws/home", "exit": 0,
        "exit_label": "exit 0", "failed": false, "origin": "balls",
        "standing": "quarantined", "stderr": "", "stdout": "", "ts": "1"
    }))
    .expect("an unknown standing is not a refusal");
    assert_eq!(read.standing, "quarantined");
}

/// Rung 1: a missing or mistyped field refuses, and the refusal names it.
#[test]
fn a_row_missing_a_field_refuses_and_names_it() {
    let why = row(&json!({"argv": "bl close x"})).expect_err("an incomplete row refuses");
    assert!(why.contains("ts"), "{why}");
    let why = row(&json!("a string")).expect_err("a row that is not an object refuses");
    assert!(why.contains("not an object"), "{why}");
    let why = row(&json!({
        "argv": "x", "cwd": "/ws/home", "exit": "one",
        "exit_label": "exit 1", "failed": true, "origin": "balls",
        "standing": "live", "stderr": "", "stdout": "", "ts": "1"
    }))
    .expect_err("a non-integer exit refuses");
    assert!(why.contains("exit"), "{why}");
}

/// The exit is narrowed, exactly as a captured run's is: a status no `i32`
/// holds is an engine saying something this seat has no way to paint.
#[test]
fn an_exit_no_i32_holds_refuses() {
    let why = row(&json!({
        "argv": "x", "cwd": "/ws/home", "exit": 5_000_000_000_i64,
        "exit_label": "exit", "failed": true, "origin": "balls",
        "standing": "live", "stderr": "", "stdout": "", "ts": "1"
    }))
    .expect_err("an out-of-range exit refuses");
    assert!(why.contains("out of range"), "{why}");
}
