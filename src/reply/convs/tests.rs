//! The conversation list: the two names, the signed age, and rung 3 on both
//! token fields.

use super::{AgentState, ConvRow, Tone, row};
use serde_json::json;

/// A whole row, with every field the list paints and several the engine sends
/// that this build does not read — rung 4, which is what makes a field the
/// engine adds a non-event here.
#[test]
fn a_row_carries_what_the_list_paints_and_ignores_what_it_does_not() {
    let read = row(&json!({
        "root_id": "20260830T051200Z-a1b2",
        "display": "port the paint probe",
        "name": "port the paint probe",
        "display_only": false,
        "state": "live",
        "uncertain": false,
        "preview": "the galley reports the string that went in",
        "age_secs": 42,
        "flight": "inference",
        "attention": 1,
        "members": 3,
        "direct": 2,
        "stoppable": true,
        "depth": 0,
        "tone": "live",
        "ball": {"id": "bl-428f"},
    }))
    .expect("a row");
    assert_eq!(
        read,
        ConvRow {
            root_id: "20260830T051200Z-a1b2".to_owned(),
            display: "port the paint probe".to_owned(),
            name: Some("port the paint probe".to_owned()),
            state: AgentState::Live,
            preview: "the galley reports the string that went in".to_owned(),
            age_secs: 42,
            attention: 1,
            members: 3,
            depth: 0,
            tone: Tone::Live,
        }
    );
}

/// **A name the engine will not answer to is withheld, and the id still
/// works.** That is the whole reason the two are separate fields: a seat that
/// posted the display name would be posting a target the engine refuses.
#[test]
fn a_display_only_name_is_absent_and_the_id_remains() {
    let read = row(&json!({
        "root_id": "20260830T051200Z-c3d4", "display": "a goal-stamp title",
        "display_only": true, "state": "quiescent", "preview": "",
        "age_secs": 0, "attention": 0, "members": 1, "depth": 1, "tone": "weak",
    }))
    .expect("a row");
    assert_eq!(read.name, None);
    assert_eq!(read.display, "a goal-stamp title");
    assert_eq!(read.root_id, "20260830T051200Z-c3d4");
}

/// **The age is signed on purpose**: two machines' clocks disagreeing is a
/// fact about a distributed seat, not a malformed answer.
#[test]
fn an_age_may_be_negative_because_two_clocks_may_disagree() {
    let read = row(&json!({
        "root_id": "x", "display": "x", "state": "stopped", "preview": "",
        "age_secs": -3, "attention": 0, "members": 1, "depth": 0, "tone": "plain",
    }))
    .expect("a row");
    assert_eq!(read.age_secs, -3);
}

/// **Rung 3 on the state**, and the round trip through the label: an
/// unrecognised word paints as itself rather than as `quiescent`, which would
/// tell an operator nothing is happening on the strength of a word this build
/// has never seen.
#[test]
fn an_unknown_state_keeps_its_word() {
    for word in ["live", "in-flight", "quiescent", "stopped", "parked"] {
        assert_eq!(AgentState::of(word).label(), word);
    }
    assert_eq!(
        AgentState::of("parked"),
        AgentState::Unknown("parked".to_owned())
    );
}

/// The same on the tone, which is the other token field and not derivable from
/// the state beside it.
#[test]
fn an_unknown_tone_keeps_its_word() {
    for word in ["plain", "weak", "good", "bad", "live", "in-flight", "amber"] {
        assert_eq!(Tone::of(word).label(), word);
    }
    assert_eq!(Tone::of("amber"), Tone::Unknown("amber".to_owned()));
}

/// Rung 1 under rung 3: a row that is not an object, and a row missing a field
/// the list paints.
#[test]
fn a_row_that_is_not_a_row_refuses_by_name() {
    assert_eq!(
        row(&json!([])),
        Err("conversation row: not an object".to_owned())
    );
    let refusal =
        row(&json!({"root_id": "x", "display": "x", "state": "live"})).expect_err("no preview");
    assert!(refusal.contains("\"preview\""), "{refusal}");
}
