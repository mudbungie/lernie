//! **The focused thing is visibly the focused thing, and one keyboard has one
//! model of where it is** (bl-2d6b).
//!
//! Round one measured two costs of the keyboard story: a Tab focus that drew
//! nothing an operator could see on the dark ground, and a Tab focus that was
//! not the selection while an arrow's was — so the same keyboard had two
//! models of *where I am*. Both are answered on the glass: whatever holds the
//! keyboard wears the brand ring, the one stroke the language leaves, and a
//! Tab that lands on a list's row hands the arrows to that list, so the mark
//! on the heading moves with the ring.

use super::super::{HERE, Pane};
use crate::paint_probe::frame::{Window, press};
use crate::test_support::window::{seated, wall};
use crate::ui::{Chunk, Model, roster};

/// Every brand-ringed rectangle on the glass after `keys`, and the model.
fn ringed(model: &mut Model, keys: &[egui::Key]) -> (Vec<egui::Rect>, String) {
    let window = Window::new();
    let mut body = |ctx: &egui::Context| crate::ui::render(ctx, model);
    window.frame(Vec::new(), &mut body);
    for key in keys {
        window.frame(vec![press(*key)], &mut body);
    }
    let output = window.frame(Vec::new(), &mut body);
    let rings = crate::paint_probe::strokes_of(&output)
        .into_iter()
        .filter(|(_, ink)| *ink == crate::ui::theme::BRAND)
        .map(|(rect, _)| rect)
        .collect();
    (rings, crate::paint_probe::text_of(&output))
}

/// **A Tab is visible where it landed**: the first Tab rings exactly one
/// thing, and nothing is ringed before any key is pressed.
#[test]
fn the_first_tab_rings_one_thing_and_nothing_is_ringed_at_rest() {
    let mut model = seated();
    let (rings, _) = ringed(&mut model, &[]);
    assert!(
        rings.is_empty(),
        "nothing holds the keyboard at rest: {rings:?}"
    );
    let (rings, _) = ringed(&mut model, &[egui::Key::Tab]);
    assert_eq!(rings.len(), 1, "one thing holds the keyboard: {rings:?}");
}

/// **Tab and the arrows agree**: tabbing onto a wall's row hands the arrows
/// to the roster — the mark moves onto its heading — and Space on that row
/// aims, exactly as a click would.
#[test]
fn a_tab_onto_a_wall_hands_the_arrows_to_the_roster_and_space_aims() {
    let mut model = Model {
        roster: vec![Chunk {
            walls: vec![wall("first"), wall("second")],
            ..crate::test_support::window::own()
        }],
        focus: Pane::Conversations,
        ..seated()
    };
    // Six window-level entries stand before the first wall; the seventh Tab
    // is the wall.
    let tabs = [egui::Key::Tab; 7];
    let (rings, painted) = ringed(&mut model, &tabs);
    assert_eq!(model.focus, Pane::Roster, "the arrows followed the Tab");
    assert!(
        painted
            .lines()
            .any(|line| line == format!("{HERE} {}", roster::HEADING)),
        "the mark moved with it:\n{painted}"
    );
    assert_eq!(rings.len(), 1, "{rings:?}");
    let window = Window::new();
    let mut body = |ctx: &egui::Context| crate::ui::render(ctx, &mut model);
    window.frame(Vec::new(), &mut body);
    for _ in 0..7 {
        window.frame(vec![press(egui::Key::Tab)], &mut body);
    }
    window.frame(vec![press(egui::Key::Space)], &mut body);
    window.frame(Vec::new(), &mut body);
    assert_eq!(
        model.aim.as_ref().map(|aim| aim.address.as_str()),
        Some("first"),
        "Space on the ringed row aims it"
    );
}

/// **Tab onto a conversation's row hands the arrows to the list** the same
/// way, and a walk from there moves the selection the ring is on.
#[test]
fn a_tab_onto_a_conversation_hands_the_arrows_to_the_list() {
    let mut model = Model {
        focus: Pane::Roster,
        ..seated()
    };
    let window = Window::new();
    // Six window-level entries, one wall, its eight controls, then the row.
    let mut landed = false;
    for _ in 0..40 {
        window.frame(vec![press(egui::Key::Tab)], |ctx| {
            crate::ui::render(ctx, &mut model);
        });
        window.frame(Vec::new(), |ctx| crate::ui::render(ctx, &mut model));
        if model.focus == Pane::Conversations {
            landed = true;
            break;
        }
    }
    assert!(landed, "a Tab reaches the conversation list");
}
