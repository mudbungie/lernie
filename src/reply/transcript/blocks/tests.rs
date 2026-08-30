//! The canonical blocks and the counters beside them — the two halves of what
//! one model entry says.

use super::{Block, block, usage};
use serde_json::{Value, json};

fn counters(v: &Value) -> Result<super::Usage, String> {
    usage(v.as_object().expect("an object"))
}

/// The three blocks this build knows, and the round trip through the label.
/// Reasoning is a **row**, not a spinner: a badge that never grows cannot tell
/// a model thinking hard from a driver that has hung.
#[test]
fn the_three_known_blocks_read_back_as_themselves() {
    assert_eq!(
        block(&json!({"kind": "text", "text": "on it"})),
        Ok(Block::Text("on it".to_owned()))
    );
    assert_eq!(
        block(&json!({"kind": "thinking", "text": "weighing two seams"})),
        Ok(Block::Thinking("weighing two seams".to_owned()))
    );
    assert_eq!(
        block(&json!({"kind": "tool-use", "id": "tu-1", "name": "read",
                      "input": "src/reply.rs"})),
        Ok(Block::ToolUse {
            id: "tu-1".to_owned(),
            name: "read".to_owned(),
            input: "src/reply.rs".to_owned(),
        })
    );
    for (kind, expected) in [
        (json!({"kind": "text", "text": ""}), "text"),
        (json!({"kind": "thinking", "text": ""}), "thinking"),
        (
            json!({"kind": "tool-use", "id": "", "name": "", "input": ""}),
            "tool-use",
        ),
    ] {
        assert_eq!(block(&kind).expect("a block").label(), expected);
    }
}

/// **Rung 3**: a block kind this build does not know paints as its own word.
/// A block silently dropped is a turn the operator reads as shorter than it
/// was, which is the one failure a transcript must not have.
#[test]
fn an_unknown_block_kind_keeps_its_word() {
    let read = block(&json!({"kind": "citation", "text": "…"})).expect("a block");
    assert_eq!(read, Block::Unknown("citation".to_owned()));
    assert_eq!(read.label(), "citation");
}

/// Rung 1 on a block: not an object, and a known kind missing its own field.
#[test]
fn a_block_that_will_not_read_refuses() {
    assert_eq!(
        block(&json!("text")),
        Err("block: not an object".to_owned())
    );
    let refusal = block(&json!({"kind": "tool-use", "id": "tu-1"})).expect_err("no name");
    assert!(refusal.contains("\"name\""), "{refusal}");
}

/// **No provider vocabulary is pinned**: a counter name is whatever the
/// provider called it, so one the adapter starts reporting rides through with
/// no edit at all.
#[test]
fn counters_ride_under_the_provider_s_own_names() {
    let read = counters(&json!({"usage": {
        "input_tokens": 1841, "output_tokens": 96, "a_counter_added_later": 7,
    }}))
    .expect("counters");
    assert_eq!(read.get("a_counter_added_later"), Some(&7));
    assert_eq!(read.len(), 3);
}

/// **Empty is the general path and it is a reading**: an entry from before
/// counters were sealed, or a provider that reported none. The object is
/// required so that "reported nothing" stays distinct from an answer this
/// codec failed to read; a zero would be a lie.
#[test]
fn an_entry_that_reported_no_counters_says_so_with_an_empty_object() {
    assert!(
        counters(&json!({"usage": {}}))
            .expect("counters")
            .is_empty()
    );
    let refusal = counters(&json!({})).expect_err("absent");
    assert!(refusal.contains("\"usage\""), "{refusal}");
    let wrong = counters(&json!({"usage": {"input_tokens": "many"}})).expect_err("not a count");
    assert!(wrong.contains("not a count"), "{wrong}");
}
