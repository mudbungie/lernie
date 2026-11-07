//! The chat pane as data: every entry kind, the two unreadables held apart, and
//! the live fold that replaces rather than accretes.
//!
//! Where the pane OPENS and what it follows is [`anchor`], split off because
//! it is the one question here answered on the glass rather than in the row
//! projection: a scroll offset is not a value `rows` can be asked for.

/// The tail anchor, the follow, and the scroll that releases it.
mod anchor;
/// The four bubbles, read off the glass.
mod bubble;
/// The idiom: gutters, the trail under its fold, and what parts turns.
mod idiom;
/// The two knobs the chat reads: bubbles off, and the fold off.
mod knobs;
/// The working's one line: its counts by kind, and its width-wide door.
mod line;
/// The live fold: where it paints, and when it has nothing to paint over.
mod live;
/// A machine's answer on the glass: folded, and one gesture from the whole.
mod machinery;
/// A toggle's memory is one conversation's (§4.45 item 5; bl-b83d).
mod memory;
/// The bubbles nested: the working's, its steps', a call's card, and the
/// turn strip's `copy` door (§4.48 item 2).
mod nested;
/// A procedure run as a marker on the glass (§4.47).
mod procedures;
/// Each committed entry kind, projected (bl-f6b5).
mod projection;
/// When a turn runs, read off the transcript's shape; and the working while
/// a tool executes (bl-2101).
mod running;
/// A message this seat sent: on the glass the frame it is sent, in the
/// pending ink until acknowledged, and gone when the server's row lands.
mod sent;
/// The working's three stages: whole in flight, compact once done, one line
/// once the answer stands — and each a door.
mod stages;
/// What the weight at a block's edge says about who is speaking.
mod weight;

use super::{LIVE, Row, WAYS_IN, render, rows};
use crate::reply::transcript::{Block, Entry, EntryKind, Transcript, Usage};
use crate::test_support::window::{pane, seated};
use crate::ui::Model;
use crate::ui::theme::Speaker;

pub(super) fn of(kind: EntryKind) -> Transcript {
    Transcript {
        entries: vec![Entry {
            name: "003-model-a.json".to_owned(),
            raw: "{}".to_owned(),
            kind,
        }],
    }
}

/// **The two unreadables are held apart.** "The server could not read this" and
/// "this seat is behind" are different sentences, and only one of them is fixed
/// by an upgrade — but neither is dropped, because a transcript the operator
/// reads as shorter than it was is the one failure it must not have.
#[test]
fn the_server_s_unreadable_and_this_seat_s_read_differently_and_both_are_kept() {
    let raw = rows(&of(EntryKind::Raw), None);
    assert!(raw[0].who.contains("unparsed"), "{:?}", raw[0]);
    let unknown = rows(&of(EntryKind::Unknown("annotation".to_owned())), None);
    assert!(
        unknown[0]
            .who
            .contains("this version of lernie cannot read"),
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
        vec![
            Row::plain(
                "compacted 4–9".to_owned(),
                "six were squashed".to_owned(),
                Speaker::Peer,
            )
            .from("003-model-a.json")
        ]
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

/// **The empty column is the mark and one line** (DESIGN §4.48 item 1):
/// with nothing aimed, and with a workspace aimed and nothing open, the one
/// line naming the ways in is the only words on it — and the mark stands
/// over it, centred, larger than the head's.
#[test]
fn the_empty_column_is_the_mark_and_one_line() {
    for nothing in [
        Model::default(),
        Model {
            conversation: None,
            ..seated()
        },
    ] {
        let window = crate::paint_probe::frame::Window::sized(900.0, 700.0);
        let output = window.frame(Vec::new(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| render(ui, &nothing));
        });
        let seen = crate::paint_probe::seen_of(&output);
        let words: Vec<&str> = seen.iter().map(|run| run.text.as_str()).collect();
        assert_eq!(words.join(" "), WAYS_IN, "{words:?}");
        let line = &seen[0].laid;
        assert!((line.center().x - 450.0).abs() < 2.0, "centred: {line:?}");
        let mark = output
            .shapes
            .iter()
            .filter_map(|clipped| match &clipped.shape {
                egui::Shape::Circle(circle) => Some(circle.center),
                _ => None,
            })
            .collect::<Vec<_>>();
        assert!(!mark.is_empty(), "the mark is painted");
        assert!(mark.iter().all(|at| at.y < line.min.y), "over the line");
    }
    let shown = seated();
    let painted = pane(|ui| {
        render(ui, &shown);
    });
    assert!(painted.contains("port it"), "{painted}");
    assert!(!painted.contains(WAYS_IN), "{painted}");
}

/// **The committed path obeys the live path's rule, because there is one rule.**
///
/// The server emits `{"kind":"thinking","text":""}` as an ordinary block, so a
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
        vec![
            Row::plain(
                "model-a".to_owned(),
                "the seam is real".to_owned(),
                Speaker::Model,
            )
            .from("003-model-a.json")
        ]
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
    let painted = pane(|ui| {
        render(ui, &model);
    });
    assert!(painted.contains("the seam is real"), "{painted}");
    assert!(!painted.contains("(thinking)"), "{painted}");
}
