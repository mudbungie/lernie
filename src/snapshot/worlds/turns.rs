//! **A turn's shapes** (DESIGN §4.48 item 2): the working settled under the
//! answer, with the strip's `copy` door, and the working open while the turn
//! runs, a finished call compact in it — the two screens the working's line,
//! a step's door and the strip's door are on at all, so the walk's lit and
//! told instruments judge them.

use super::World;
use crate::reply::transcript::{Block, Entry, EntryKind, Usage};
use crate::test_support::window::{said, seated};

/// One entry of the fixture's turn.
fn entry(kind: EntryKind) -> Entry {
    Entry {
        name: "002-model-a.json".to_owned(),
        raw: "{}".to_owned(),
        kind,
    }
}

/// The turn's one call, and what it said back.
fn call() -> [Entry; 2] {
    [
        entry(EntryKind::Model {
            model_id: "model-a".to_owned(),
            blocks: vec![Block::ToolUse {
                id: "tu-1".to_owned(),
                name: "bash".to_owned(),
                input: "{\"command\":\"cargo test\"}".to_owned(),
            }],
            usage: Usage::new(),
        }),
        entry(EntryKind::ToolResult {
            tool_use_id: "tu-1".to_owned(),
            content: "Exit code: 0\ntest result: ok".to_owned(),
            is_error: false,
        }),
    ]
}

/// A turn of `tail` after the operator's words and the call.
fn turn(name: &'static str, tail: Option<Entry>) -> World {
    let mut model = seated();
    model.transcript.entries = [said("op", "port it")]
        .into_iter()
        .chain(call())
        .chain(tail)
        .collect();
    World { name, model }
}

/// **The turn settled**: the working one line, the answer, its strip.
pub(super) fn answered() -> World {
    let answer = entry(EntryKind::Model {
        model_id: "model-a".to_owned(),
        blocks: vec![Block::Text("the suite is green".to_owned())],
        usage: Usage::new(),
    });
    turn("answered", Some(answer))
}

/// **The turn running**: the working open, its call returned and compact.
pub(super) fn answering() -> World {
    turn("answering", None)
}
