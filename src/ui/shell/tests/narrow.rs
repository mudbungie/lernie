//! **The narrow shape** (bl-dfda): one column at a time, and the bar that names
//! the two.
//!
//! The defect these are written against is a picture — at 400x800 the columns
//! stood side by side, each about 120 points wide, with every line in every one
//! of them wrapped to two or three words. The assertions are about the layout
//! rather than the pixels: what is on the glass, what is not, and what one
//! gesture on the bar changes.

use super::super::{Column, render};
use crate::paint_probe::frame::{Window, press};
use crate::test_support::window::{click, seated, wall};
use crate::ui::{Enrolling, Model, chat, composer, roster};

/// A phone-shaped window: the size the ball named, and narrower than the width
/// at which the yield can still leave the conversation its floor.
fn phone() -> Window {
    Window::sized(400.0, 800.0)
}

/// **One column at a time, and the bar names both.** The roster's rows are
/// not on the glass beside the conversation, which is the whole of the change:
/// they are one gesture away instead of 120 points wide.
#[test]
fn the_narrow_window_paints_one_column_and_a_bar_naming_both() {
    let mut model = seated();
    let shown = phone().text(|ctx| render(ctx, &mut model));
    for word in [roster::HEADING, chat::HEADING] {
        assert!(shown.contains(word), "the bar names {word:?}:\n{shown}");
    }
    assert!(
        shown.contains("port it"),
        "the conversation column is the one on the glass:\n{shown}"
    );
    assert!(
        !shown.contains(&roster::line(&wall("home"))),
        "and the roster is not beside it:\n{shown}"
    );
}

/// **The bar is the navigation**: one gesture from any column to any other.
#[test]
fn a_gesture_on_the_bar_brings_that_column_to_the_glass() {
    let mut model = seated();
    let window = phone();
    click(&window, roster::HEADING, |ctx| render(ctx, &mut model));
    assert_eq!(model.column, Column::Engines);
    let shown = window.text(|ctx| render(ctx, &mut model));
    assert!(
        shown.contains(&roster::line(&wall("home"))),
        "the engines column is on the glass:\n{shown}"
    );
    assert!(
        !shown.contains("port it"),
        "and the conversation is not:\n{shown}"
    );
}

/// **The composer stands down off the conversation's own column**, which is the
/// rule every covering pane already keeps read literally: the conversation it
/// deposits into is on the glass only when its column is.
#[test]
fn the_composer_stands_with_its_conversation_and_nowhere_else() {
    let mut model = seated();
    let window = phone();
    assert!(
        window
            .text(|ctx| render(ctx, &mut model))
            .contains(composer::SEND),
        "the box is under the conversation"
    );
    model.column = Column::Engines;
    assert!(
        !window
            .text(|ctx| render(ctx, &mut model))
            .contains(composer::SEND),
        "and not under the list"
    );
}

/// **A covering pane stands the bar down too.** A pane that covers the window
/// is a modal, and a navigation control that changed a column nobody can see
/// would answer a click with nothing — the way out is the pane's own control,
/// which where the material is a secret is the one that forgets.
#[test]
fn a_covering_pane_takes_the_bar_with_the_columns_and_its_close_brings_it_back() {
    let aim = seated()
        .aim
        .unwrap_or_else(|| panic!("the fixture is aimed"));
    let mut model = Model {
        enroll: Some(Enrolling::at(aim)),
        ..seated()
    };
    let window = phone();
    let covered = window.text(|ctx| render(ctx, &mut model));
    assert!(
        covered.contains(crate::ui::enroll::HEADING),
        "the pane covers the window:\n{covered}"
    );
    assert!(
        !covered.contains(roster::HEADING),
        "and the bar is not standing under it:\n{covered}"
    );
    click(&window, crate::ui::enroll::CLOSE, |ctx| {
        render(ctx, &mut model);
    });
    let back = window.text(|ctx| render(ctx, &mut model));
    assert!(back.contains(roster::HEADING), "the bar is back:\n{back}");
}

/// **Left and right name a place, and in this shape the place is a column** —
/// the one shape where they name anything at all, since the broad window has
/// both columns on the glass already (DESIGN §4.39).
#[test]
fn the_side_keys_step_the_column_and_stop_at_the_ends() {
    let mut model = seated();
    let window = phone();
    for (key, column) in [
        (egui::Key::ArrowLeft, Column::Engines),
        (egui::Key::ArrowLeft, Column::Engines),
        (egui::Key::ArrowRight, Column::Conversation),
        (egui::Key::ArrowRight, Column::Conversation),
    ] {
        window.frame(vec![press(key)], |ctx| render(ctx, &mut model));
        assert_eq!(model.column, column, "after {key:?}");
    }
}

/// **The walk is the same one list in either shape** (DESIGN §4.39): the
/// narrow window paints the engines column, and a step down it moves the one
/// cursor — here off the aimed wall and onto its first conversation, which is
/// what the conversation column behind it then shows.
#[test]
fn an_arrow_walks_the_one_list_in_the_narrow_shape_too() {
    let mut model = Model {
        column: Column::Engines,
        conversation: None,
        ..seated()
    };
    phone().frame(vec![press(egui::Key::ArrowDown)], |ctx| {
        render(ctx, &mut model);
    });
    assert_eq!(
        model.conversation,
        Some("20260830T051200Z-a1b2".to_owned()),
        "the walk landed on the aimed wall's first conversation"
    );
}
