//! The whole window in one frame: that every pane is on it, that a notice
//! stands where content would have been, and that it can be put down.

use super::{CHAT_FLOOR, DISMISS, SIDE_FLOOR, render, widths};
use crate::paint_probe::frame::{Window, press};
use crate::test_support::window::{click, conv, own, painted, seated, seen, wall};
use crate::ui::{Chunk, Model, Notice};

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

/// **A refusal and an unreadable answer are both visible, and they read
/// differently** — one is the engine's sentence, the other is a statement about
/// this seat, and only the second is fixed by an upgrade. Neither is a silent
/// drop, which is the reply vocabulary's own policy on the glass.
#[test]
fn a_notice_stands_where_the_content_would_have_been_and_says_whose_it_is() {
    for (notice, expected) in [
        (
            Notice::Refused("unknown workspace \"hoem\"".to_owned()),
            "the engine refused: unknown workspace \"hoem\"",
        ),
        (
            Notice::Unreadable("cannot paint a \"board\" answer".to_owned()),
            "this seat could not read the answer: cannot paint a \"board\" answer",
        ),
    ] {
        let mut model = Model {
            notice: Some(notice),
            ..seated()
        };
        let shown = painted(&mut model);
        assert!(shown.contains(expected), "{expected:?}:\n{shown}");
        assert!(
            shown.contains("home  (named)  2 conversations"),
            "a refusal about one pane does not stop the others:\n{shown}"
        );
    }
}

/// **It is a bar, not a modal**, and it can be put down: an operator who has
/// read a refusal should not have to wait for the next answer to clear it.
#[test]
fn a_notice_can_be_put_down() {
    let mut model = Model {
        notice: Some(Notice::Refused("no".to_owned())),
        ..seated()
    };
    let window = Window::new();
    click(&window, DISMISS, |ctx| render(ctx, &mut model));
    assert_eq!(model.notice, None);
}

/// **The policy the window had none of** (bl-e5d2): the conversation has a
/// floor and the two list panes yield to it, together and in proportion, until
/// they reach their own floor — where nothing yields, because two panes showing
/// nothing buys the chat pane a width it still cannot use.
#[test]
fn the_list_panes_yield_to_the_conversation_s_floor_and_then_stop() {
    assert_eq!(widths(1200.0), (280.0, 320.0), "wide enough for both");
    assert_eq!(
        widths(1020.0),
        (280.0, 320.0),
        "exactly enough is still enough"
    );
    let (roster, convs) = widths(900.0);
    assert!(
        (roster - 224.0).abs() < 0.5 && (convs - 256.0).abs() < 0.5,
        "the loss is shared in proportion: {roster}, {convs}"
    );
    assert!(
        900.0 - roster - convs >= CHAT_FLOOR,
        "the conversation kept its floor"
    );
    assert_eq!(
        widths(400.0),
        (SIDE_FLOOR, SIDE_FLOOR),
        "past their own floor the list panes stop yielding"
    );
}

/// The same, on the glass: at 900 points the conversation used to be a
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
    let last = crate::ui::roster::line(&wall("wall-23"));
    let first = seen(&window, |ctx| render(ctx, &mut model));
    assert!(
        !first.iter().any(|run| run.text == last),
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
    let shown = seen(&window, |ctx| render(ctx, &mut model));
    assert!(
        shown.iter().any(|run| run.text == last),
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
    assert!(
        seen(&window, |ctx| render(ctx, &mut model))
            .iter()
            .any(|run| run.text == last),
        "the walked-to row is on the glass"
    );
}

/// **The notice wraps rather than being cut at the frame** (bl-3d0f). A
/// horizontal layout lays its label on one line however long it is, and the
/// panel cut it at the window's right edge with no ellipsis to say so — and
/// every refusal this seat paints puts the fact first and the remedy last, so
/// the half that was lost was always the half that says what to do.
///
/// The subject is the first run of a seat on an unprovisioned box: the notice
/// is the only thing on the window carrying an instruction.
#[test]
fn a_long_refusal_wraps_and_its_remedy_reaches_the_glass() {
    let said = format!(
        "no wire provisioned at /home/u/.local/share/lernie/wire: {}",
        crate::channel::material::REMEDY
    );
    let mut model = Model {
        notice: Some(Notice::Unreachable(said.clone())),
        ..seated()
    };
    let window = Window::sized(900.0, 600.0);
    window.text(|ctx| render(ctx, &mut model));
    let bar = seen(&window, |ctx| render(ctx, &mut model))
        .into_iter()
        .find(|run| run.text.starts_with("this seat could not reach it"))
        .expect("the bar is on the glass");
    // **The rects are what testify here, not the glyphs.** A galley's rows
    // carry no newline where the WRAP broke them, so a wrapped run and a run
    // laid past the frame read back as the same string — which is the paint
    // probe's own division of labour: geometry is unaffected, it is the text
    // that lies.
    assert!(
        bar.laid.width() <= 900.0,
        "the run was laid inside the window rather than past it: {:?}",
        bar.laid
    );
    assert!(
        bar.shown.width() >= bar.laid.width() - 0.5,
        "and nothing was clipped off its end: laid {:?}, shown {:?}",
        bar.laid,
        bar.shown
    );
    assert!(
        bar.text.ends_with("the seat mints nothing"),
        "so the remedy's last words are on the glass: {:?}",
        bar.text
    );
}
