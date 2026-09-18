//! **The one cursor track** (DESIGN §4.39): where each arrow lands, and what
//! landing there means — an engine's row stood on, a wall aimed, a
//! conversation selected — plus the sideways keys, which name a column in the
//! narrow shape and nothing at all in the broad one.
//!
//! Split from [`super`] at the 300-line cap on the seam the module has: that
//! file is what stands the keys DOWN and the one binding that is not a walk,
//! and this is the walk.

use super::{stocked, typed};
use crate::test_support::window::{painted, seated};
use crate::ui::Model;

/// **Left and right name nothing in the broad shape** (DESIGN §4.39). They
/// used to hand the arrows to one of two list panes; there is one list, so
/// there is no place for the key to name and it names none — the cursor is
/// exactly where it was, and the walk is the only thing that moves it.
#[test]
fn sideways_does_nothing_while_both_columns_are_on_the_glass() {
    let mut model = stocked();
    let was = model.column;
    typed(
        &mut model,
        &[
            egui::Key::ArrowLeft,
            egui::Key::ArrowRight,
            egui::Key::ArrowLeft,
        ],
    );
    assert_eq!(model.column, was, "the column is the narrow shape's own");
    assert_eq!(
        model.conversation.as_deref(),
        seated().conversation.as_deref(),
        "and nothing was selected by a key that names no place"
    );
}

/// **The roster's walk crosses two kinds of row** (DESIGN §4.39): landing on
/// a wall aims it, exactly as a click does, and landing on an engine's row
/// only STANDS there — a walk that opened every engine it moved through would
/// be a walk nobody could use to reach the one below.
#[test]
fn the_arrows_walk_the_engine_rows_and_the_open_engine_s_walls() {
    let mut model = Model {
        aim: None,
        ..stocked()
    };
    typed(&mut model, &[egui::Key::ArrowDown]);
    assert_eq!(
        model.standing.as_deref(),
        Some("(this box's own engine)"),
        "a list nobody has entered opens at its first row, which is an engine"
    );
    assert_eq!(model.aim, None, "and standing there selects nothing");
    typed(&mut model, &[egui::Key::ArrowDown]);
    assert_eq!(
        model.aim.as_ref().map(|aim| aim.address.clone()),
        Some("home".to_owned()),
        "the open engine's wall is the next stop, and landing on it aims it"
    );
    assert_eq!(model.standing, None, "an aim is where the cursor is");
    typed(&mut model, &[egui::Key::ArrowDown]);
    assert_eq!(
        model.standing.as_deref(),
        Some("elsewhere"),
        "and the closed engine's own row is next, never its walls"
    );
    typed(&mut model, &[egui::Key::ArrowDown]);
    assert_eq!(
        model.standing.as_deref(),
        Some("elsewhere"),
        "the end saturates rather than wrapping"
    );
    typed(&mut model, &[egui::Key::ArrowUp]);
    assert_eq!(
        model.aim.as_ref().map(|aim| aim.address.clone()),
        Some("home".to_owned())
    );
}

/// **Enter opens the engine the walk is standing on**, through the same door
/// the row's own click calls — which is what keeps the binding from being a
/// second surface. It is a second keypress and not the walk's own act, because
/// the walk has to be able to pass a closed engine without opening it.
#[test]
fn enter_on_the_row_the_walk_stands_on_opens_that_engine() {
    let mut model = Model {
        aim: None,
        ..stocked()
    };
    typed(
        &mut model,
        &[
            egui::Key::ArrowDown,
            egui::Key::ArrowDown,
            egui::Key::ArrowDown,
            egui::Key::Enter,
        ],
    );
    assert_eq!(model.engines.open.as_deref(), Some("elsewhere"));
    assert_eq!(
        model.aim.as_ref().map(|aim| aim.address.clone()),
        Some("elsewhere".to_owned()),
        "opening aims the engine's first addressable wall"
    );
}

/// **The conversations are stops on the SAME track** (DESIGN §4.39): the walk
/// goes engine row, wall, that wall's conversations, next engine row — one
/// list in paint order — and landing on a conversation selects it, which is
/// the selection every read follows.
#[test]
fn the_walk_runs_on_through_the_aimed_wall_s_conversations() {
    let mut model = Model {
        conversation: None,
        ..stocked()
    };
    // The cursor is on the aimed wall, so one step down is its first
    // conversation.
    typed(&mut model, &[egui::Key::ArrowDown]);
    assert_eq!(model.conversation.as_deref(), Some("a"));
    typed(&mut model, &[egui::Key::ArrowDown]);
    assert_eq!(model.conversation.as_deref(), Some("b"));
    // Past the last conversation the track carries on into the next engine's
    // own row, because it is one list and not two.
    typed(&mut model, &[egui::Key::ArrowDown]);
    assert_eq!(model.standing.as_deref(), Some("elsewhere"));
    assert_eq!(
        model.conversation.as_deref(),
        Some("b"),
        "standing on an engine selects nothing"
    );
    typed(&mut model, &[egui::Key::ArrowDown]);
    assert_eq!(
        model.standing.as_deref(),
        Some("elsewhere"),
        "the end saturates rather than wrapping"
    );
}

/// **A row the pointer can aim at is a row a key can aim at.** The pending row
/// a start puts in the list is in the one list both walk, so an arrow leaves it
/// for the first conversation the engine actually answered — which is also how
/// an operator escapes a claim whose driver never wrote its branch.
#[test]
fn the_arrows_walk_the_started_conversation_s_row_like_any_other() {
    let mut model = Model {
        conversation: Some("brisk-otter".to_owned()),
        start: Some(crate::ui::model::Start {
            address: "home".to_owned(),
            goal: "port it".to_owned(),
            phase: crate::ui::model::Phase::Started("brisk-otter".to_owned()),
            spread: None,
        }),
        ..stocked()
    };
    let shown = painted(&mut model);
    assert!(shown.contains("brisk-otter"), "{shown}");
    typed(&mut model, &[egui::Key::ArrowDown]);
    assert_eq!(
        model.conversation.as_deref(),
        Some("a"),
        "one step down out of the pending row and into the list"
    );
}
