//! The tool window: the two transitions, the merge that pairs them, and the
//! readings that must not collapse into each other.

use serde_json::json;

use super::{Window, fold, windows};

/// One frame's window, read.
fn read(value: &serde_json::Value) -> Result<Vec<Window>, String> {
    windows(value.as_object().expect("an object"))
}

/// **The pair REMOTE §5.5 spells**, both transitions of one call, read as they
/// cross: the opening entry carries the name and the input, the closing one
/// carries the id and the status and restates nothing.
#[test]
fn the_two_transitions_are_read_as_upstream_spells_them() {
    let entries = read(&json!({"tools": [
        {"tool_use": "toolu_01", "tool": "box2_Bash",
         "input": "{\"command\":\"hostname && uptime\"}"},
        {"tool_use": "toolu_01", "exit_code": 0},
    ]}))
    .expect("a window");
    assert_eq!(
        entries,
        vec![
            Window {
                tool_use: "toolu_01".to_owned(),
                tool: Some("box2_Bash".to_owned()),
                input: Some("{\"command\":\"hostname && uptime\"}".to_owned()),
                exit_code: None,
                held: None,
            },
            Window {
                tool_use: "toolu_01".to_owned(),
                tool: None,
                input: None,
                exit_code: Some(0),
                held: None,
            },
        ]
    );
}

/// **Presence is the status.** A call in flight and a call that came back 0
/// are two readings and nothing else says them apart — an absent `exit_code`
/// read as zero would paint every running command as one that succeeded.
#[test]
fn an_absent_exit_code_is_in_flight_and_never_a_zero() {
    let entries = read(&json!({"tools": [
        {"tool_use": "a", "tool": "Bash", "input": "{}"},
        {"tool_use": "b", "tool": "Bash", "input": "{}", "exit_code": 0},
    ]}))
    .expect("a window");
    assert_eq!(entries[0].exit_code, None);
    assert_eq!(entries[1].exit_code, Some(0));
}

/// **The merge is what lets the closing entry restate nothing**: two
/// transitions in, one call out, keyed by the id both carry.
#[test]
fn the_two_halves_fold_onto_one_call() {
    let mut held = Vec::new();
    for entry in read(&json!({"tools": [
        {"tool_use": "toolu_01", "tool": "box2_Bash", "input": "{}"},
        {"tool_use": "toolu_02", "tool": "Read", "input": "{}"},
        {"tool_use": "toolu_01", "exit_code": 2},
    ]}))
    .expect("a window")
    {
        fold(&mut held, entry);
    }
    assert_eq!(held.len(), 2, "{held:?}");
    assert_eq!(held[0].tool.as_deref(), Some("box2_Bash"));
    assert_eq!(held[0].exit_code, Some(2));
    assert_eq!(held[1].exit_code, None, "the other call is still in flight");
}

/// **A transition is additive whichever half it is.** A closing entry that
/// carried a name would not blank the input the opening one rode on, and a
/// second opening for one id cannot un-close a call that has already landed.
#[test]
fn a_later_transition_fills_and_never_blanks() {
    let mut held = Vec::new();
    fold(
        &mut held,
        Window {
            tool_use: "x".to_owned(),
            tool: Some("Bash".to_owned()),
            input: Some("{}".to_owned()),
            exit_code: Some(1),
            held: None,
        },
    );
    fold(
        &mut held,
        Window {
            tool_use: "x".to_owned(),
            tool: None,
            input: None,
            exit_code: None,
            held: None,
        },
    );
    assert_eq!(
        held,
        vec![Window {
            tool_use: "x".to_owned(),
            tool: Some("Bash".to_owned()),
            input: Some("{}".to_owned()),
            exit_code: Some(1),
            held: None,
        }]
    );
}

/// The empty window is a reading and not an absence: nothing ran since the
/// last frame, said out loud.
#[test]
fn an_empty_window_reads_as_a_window_with_nothing_in_it() {
    assert_eq!(read(&json!({"tools": []})).expect("a window"), Vec::new());
}
