//! The clients listing: the strict fields, the consent whose absence is a
//! reading, and the schema that rides through unread.

use crate::reply::{Read, Reply, read};
use serde_json::json;

/// A well-formed row with both kinds of tool on it, as the corpus carries one.
fn full() -> serde_json::Value {
    json!({"ok": true, "kind": "clients", "rows": [
        {"client": "laptop", "present": true, "tools": [
            {"name": "Bash", "description": "run a command",
             "input_schema": {"type": "object"}},
            {"name": "bash", "description": "run it where the caller says",
             "input_schema": {"type": "object"}, "subject_cwd": true}]},
        {"client": "phone", "present": false, "tools": []}]})
}

/// The listing carries both lifetimes on the row and the consent on the tool.
#[test]
fn a_row_carries_presence_the_set_and_each_tool_s_consent() {
    let Read::Answer(Reply::Clients(rows)) = read(&full()) else {
        panic!("a clients answer: {:?}", read(&full()));
    };
    assert_eq!(rows.len(), 2);
    assert_eq!(rows[0].client, "laptop");
    assert!(rows[0].present);
    assert_eq!(
        rows[0]
            .tools
            .iter()
            .map(|tool| (tool.name.clone(), tool.subject_cwd))
            .collect::<Vec<(String, bool)>>(),
        vec![("Bash".to_owned(), false), ("bash".to_owned(), true)]
    );
    assert!(!rows[1].present, "presence is a fact about the moment");
    assert!(rows[1].tools.is_empty(), "and the set is its own fact");
}

/// **A tool's `input_schema` rides through unread**, which is the vocabulary's
/// rung 4: it is the host's statement to a model, and nothing here paints it.
/// A row with none still reads, so the seat cannot be broken by a shape it
/// does not look at.
#[test]
fn the_schema_is_not_read_and_a_row_without_one_still_lands() {
    let bare = json!({"ok": true, "kind": "clients", "rows": [
        {"client": "laptop", "present": true,
         "tools": [{"name": "Bash", "description": "run a command"}]}]});
    assert!(matches!(read(&bare), Read::Answer(Reply::Clients(_))));
}

/// **Absent consent reads false and a mistyped one refuses** (REMOTE §5.1),
/// `null` included — upstream's own decoder refuses it, and two ends
/// disagreeing about what an absence is would be a consent read one way and
/// enforced the other.
#[test]
fn a_consent_that_is_not_a_boolean_refuses_and_names_the_field() {
    for wrong in [json!("yes"), json!(null), json!(1)] {
        let frame = json!({"ok": true, "kind": "clients", "rows": [
            {"client": "laptop", "present": true, "tools": [
                {"name": "Bash", "description": "d", "subject_cwd": wrong}]}]});
        let Read::Unreadable(why) = read(&frame) else {
            panic!("a mistyped consent refuses: {frame}");
        };
        assert!(why.contains("subject_cwd"), "{why}");
    }
}

/// Every required field refuses by name, which is the whole of rung 1: the
/// seat's reader is the only party that can say which key of which answer was
/// wrong.
#[test]
fn a_missing_field_refuses_and_names_itself() {
    for (frame, key) in [
        (
            json!({"ok": true, "kind": "clients", "rows": [{"present": true, "tools": []}]}),
            "client",
        ),
        (
            json!({"ok": true, "kind": "clients", "rows": [{"client": "a", "tools": []}]}),
            "present",
        ),
        (
            json!({"ok": true, "kind": "clients", "rows": [{"client": "a", "present": true}]}),
            "tools",
        ),
        (
            json!({"ok": true, "kind": "clients", "rows": [{"client": "a", "present": true,
                   "tools": [{"description": "d"}]}]}),
            "name",
        ),
        (
            json!({"ok": true, "kind": "clients", "rows": ["not an object"]}),
            "not a JSON object",
        ),
        (
            json!({"ok": true, "kind": "clients", "rows": [{"client": "a", "present": true,
                   "tools": ["not an object"]}]}),
            "not a JSON object",
        ),
    ] {
        let Read::Unreadable(why) = read(&frame) else {
            panic!("{key}: {frame}");
        };
        assert!(why.contains(key), "{key:?}: {why}");
    }
}

/// **What a row says of itself in one clause**, and the two words that are not
/// interchangeable: a machine that is not connected has not necessarily gone
/// away — a tool host holds its connection only while it waits for work.
#[test]
fn the_line_says_presence_and_how_much_is_offered() {
    let Read::Answer(Reply::Clients(rows)) = read(&full()) else {
        panic!("a clients answer");
    };
    assert_eq!(
        rows[0].line(),
        format!("laptop  — {}, 2 tool(s)", super::HERE)
    );
    assert_eq!(
        rows[1].line(),
        format!("phone  — {}, 0 tool(s)", super::AWAY)
    );
}
