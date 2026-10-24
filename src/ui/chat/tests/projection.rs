//! **Each committed entry kind as rows** (§4.9, bl-f6b5): a delivered
//! message and its epitaph, a model turn block by block, a tool result and
//! whether it failed — the projection seam, split from the pane's own tests
//! at the design-time budget (bl-1d79).

use super::super::{Row, rows};
use super::of;
use crate::reply::transcript::{Block, EntryKind, Transcript, Usage};
use crate::test_support::window::said;
use crate::ui::theme::Speaker;

/// A delivered message, and the epitaph that says how a child ended.
#[test]
fn a_delivered_message_names_its_sender_and_its_ending() {
    assert_eq!(
        rows(
            &Transcript {
                entries: vec![said("op", "port it")]
            },
            None
        ),
        vec![
            Row::plain("op".to_owned(), "port it".to_owned(), Speaker::Operator,).from("001-op.md")
        ]
    );
    let ended = of(EntryKind::Delivered {
        sender: "child".to_owned(),
        sender_name: None,
        epitaph: Some("delivered".to_owned()),
        body: "landed".to_owned(),
    });
    assert_eq!(rows(&ended, None)[0].who, "child (delivered)");
}

/// **Reasoning is a row, not a spinner.** A badge that never grows cannot tell
/// a model thinking hard from a driver that has hung, so each block becomes its
/// own row and each says which kind it is.
#[test]
fn a_model_turn_becomes_one_row_per_block() {
    let turn = of(EntryKind::Model {
        model_id: "model-a".to_owned(),
        blocks: vec![
            Block::Thinking("weighing two seams".to_owned()),
            Block::Text("the seam is real".to_owned()),
            Block::ToolUse {
                id: "tu-1".to_owned(),
                name: "read".to_owned(),
                input: "src/ui.rs".to_owned(),
            },
            Block::Unknown("citation".to_owned()),
        ],
        usage: Usage::new(),
    });
    let who: Vec<String> = rows(&turn, None).into_iter().map(|r| r.who).collect();
    assert_eq!(
        who,
        vec![
            "model-a (thinking)",
            "model-a",
            "read(src/ui.rs)",
            "model-a (citation, which this version of lernie cannot read)",
        ]
    );
}

/// A tool result says which way it went, because a failure the operator has to
/// read the body to spot is a failure they will miss.
#[test]
fn a_tool_result_says_whether_it_failed() {
    for (is_error, word) in [(false, "returned"), (true, "failed")] {
        let entry = of(EntryKind::ToolResult {
            tool_use_id: "tu-1".to_owned(),
            content: "done".to_owned(),
            is_error,
        });
        assert_eq!(rows(&entry, None)[0].who, word);
    }
}
