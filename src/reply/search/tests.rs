//! One search answer: the four facts about a match, the four address fields
//! that are optional because a hit is one of three shapes, and the unreadable
//! list that is a different claim from finding nothing.

use super::{Found, Hit, KIND, found, row};
use crate::reply::{Read, Reply, read};
use serde_json::{Value, json};

/// A conversation hit, as the corpus carries one.
fn conversation() -> Value {
    json!({
        "at": "conversation",
        "agent": "c-1",
        "workspace": "ws",
        "field": "text",
        "excerpt": "the gate",
        "offset": 12,
    })
}

/// The whole answer, as the corpus carries it.
fn answer() -> Value {
    json!({
        "kind": KIND,
        "ok": true,
        "needle": "gate",
        "rows": [
            {"at": "ball", "project": "p", "id": "bl-1", "field": "name",
             "excerpt": "bl-1", "offset": 0},
            conversation(),
        ],
        "unreadable": ["p: balls unlistable"],
    })
}

#[test]
fn a_hit_carries_what_it_is_in_where_in_it_and_the_words_around_it() {
    let read = row(&conversation()).expect("a search row");
    assert_eq!(
        read,
        Hit {
            at: "conversation".to_owned(),
            field: "text".to_owned(),
            excerpt: "the gate".to_owned(),
            offset: 12,
            project: None,
            id: None,
            workspace: Some("ws".to_owned()),
            agent: Some("c-1".to_owned()),
        }
    );
    assert_eq!(read.subject(), "conversation  ws  c-1");
    assert_eq!(read.at_field(), "text +12");
}

/// **The address fields are read as options, never branched on `at`.** A ball
/// hit names a project and an id, a workspace hit names a workspace, and a
/// subject a newer engine grew would fall off any closed set this end kept.
#[test]
fn each_shape_of_hit_reads_and_the_subject_is_whatever_it_named() {
    for (frame, subject) in [
        (
            json!({"at": "ball", "project": "p", "id": "bl-1", "field": "name",
                   "excerpt": "bl-1", "offset": 0}),
            "ball  p  bl-1",
        ),
        (
            json!({"at": "workspace", "workspace": "ws", "field": "summary",
                   "excerpt": "ws", "offset": 3}),
            "workspace  ws",
        ),
        (
            json!({"at": "tekeli-li", "field": "f", "excerpt": "e", "offset": 1}),
            "tekeli-li",
        ),
    ] {
        assert_eq!(row(&frame).expect("a hit").subject(), subject);
    }
}

/// **Rung 1 refuses by name.** The four facts about the match are required;
/// an address field of the wrong type still refuses rather than reading as
/// absent.
#[test]
fn the_match_facts_are_required_and_a_bad_address_still_refuses() {
    for (field, why) in [
        ("at", "non-string"),
        ("field", "non-string"),
        ("excerpt", "non-string"),
        ("offset", "non-integer"),
    ] {
        let mut frame = conversation();
        frame[field] = json!(null);
        let said = row(&frame).expect_err(field);
        assert!(said.contains(field), "{said}");
        assert!(said.contains(why), "{said}");
    }
    let mut frame = conversation();
    frame["workspace"] = json!(7);
    let said = row(&frame).expect_err("a non-string workspace");
    assert!(said.contains("workspace"), "{said}");
    let said = row(&json!([])).expect_err("a row that is not an object");
    assert!(said.contains("not an object"), "{said}");
}

/// **What could not be read is its own field and its own sentence.** Folding
/// it into an empty result would report a clean search over a store nothing
/// ever opened.
#[test]
fn the_answer_keeps_the_needle_the_hits_and_what_it_could_not_read_apart() {
    let obj = answer();
    let read = found(obj.as_object().expect("the answer")).expect("a search answer");
    assert_eq!(
        read,
        Found {
            needle: "gate".to_owned(),
            rows: vec![
                Hit {
                    at: "ball".to_owned(),
                    field: "name".to_owned(),
                    excerpt: "bl-1".to_owned(),
                    offset: 0,
                    project: Some("p".to_owned()),
                    id: Some("bl-1".to_owned()),
                    workspace: None,
                    agent: None,
                },
                row(&conversation()).expect("the conversation hit"),
            ],
            unreadable: vec!["p: balls unlistable".to_owned()],
        }
    );
}

/// Every required field of the answer itself refuses by name, including a
/// non-string entry in the unreadable list.
#[test]
fn the_answers_own_fields_refuse_by_name() {
    for (field, needle) in [
        ("needle", "needle"),
        ("rows", "rows"),
        ("unreadable", "unreadable"),
    ] {
        let mut frame = answer();
        frame[field] = json!(7);
        let obj = frame.as_object().expect("the answer").clone();
        let said = found(&obj).expect_err(field);
        assert!(said.contains(needle), "{said}");
    }
    let mut frame = answer();
    frame["unreadable"] = json!([7]);
    let obj = frame.as_object().expect("the answer").clone();
    let said = found(&obj).expect_err("a non-string reason");
    assert!(said.contains("unreadable"), "{said}");
}

/// The whole frame, through the real door.
#[test]
fn the_frame_reads_as_the_answer() {
    let frame = answer();
    let Read::Answer(Reply::Found(read)) = read(&frame) else {
        panic!("a search frame is an answer: {:?}", read(&frame));
    };
    assert_eq!(read.rows.len(), 2);
}
