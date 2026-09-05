//! The chat pane as data: every entry kind, the two unreadables held apart, and
//! the live fold that replaces rather than accretes.

use super::{LIVE, NO_CONVERSATION, Row, render, rows};
use crate::reply::stream::{Delta, Stream};
use crate::reply::transcript::{Block, Entry, EntryKind, Transcript, Usage};
use crate::test_support::window::{pane, said, seated};
use crate::ui::Model;

fn of(kind: EntryKind) -> Transcript {
    Transcript {
        entries: vec![Entry {
            name: "003-model-a.json".to_owned(),
            raw: "{}".to_owned(),
            kind,
        }],
    }
}

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
        vec![Row {
            who: "op".to_owned(),
            said: "port it".to_owned(),
        }]
    );
    let ended = of(EntryKind::Delivered {
        sender: "child".to_owned(),
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
            "model-a → read tu-1",
            "model-a (citation, which this seat cannot read)",
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
        assert_eq!(rows(&entry, None)[0].who, format!("tu-1 {word}"));
    }
}

/// **The two unreadables are held apart.** "The engine could not read this" and
/// "this seat is behind" are different sentences, and only one of them is fixed
/// by an upgrade — but neither is dropped, because a transcript the operator
/// reads as shorter than it was is the one failure it must not have.
#[test]
fn the_engine_s_unreadable_and_this_seat_s_read_differently_and_both_are_kept() {
    let raw = rows(&of(EntryKind::Raw), None);
    assert!(raw[0].who.contains("unparsed"), "{:?}", raw[0]);
    let unknown = rows(&of(EntryKind::Unknown("annotation".to_owned())), None);
    assert!(
        unknown[0].who.contains("this seat cannot read"),
        "{:?}",
        unknown[0]
    );
    assert_ne!(raw[0].who, unknown[0].who);
    for shown in [&raw, &unknown] {
        assert_eq!(shown[0].said, "{}", "the bytes are kept either way");
    }
}

/// A compacted span asserts only what it is: which counter values went, and
/// whatever stood in their place.
#[test]
fn a_compacted_span_says_what_went() {
    let gap = of(EntryKind::Compacted {
        first: 4,
        last: 9,
        summary: "six were squashed".to_owned(),
    });
    assert_eq!(
        rows(&gap, None),
        vec![Row {
            who: "compacted 4–9".to_owned(),
            said: "six were squashed".to_owned(),
        }]
    );
}

/// **An empty half is no row, never a blank one**: a model that has only
/// thought so far, or one that answered without reasoning. A blank row would
/// claim something was said.
#[test]
fn each_half_of_a_turn_in_flight_is_its_own_row_and_an_empty_one_is_none() {
    let both = of(EntryKind::Streaming {
        thinking: "weighing".to_owned(),
        text: "so far".to_owned(),
    });
    assert_eq!(
        rows(&both, None)
            .into_iter()
            .map(|r| r.who)
            .collect::<Vec<String>>(),
        vec![format!("{LIVE} (thinking)"), LIVE.to_owned()]
    );
    let quiet = of(EntryKind::Streaming {
        thinking: String::new(),
        text: String::new(),
    });
    assert!(rows(&quiet, None).is_empty());
}

/// **The newest fold wins.** The tail reaches a seat by two routes at two
/// cadences, and *replace* is the only reconciliation either needs — appending
/// would paint the answer twice.
#[test]
fn a_live_fold_replaces_the_streaming_entry_rather_than_standing_beside_it() {
    let committed = Transcript {
        entries: vec![
            said("op", "port it"),
            Entry {
                name: "«live»".to_owned(),
                raw: "half a".to_owned(),
                kind: EntryKind::Streaming {
                    thinking: String::new(),
                    text: "half a".to_owned(),
                },
            },
        ],
    };
    let newer = Stream {
        text: Some("half a sentence".to_owned()),
        thinking: None,
        last_delta: Some(Delta::Text),
    };
    let shown = rows(&committed, Some(&newer));
    assert_eq!(shown.len(), 2, "{shown:?}");
    assert_eq!(shown[1].said, "half a sentence");
    // With no newer fold the committed one still paints: the pull read's own.
    assert_eq!(rows(&committed, None)[1].said, "half a");
}

/// The pane says what it is waiting for rather than showing an empty box.
#[test]
fn the_pane_says_what_it_is_waiting_for() {
    let mut nothing = Model::default();
    assert!(pane(|ui| render(ui, &nothing)).contains(NO_CONVERSATION));
    nothing = seated();
    let painted = pane(|ui| render(ui, &nothing));
    assert!(painted.contains("port it"), "{painted}");
}

/// **The committed path obeys the live path's rule, because there is one rule.**
///
/// The engine emits `{"kind":"thinking","text":""}` as an ordinary block, so a
/// committed model turn can carry a half with nothing in it exactly as a turn
/// in flight can. It painted a `(thinking)` header over nothing (bl-beb7) —
/// which reads as *the model thought something and this seat lost it* — until
/// both paths went through the one rule.
#[test]
fn a_committed_half_with_nothing_in_it_is_no_row_either() {
    let turn = of(EntryKind::Model {
        model_id: "model-a".to_owned(),
        blocks: vec![
            Block::Thinking(String::new()),
            Block::Text("the seam is real".to_owned()),
        ],
        usage: Usage::new(),
    });
    assert_eq!(
        rows(&turn, None),
        vec![Row {
            who: "model-a".to_owned(),
            said: "the seam is real".to_owned(),
        }]
    );
    // And the other half, the same way: a turn that has only thought so far.
    let thinking_only = of(EntryKind::Model {
        model_id: "model-a".to_owned(),
        blocks: vec![
            Block::Thinking("weighing two seams".to_owned()),
            Block::Text(String::new()),
        ],
        usage: Usage::new(),
    });
    assert_eq!(
        rows(&thinking_only, None)
            .into_iter()
            .map(|r| r.who)
            .collect::<Vec<String>>(),
        vec!["model-a (thinking)".to_owned()]
    );
}

/// **Dropping an empty half must not become dropping an empty row.** A tool
/// call with no input still happened, and an unreadable block is deliberately
/// blank — the rung-3 surfacing the module header promises. Both keep their
/// rows, so the fix above cannot grow into the opposite defect.
#[test]
fn a_blank_row_that_is_not_a_half_of_a_turn_is_kept() {
    let turn = of(EntryKind::Model {
        model_id: "model-a".to_owned(),
        blocks: vec![
            Block::ToolUse {
                id: "tu-1".to_owned(),
                name: "list".to_owned(),
                input: String::new(),
            },
            Block::Unknown("citation".to_owned()),
        ],
        usage: Usage::new(),
    });
    let shown = rows(&turn, None);
    assert_eq!(shown.len(), 2, "{shown:?}");
    assert!(shown.iter().all(|r| r.said.is_empty()), "{shown:?}");
}

/// **On the glass**, where the defect was read: the header of an empty half
/// does not reach it, and the answer beside it does.
#[test]
fn the_header_of_an_empty_half_never_reaches_the_glass() {
    let mut model = seated();
    model.transcript = of(EntryKind::Model {
        model_id: "model-a".to_owned(),
        blocks: vec![
            Block::Thinking(String::new()),
            Block::Text("the seam is real".to_owned()),
        ],
        usage: Usage::new(),
    });
    let painted = pane(|ui| render(ui, &model));
    assert!(painted.contains("the seam is real"), "{painted}");
    assert!(!painted.contains("(thinking)"), "{painted}");
}
