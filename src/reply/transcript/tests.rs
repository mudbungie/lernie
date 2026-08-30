//! The conversation: every entry kind, the two ways an entry can be
//! unreadable, and the epitaph that is carried rather than re-derived.

use super::{Block, Entry, EntryKind, transcript};
use serde_json::{Value, json};

fn read(v: &Value) -> Result<super::Transcript, String> {
    transcript(v.as_object().expect("an object"))
}

/// One frame of every kind, in message order.
#[test]
fn every_entry_kind_reads_back_as_itself() {
    let answered = read(&json!({"ok": true, "kind": "transcript", "rows": [
        {"name": "001-op.md", "raw": "port it\n", "kind": "delivered",
         "sender": "op", "body": "port it"},
        {"name": "002-model-a.json", "raw": "{}", "kind": "model",
         "model_id": "model-a",
         "blocks": [{"kind": "text", "text": "on it"}],
         "usage": {"input_tokens": 12}},
        {"name": "003-tool.json", "raw": "{}", "kind": "tool-result",
         "tool_use_id": "tu-1", "content": "done", "is_error": false},
        {"name": "«live»", "raw": "so far", "kind": "streaming",
         "thinking": "", "text": "so far"},
        {"name": "«compacted»", "raw": "", "kind": "compacted",
         "first": 4, "last": 9, "summary": "six were squashed"},
        {"name": "005-unreadable.bin", "raw": "??", "kind": "raw"},
    ]}))
    .expect("a transcript");
    let kinds: Vec<EntryKind> = answered.entries.iter().map(|e| e.kind.clone()).collect();
    assert_eq!(
        kinds,
        vec![
            EntryKind::Delivered {
                sender: "op".to_owned(),
                epitaph: None,
                body: "port it".to_owned(),
            },
            EntryKind::Model {
                model_id: "model-a".to_owned(),
                blocks: vec![Block::Text("on it".to_owned())],
                usage: [("input_tokens".to_owned(), 12)].into_iter().collect(),
            },
            EntryKind::ToolResult {
                tool_use_id: "tu-1".to_owned(),
                content: "done".to_owned(),
                is_error: false,
            },
            EntryKind::Streaming {
                thinking: String::new(),
                text: "so far".to_owned(),
            },
            EntryKind::Compacted {
                first: 4,
                last: 9,
                summary: "six were squashed".to_owned(),
            },
            EntryKind::Raw,
        ]
    );
    assert_eq!(answered.entries[0].name, "001-op.md");
    assert_eq!(answered.entries[0].raw, "port it\n");
}

/// **The epitaph rides as its label, not as an enum this seat mirrors.** The
/// engine's own reader for it is total, so a closed set here would be stricter
/// than the authority this crate implements against.
#[test]
fn an_epitaph_is_carried_verbatim_including_one_this_build_never_saw() {
    for word in ["delivered", "abandoned", "a-word-from-a-newer-engine"] {
        let answered = read(&json!({"rows": [
            {"name": "002-child.md", "raw": "landed", "kind": "delivered",
             "sender": "child", "epitaph": word, "body": "landed"},
        ]}))
        .expect("a transcript");
        let EntryKind::Delivered { epitaph, .. } = &answered.entries[0].kind else {
            panic!("a delivered entry");
        };
        assert_eq!(epitaph.as_deref(), Some(word));
    }
}

/// **Rung 3 on the entry kind, and it is held apart from `raw`.** "The engine
/// could not read this" and "this seat is behind" are different sentences, and
/// only one of them is fixed by an upgrade — so an entry of an unknown kind
/// keeps its word rather than becoming [`EntryKind::Raw`].
#[test]
fn an_unknown_entry_kind_keeps_its_word_and_is_not_raw() {
    let answered = read(&json!({"rows": [
        {"name": "006-annotation.json", "raw": "{}", "kind": "annotation"},
    ]}))
    .expect("a transcript");
    assert_eq!(
        answered.entries,
        vec![Entry {
            name: "006-annotation.json".to_owned(),
            raw: "{}".to_owned(),
            kind: EntryKind::Unknown("annotation".to_owned()),
        }]
    );
}

/// Rung 1: a row that is not an object, a row with no bytes behind it, and a
/// kind whose own fields are missing.
#[test]
fn an_entry_that_will_not_read_refuses_by_name() {
    for (rows, said) in [
        (json!([7]), "not an object"),
        (json!([{"name": "x", "kind": "raw"}]), "\"raw\""),
        (
            json!([{"name": "x", "raw": "", "kind": "delivered", "sender": "op"}]),
            "\"body\"",
        ),
        (
            json!([{"name": "x", "raw": "", "kind": "model", "model_id": "m",
                    "blocks": []}]),
            "\"usage\"",
        ),
    ] {
        let refusal = read(&json!({"rows": rows})).expect_err("refused");
        assert!(refusal.contains(said), "{rows}: {refusal}");
    }
}
