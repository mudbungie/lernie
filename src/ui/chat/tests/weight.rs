//! **Who is speaking, read off the glass** (STYLE §5): the rule beside each
//! block, the ink of the header over it, and the one row that wears a state
//! instead of a weight.
//!
//! A weight is not a word, so nothing in the row projection can be asked what
//! reached the operator — the rule is a fill and the header is an ink, and
//! both are only on the frame. These beats read the frame.

use crate::paint_probe::frame::Window;
use crate::paint_probe::{fills_of, seen_of};
use crate::reply::transcript::{Block, Entry, EntryKind, Transcript, Usage};
use crate::test_support::window::{numbered, said, seated};
use crate::ui::Model;
use crate::ui::theme;

/// One entry of the shape the wire hands over.
fn entry(kind: EntryKind) -> Entry {
    Entry {
        name: "003-entry.json".to_owned(),
        raw: "{}".to_owned(),
        kind,
    }
}

/// A model turn of one text block.
fn turn(model_id: &str, text: &str) -> Entry {
    entry(EntryKind::Model {
        model_id: model_id.to_owned(),
        blocks: vec![Block::Text(text.to_owned())],
        usage: Usage::new(),
    })
}

/// One tool result, its way and its body.
fn tool(content: &str, is_error: bool) -> Entry {
    entry(EntryKind::ToolResult {
        tool_use_id: "tu-1".to_owned(),
        content: content.to_owned(),
        is_error,
    })
}

/// One settled frame of the whole window over `entries`.
fn glass(entries: Vec<Entry>) -> egui::FullOutput {
    let mut model = Model {
        transcript: Transcript { entries },
        ..seated()
    };
    let window = Window::sized(1400.0, 900.0);
    window.frame(Vec::new(), |ctx| crate::ui::render(ctx, &mut model));
    window.frame(Vec::new(), |ctx| crate::ui::render(ctx, &mut model))
}

/// The run reading exactly `header`, or a failure naming what is not there.
fn run(output: &egui::FullOutput, header: &str) -> crate::paint_probe::Seen {
    seen_of(output)
        .into_iter()
        .find(|seen| seen.text == header)
        .unwrap_or_else(|| panic!("nothing on the glass reads {header:?}"))
}

/// **The colours of the rules standing beside the block `header` heads**: the
/// [`theme::RULE`]-wide fills that span its line and stand just to its left,
/// which is where `theme::paint::ruled` puts one. The horizontal window is
/// what keeps another column's row rule, at the same height in another pane,
/// out of the answer.
fn beside(output: &egui::FullOutput, header: &str) -> Vec<egui::Color32> {
    let head = run(output, header);
    let (at, left) = (head.shown.center().y, head.laid.min.x);
    fills_of(output)
        .into_iter()
        .filter(|(rect, _)| {
            (rect.width() - theme::RULE).abs() < 0.5
                && rect.min.y <= at
                && at <= rect.max.y
                && rect.max.x <= left
                && left - rect.min.x < 40.0
        })
        .map(|(_, fill)| fill)
        .collect()
}

/// **The four weights, on the glass, in one transcript.** The operator's own
/// words wear the brand because the act is theirs, the model's wear full ink,
/// a peer's are weak and a sender the engine buried is fainter still — and
/// none of the four is a hue, which is what leaves colour meaning state.
#[test]
fn each_speaker_s_block_wears_its_own_weight() {
    let ended = entry(EntryKind::Delivered {
        sender: "child".to_owned(),
        sender_name: None,
        epitaph: Some("delivered".to_owned()),
        body: "landed".to_owned(),
    });
    let output = glass(vec![
        said("op", "port it"),
        said("user", "and again"),
        said("judge-one", "seconded"),
        ended,
        turn("model-a", "the seam is real"),
    ]);
    for (header, weight) in [
        ("op", theme::speaker(theme::Speaker::Operator)),
        ("user", theme::speaker(theme::Speaker::Operator)),
        ("judge-one", theme::speaker(theme::Speaker::Peer)),
        ("child (delivered)", theme::speaker(theme::Speaker::Ended)),
        ("model-a", theme::speaker(theme::Speaker::Model)),
    ] {
        assert!(
            beside(&output, header).contains(&weight),
            "{header:?} does not wear {weight:?}: {:?}",
            beside(&output, header)
        );
    }
    // And the operator's is the brand itself, which is what "their own act" is
    // worn in everywhere else on this glass.
    assert!(beside(&output, "op").contains(&theme::BRAND));
    // The model's is body ink and not an accent: a speaker is never a hue.
    assert!(beside(&output, "model-a").contains(&theme::INK));
}

/// **A call that failed is the one row painted in a STATE**, because *this
/// will not mend itself* is a state and the five other rows on the glass are
/// speakers. A failure the operator has to read the body to spot is one they
/// will miss.
#[test]
fn a_failed_tool_result_wears_the_error_accent_and_a_returned_one_does_not() {
    let red = theme::accent(theme::State::Error);
    let output = glass(vec![tool("no such file", true)]);
    assert_eq!(run(&output, "tu-1 failed").ink, red);
    assert!(beside(&output, "tu-1 failed").contains(&red));
    let returned = glass(vec![tool("done", false)]);
    assert_ne!(run(&returned, "tu-1 returned").ink, red);
    assert!(!beside(&returned, "tu-1 returned").contains(&red));
}

/// **A folded head is dimmed**, so *there is more of this* is read off the
/// weight of the text before anybody reads the control that says so.
#[test]
fn a_folded_head_stands_in_weak_ink() {
    let output = glass(vec![tool(&numbered(700), false)]);
    let head = seen_of(&output)
        .into_iter()
        .find(|seen| seen.text.starts_with("line 001"))
        .expect("the head of the answer is on the glass");
    assert_eq!(head.ink, theme::INK_WEAK, "{:?}", head.text);
}
