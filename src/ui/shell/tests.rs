//! The whole window in one frame: that every pane is on it, that a notice
//! stands where content would have been, and that it can be put down.
//!
//! The width policy's own arithmetic is `super::policy`'s suite; what is
//! asserted here is the layout it produces, and [`narrow`] holds the shape the
//! window takes when it can no longer produce three columns.

use super::{render, widths};
use crate::paint_probe::frame::{Window, press};
use crate::test_support::window::{conv, own, painted, seated, seen, wall};
use crate::ui::{Chunk, Model};

/// The narrow shape: one column at a time, and the bar that names the three.
mod narrow;
/// The notice bar: where it stands, whose ink it wears, and how it is put
/// down.
mod notice;
/// What a list pane's width is on the glass, and the band that used to stand
/// beside it.
mod width;

/// **One frame paints every pane.** The smoke test the whole ball is about: a
/// window that opens and shows the roster, the list, the conversation and the
/// composer, all from a snapshot and with nothing dialled.
#[test]
fn one_frame_paints_the_roster_the_list_the_conversation_and_the_composer() {
    let mut model = seated();
    let shown = painted(&mut model);
    for expected in [
        "channels",
        "(this box's own engine)",
        "home  (named)  2 conversations",
        "conversations",
        "port the paint probe  [quiescent]  42s",
        "conversation",
        "op",
        "port it",
        crate::ui::composer::SEND,
    ] {
        assert!(
            shown.contains(expected),
            "{expected:?} is not on the glass:\n{shown}"
        );
    }
}

/// **An empty window is not a blank one: every pane says what it is waiting
/// for** — and the window under test is seeded the way `src/main.rs` seeds one,
/// off a data root holding nothing at all (bl-08b6). That is the first run of a
/// seat on a new box, and the channels pane is the whole of what it has.
#[test]
fn an_empty_window_says_what_each_pane_is_waiting_for() {
    let scratch = crate::test_support::Scratch::new();
    let mut model = Model {
        roster: crate::seat::channels(scratch.path()),
        ..Model::default()
    };
    let shown = painted(&mut model);
    assert!(
        shown.contains("nothing provisioned at"),
        "the channels pane says what it holds and why it is empty:\n{shown}"
    );
    assert!(
        shown.contains("the seat mints nothing"),
        "and names the act that fills it:\n{shown}"
    );
    // **And every one names the next act, in weak ink** (bl-f251, item 5):
    // the window's three empty panes each say what to do rather than what is
    // missing, and say it one step under content.
    let window = crate::paint_probe::frame::Window::new();
    let runs = crate::test_support::window::seen(&window, |ctx| crate::ui::render(ctx, &mut model));
    for empty in [crate::ui::convs::NO_WALL, crate::ui::chat::NO_CONVERSATION] {
        let run = runs
            .iter()
            .find(|run| run.text == empty)
            .unwrap_or_else(|| panic!("{empty:?} is on the glass whole"));
        assert_eq!(run.ink, crate::ui::theme::INK_WEAK, "{empty:?}");
    }
    for expected in [
        crate::ui::convs::NO_WALL,
        crate::ui::chat::NO_CONVERSATION,
        crate::ui::composer::NOWHERE,
    ] {
        assert!(
            shown.contains(expected),
            "{expected:?} is not on the glass:\n{shown}"
        );
    }
}

/// The policy on the glass: at 900 points the conversation used to be a
/// ~140-point strip while the roster kept 280. Now the panes yield, and a
/// message in the chat pane starts where the floor says it does.
#[test]
fn a_narrow_window_paints_the_conversation_at_its_floor() {
    let mut model = seated();
    let window = Window::sized(900.0, 600.0);
    let said = seen(&window, |ctx| render(ctx, &mut model))
        .into_iter()
        .find(|run| run.text == "port it")
        .expect("the conversation is on the glass");
    let (roster, convs) = widths(900.0);
    assert!(
        said.laid.min.x < roster + convs + 40.0,
        "the chat pane begins where the two list panes end: {:?}",
        said.laid
    );
}

/// **A list longer than its pane scrolls, and the keyboard walk brings its own
/// row along** (bl-e5d2). The overflow used to be cut at the panel edge
/// mid-glyph, with nothing saying it had been cut — while the arrow walk moved
/// the selection onto rows the glass had never painted, which is exactly the
/// disagreement `crate::ui::roster::aimable` exists to prevent.
#[test]
fn the_roster_scrolls_and_a_walk_to_the_last_wall_puts_it_on_the_glass() {
    let mut model = Model {
        roster: vec![Chunk {
            walls: (0..24).map(|i| wall(&format!("wall-{i:02}"))).collect(),
            ..own()
        }],
        ..Model::default()
    };
    let window = Window::sized(900.0, 260.0);
    // **A roster row is elided at the column's width** (`theme::paint::row`),
    // so the row reads back on the glass as its head and a `…`. The assertion
    // is on what was SHOWN and so it is aimed at the head — reading the whole
    // line back would be reading the string that went in, which is the one
    // thing `crate::paint_probe` exists to refuse.
    let last = "wall-23";
    let first = seen(&window, |ctx| render(ctx, &mut model));
    assert!(
        !first.iter().any(|run| run.text.starts_with(last)),
        "the fold is real, or this test proves nothing"
    );
    for _ in 0..24 {
        window.frame(vec![press(egui::Key::ArrowDown)], |ctx| {
            render(ctx, &mut model);
        });
    }
    assert_eq!(
        model.aim.clone().map(|aim| aim.address),
        Some("wall-23".to_owned())
    );
    // **One settle frame**, for `crate::snapshot`'s own reason: egui applies a
    // scroll over the frame after the one that asked for it, so a single pass
    // reads a window mid-layout. The keypress frame is where `scroll_to_me` is
    // spent; this is where it has landed.
    window.frame(Vec::new(), |ctx| render(ctx, &mut model));
    let shown = seen(&window, |ctx| render(ctx, &mut model));
    assert!(
        shown.iter().any(|run| run.text.starts_with(last)),
        "the walked-to row is on the glass: {:?}",
        shown.iter().map(|run| &run.text).collect::<Vec<&String>>()
    );
}

/// The conversation list, on the same rule and through the same walk.
#[test]
fn the_conversation_list_scrolls_and_a_walk_puts_its_row_on_the_glass() {
    let mut model = Model {
        convs: (0..24)
            .map(|i| conv(&format!("id-{i:02}"), &format!("conv-{i:02}")))
            .collect(),
        focus: crate::ui::keys::Pane::Conversations,
        ..seated()
    };
    let window = Window::sized(900.0, 260.0);
    let last = crate::ui::convs::headline(&conv("id-23", "conv-23"));
    assert!(
        !seen(&window, |ctx| render(ctx, &mut model))
            .iter()
            .any(|run| run.text == last),
        "the fold is real, or this test proves nothing"
    );
    for _ in 0..24 {
        window.frame(vec![press(egui::Key::ArrowDown)], |ctx| {
            render(ctx, &mut model);
        });
    }
    // One settle frame, for the roster walk's reason above: the scroll lands
    // on the frame after the one that asked for it.
    window.frame(Vec::new(), |ctx| render(ctx, &mut model));
    assert!(
        seen(&window, |ctx| render(ctx, &mut model))
            .iter()
            .any(|run| run.text == last),
        "the walked-to row is on the glass"
    );
}
