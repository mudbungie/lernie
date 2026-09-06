//! **What hangs under a conversation, on the glass** (bl-00f5).
//!
//! `crate::ui::model::subtree`'s suite asserts the list; this asserts what the
//! operator sees and presses. The two are different questions — the fold could
//! be computed perfectly and every row painted anyway — and only this one can
//! say the machinery is off the strip.

use super::render;
use crate::paint_probe::frame::Window;
use crate::reply::convs::ConvRow;
use crate::test_support::window::{click, conv, pane, seated};
use crate::ui::Model;

/// The compactors' shared preview, verbatim in the shape the ball measured.
const MACHINERY: &str = "You are the compactor for branch `20260906T034041Z`.";

/// A root with four compactors under it, and one conversation beside them.
fn session() -> Model {
    let mut rows: Vec<ConvRow> = Vec::new();
    let mut root = conv("c-root", "ShorelineGuppy");
    root.members = 5;
    rows.push(root);
    for name in [
        "CardboardFoothill",
        "CrispGorge",
        "DuneBreeze",
        "TortillaSaucepan",
    ] {
        let mut child = conv(name, name);
        child.depth = 1;
        child.preview = MACHINERY.to_owned();
        rows.push(child);
    }
    rows.push(conv("c-other", "WhiskFrost"));
    Model {
        convs: rows,
        ..seated()
    }
}

/// The word the control wears while the subtree is folded.
fn opener() -> String {
    format!("{} 4", super::super::SHOW)
}

#[test]
fn a_conversation_s_machinery_is_off_the_strip_and_one_gesture_away() {
    let mut model = session();
    let painted = pane(|ui| render(ui, &mut model));
    assert!(painted.contains("ShorelineGuppy"), "{painted}");
    assert!(painted.contains("WhiskFrost"), "{painted}");
    assert!(!painted.contains("CardboardFoothill"), "{painted}");
    assert!(!painted.contains(MACHINERY), "{painted}");
    assert!(painted.contains(&opener()), "{painted}");
}

#[test]
fn the_control_brings_the_subtree_and_the_other_one_puts_it_back() {
    let mut model = session();
    let window = Window::new();
    click(&window, &opener(), |ctx| {
        egui::CentralPanel::default().show(ctx, |ui| render(ui, &mut model));
    });
    let opened = pane(|ui| render(ui, &mut model));
    assert!(opened.contains("CardboardFoothill"), "{opened}");
    assert!(opened.contains("TortillaSaucepan"), "{opened}");
    let closer = format!("{} 4", super::super::HIDE);
    assert!(opened.contains(&closer), "{opened}");

    click(&window, &closer, |ctx| {
        egui::CentralPanel::default().show(ctx, |ui| render(ui, &mut model));
    });
    let back = pane(|ui| render(ui, &mut model));
    assert!(!back.contains("CardboardFoothill"), "{back}");
}

/// **A conversation with nothing under it gets no control**, which is the
/// fold's own rule read from the other side: an expander over nothing costs a
/// gesture and hides no row.
#[test]
fn a_conversation_with_nothing_under_it_carries_no_control() {
    let mut alone = seated();
    let painted = pane(|ui| render(ui, &mut alone));
    assert!(!painted.contains(super::super::SHOW), "{painted}");
}
