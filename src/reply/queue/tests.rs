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
        "says": "parked a tool invocation for your answer",
        "failure": "Unauthorized",
        "flag": {"at": "2026-01-01T00:00:00Z", "reason": "please look at this one"},
        "held": {"tool": "Bash", "tool_use": "toolu_1", "reason": "writes", "scopes": ["call"]},
    })
}

/// The bare row: everything the wire spells `null` read as an absence.
fn bare() -> serde_json::Value {
    json!({
        "workspace": "ws", "agent": "c-2", "display": "Dun",
        "state": "live", "uncertain": true, "preview": "",
        "age_secs": 0, "pending": 0, "signals": [], "says": "",
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
            says: Some("parked a tool invocation for your answer".to_owned()),
            failure: Some("Unauthorized".to_owned()),
            flag: Some(Flag {
                at: "2026-01-01T00:00:00Z".to_owned(),
                reason: "please look at this one".to_owned(),
            }),
            held: Some(Held {
                tool: "Bash".to_owned(),
                tool_use: "toolu_1".to_owned(),
                reason: "writes".to_owned(),
                scopes: vec!["call".to_owned()],
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

/// **The held row's `scopes` is an absence, not a requirement** (yog
/// bl-0da6; bl-da16): a held object without the key — every engine before it
/// — reads as an empty list, *the engine did not say*; one that carries it
/// reads verbatim, a word this build does not know included (rung 3, the
/// filter is the offer's); and one of the wrong shape refuses by name (rung 1).
#[test]
fn the_held_row_s_scopes_are_read_where_said_and_empty_where_not() {
    let mut frame = full();
    frame["held"]
        .as_object_mut()
        .expect("the object")
        .remove("scopes");
    let read = row(&frame).expect("a queue row");
    assert_eq!(read.held.expect("held").scopes, Vec::<String>::new());
    frame["held"]["scopes"] = json!(["call", "everywhere"]);
    let read = row(&frame).expect("a queue row");
    assert_eq!(read.held.expect("held").scopes, ["call", "everywhere"]);
    for wrong in [json!("call"), json!([7])] {
        frame["held"]["scopes"] = wrong;
        let why = row(&frame).expect_err("scopes of the wrong shape");
        assert!(why.contains("scopes"), "{why}");
    }
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

/// **`says` is the engine's sentence, and the signals stand in only where it
/// wrote none** (bl-5507): an empty string and a missing key are both absence
/// (rung 4 reads a key an older engine never wrote as nothing said).
#[test]
fn the_engine_s_sentence_wins_and_the_signals_stand_in_without_it() {
    let whole = row(&full()).expect("a queue row");
    assert_eq!(
        whole.why(", ").as_deref(),
        Some("parked a tool invocation for your answer")
    );
    let mut unsaid = full();
    unsaid.as_object_mut().expect("object").remove("says");
    let unsaid = row(&unsaid).expect("a queue row");
    assert_eq!(unsaid.says, None);
    assert_eq!(unsaid.why(", ").as_deref(), Some("held, mail, flagged"));
    let quiet = row(&bare()).expect("a queue row");
    assert_eq!((quiet.says.clone(), quiet.why(", ")), (None, None));
}

/// **`seen`'s receipt is an answer**: the item acted on and the queue that
/// remains, in the same rows (bl-6802).
#[test]
fn the_receipt_of_seen_carries_the_queue_that_remains() {
    let frame = json!({"kind": "acknowledged", "ok": true, "workspace": "ws",
                       "agent": "c-9", "rows": [full(), bare()]});
    let Read::Answer(Reply::Acknowledged(acked)) = read(&frame) else {
        panic!("an acknowledged answer");
    };
    assert_eq!(
        (acked.workspace.as_str(), acked.agent.as_str()),
        ("ws", "c-9")
    );
    assert_eq!(
        acked.rows,
        vec![row(&full()).expect("row"), row(&bare()).expect("row")]
    );
}
