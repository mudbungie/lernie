//! The queue row: the strict fields, the three absences that are readings, the
//! nested objects, and the signal vocabulary carried verbatim.

use super::{Flag, Held, KIND, QueueRow, row};
use crate::reply::convs::AgentState;
use crate::reply::{Read, Reply, read};
use serde_json::json;

/// A well-formed row with everything present, as the corpus carries one.
fn full() -> serde_json::Value {
    json!({
        "workspace": "ws", "agent": "c-1", "display": "Cobalt",
        "state": "stopped", "uncertain": false, "preview": "p",
        "age_secs": 5, "pending": 2,
        "signals": ["held", "mail", "flagged"],
        "failure": "Unauthorized",
        "flag": {"at": "2026-01-01T00:00:00Z", "reason": "please look at this one"},
        "held": {"tool": "Bash", "tool_use": "toolu_1", "reason": "writes"},
    })
}

/// The bare row: everything the wire spells `null` read as an absence.
fn bare() -> serde_json::Value {
    json!({
        "workspace": "ws", "agent": "c-2", "display": "Dun",
        "state": "live", "uncertain": true, "preview": "",
        "age_secs": 0, "pending": 0, "signals": [],
        "failure": null, "flag": null, "held": null,
    })
}

#[test]
fn a_row_carries_the_whole_ask() {
    assert_eq!(
        row(&full()).expect("a queue row"),
        QueueRow {
            workspace: "ws".to_owned(),
            agent: "c-1".to_owned(),
            display: "Cobalt".to_owned(),
            state: AgentState::Stopped,
            uncertain: false,
            preview: "p".to_owned(),
            age_secs: 5,
            pending: 2,
            signals: vec!["held".to_owned(), "mail".to_owned(), "flagged".to_owned()],
            failure: Some("Unauthorized".to_owned()),
            flag: Some(Flag {
                at: "2026-01-01T00:00:00Z".to_owned(),
                reason: "please look at this one".to_owned(),
            }),
            held: Some(Held {
                tool: "Bash".to_owned(),
                tool_use: "toolu_1".to_owned(),
                reason: "writes".to_owned(),
            }),
        }
    );
}

/// **`null` is an absence and never an empty value.** *Nobody flagged this* and
/// *somebody flagged it and left no words* are two claims, and the reading
/// keeps them two — the same for the failure clause and the parked invocation.
#[test]
fn the_three_nullable_facts_read_as_absences() {
    let read = row(&bare()).expect("a bare queue row");
    assert_eq!((read.failure, read.flag, read.held), (None, None, None));
    assert!(read.signals.is_empty());
    // An absent key reads the same as an explicit null: a reader must not be
    // made to tell an encoder's two spellings of *nothing* apart.
    let mut absent = bare();
    for key in ["failure", "flag", "held"] {
        absent
            .as_object_mut()
            .expect("the row")
            .remove(key)
            .expect("the null");
    }
    let read = row(&absent).expect("a queue row with the keys left out");
    assert_eq!((read.failure, read.flag, read.held), (None, None, None));
}

/// **Rung 1 refuses by name**, on the row's own fields, on each nested object's,
/// and on a nested value that is not an object at all.
#[test]
fn every_required_field_refuses_and_says_which_one() {
    for key in [
        "workspace",
        "agent",
        "display",
        "state",
        "uncertain",
        "preview",
        "age_secs",
        "pending",
        "signals",
    ] {
        let mut frame = full();
        frame.as_object_mut().expect("the row").remove(key);
        let why = row(&frame).expect_err("a row missing a required field");
        assert!(why.contains(key), "{key}: {why}");
    }
    for (nest, key) in [
        ("flag", "at"),
        ("flag", "reason"),
        ("held", "tool"),
        ("held", "tool_use"),
        ("held", "reason"),
    ] {
        let mut frame = full();
        frame[nest].as_object_mut().expect("the object").remove(key);
        let why = row(&frame).expect_err("a nested object missing a field");
        assert!(why.contains(key), "{nest}/{key}: {why}");
    }
    for nest in ["flag", "held"] {
        let mut frame = full();
        frame[nest] = json!("not an object");
        let why = row(&frame).expect_err("a nested field of the wrong type");
        assert!(why.contains(nest), "{nest}: {why}");
    }
    let why = row(&json!([])).expect_err("a row that is not an object");
    assert!(why.contains("not an object"), "{why}");
    let mut frame = full();
    frame["signals"] = json!([7]);
    let why = row(&frame).expect_err("a signal that is not a string");
    assert!(why.contains("signal"), "{why}");
}

/// **Rung 3 on the state**, which is the conversation list's own vocabulary
/// read one noun over: a word this build does not know keeps its spelling.
#[test]
fn an_unknown_state_keeps_its_word() {
    let mut frame = full();
    frame["state"] = json!("becalmed");
    assert_eq!(
        row(&frame).expect("a queue row").state,
        AgentState::Unknown("becalmed".to_owned())
    );
}

/// The two kinds land as answers off the real reader, which is what the corpus
/// move to `answers/` asserts one level up.
#[test]
fn the_queue_and_its_raise_are_answers() {
    let listing = read(&json!({"kind": KIND, "ok": true, "rows": [full(), bare()]}));
    let Read::Answer(Reply::Attention(rows)) = listing else {
        panic!("a queue answer, got {listing:?}");
    };
    assert_eq!(rows.len(), 2);
    assert_eq!(
        read(&json!({"kind": "flagged", "ok": true})),
        Read::Answer(Reply::Flagged)
    );
}
