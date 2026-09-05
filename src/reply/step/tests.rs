//! One step's drill-in: the two vocabularies, the bytes kept and the tree
//! dropped, and the strictness that names a field.

use serde_json::json;

use super::{Doc, UNPARSED, step};
use crate::reply::files::Preview;

/// The fixture's own shape: one record of every class, a stream, a tool call
/// and one capture log with bytes.
fn drill() -> serde_json::Value {
    json!({
        "seq": "001",
        "meta": {"kind": "json", "raw": "{\"commit\":\"abc\"}", "value": {"commit": "abc"}},
        "request": {"kind": "absent"},
        "staging": {"kind": "unparsed", "note": UNPARSED, "raw": "not json"},
        "response": [{"kind": "absent"}],
        "tools": [{"tool_id": "toolu_1", "is_error": false,
                   "input": {"kind": "absent"},
                   "output": {"kind": "unparsed", "note": UNPARSED, "raw": "raw"}}],
        "stderr": {"kind": "truncated", "text": "the adapter's last words", "size": 999_999}
    })
}

/// **The whole drill-in reads**, and the record vocabulary is the three the
/// engine writes.
#[test]
fn the_drill_in_carries_the_four_records_the_stream_and_the_tool_calls() {
    let read = step(drill().as_object().expect("an object")).expect("the drill-in reads");
    assert_eq!(read.seq, "001");
    assert_eq!(
        read.meta,
        Doc::Json {
            raw: "{\"commit\":\"abc\"}".to_owned()
        }
    );
    assert_eq!(read.request, Doc::Absent);
    assert_eq!(
        read.staging,
        Doc::Unparsed {
            note: UNPARSED.to_owned(),
            raw: "not json".to_owned()
        }
    );
    assert_eq!(read.response, vec![Doc::Absent]);
    assert_eq!(read.tools[0].tool_id, "toolu_1");
    assert!(!read.tools[0].is_error);
    assert_eq!(
        read.stderr,
        Some(Preview::Truncated {
            text: "the adapter's last words".to_owned(),
            size: 999_999
        })
    );
    assert_eq!(read.driver, None, "a log with no bytes is an absent key");
}

/// **The tree is not carried**, which is §4.9 spending itself: a record that
/// parsed keeps the bytes it parsed from, and the `value` beside them rides
/// through unread because nothing paints a tree.
#[test]
fn a_parsed_record_keeps_its_bytes_and_not_its_tree() {
    let mut malformed = drill();
    malformed["meta"]["value"] = json!("this is not the object it parsed from");
    let read = step(malformed.as_object().expect("an object")).expect("the tree is not read");
    assert_eq!(
        read.meta,
        Doc::Json {
            raw: "{\"commit\":\"abc\"}".to_owned()
        }
    );
}

/// **The two vocabularies are two.** A record's `kind` and a capture log's are
/// different closed sets, and an unknown word in either paints as itself
/// rather than refusing the whole drill-in.
#[test]
fn an_unknown_class_in_either_vocabulary_rides_verbatim() {
    let mut frame = drill();
    frame["request"] = json!({"kind": "sideways"});
    frame["stderr"] = json!({"kind": "sideways"});
    let read =
        step(frame.as_object().expect("an object")).expect("unknown classes are not refusals");
    assert_eq!(read.request, Doc::Unknown("sideways".to_owned()));
    assert_eq!(read.stderr, Some(Preview::Unknown("sideways".to_owned())));
}

/// Rung 1: a missing or mistyped field refuses, and names itself.
#[test]
fn a_malformed_drill_in_refuses_and_names_the_field() {
    let why = step(json!({"seq": "001"}).as_object().expect("an object"))
        .expect_err("a drill-in with no meta refuses");
    assert!(why.contains("meta"), "{why}");

    let mut frame = drill();
    frame["meta"] = json!("a string");
    let why = step(frame.as_object().expect("an object"))
        .expect_err("a record that is not an object refuses");
    assert!(why.contains("not an object"), "{why}");

    let mut frame = drill();
    frame["staging"] = json!({"kind": "unparsed", "raw": "x"});
    let why = step(frame.as_object().expect("an object"))
        .expect_err("an unparsed record with no note refuses");
    assert!(why.contains("note"), "{why}");

    let mut frame = drill();
    frame["tools"] = json!(["a string"]);
    let why = step(frame.as_object().expect("an object"))
        .expect_err("a tool call that is not an object refuses");
    assert!(why.contains("not an object"), "{why}");

    let mut frame = drill();
    frame["tools"][0]["output"] = json!(null);
    let why = step(frame.as_object().expect("an object")).expect_err("a null record refuses");
    assert!(why.contains("not an object"), "{why}");
}
