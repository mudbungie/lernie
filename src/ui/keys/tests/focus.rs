//! **The focused thing is visibly the focused thing** (bl-2d6b), and every
//! control a pointer reaches a Tab reaches too (yog's `docs/QUALITY.md` F1).
//!
//! Round one measured two costs of the keyboard story: a Tab focus that drew
//! nothing an operator could see on the dark ground, and a Tab focus that was
//! not the selection while an arrow's was. The first is answered on the glass
//! — whatever holds the keyboard wears the brand ring, the one stroke the
//! language leaves. **The second dissolved with the second list** (DESIGN
//! §4.39): there was a `Pane` field saying which of two lists the arrows
//! belonged to, and a mark on that list's heading; with one list there is one
//! answer, so there is nothing for a Tab to hand over and nothing for a mark
//! to say.

use crate::paint_probe::frame::{Window, press};
use crate::test_support::window::{seated, wall};
use crate::ui::{Chunk, Model};

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

/// A window on one engine holding two walls, which is the shape both beats
/// below Tab their way down.
fn two_walls() -> Model {
    Model {
        roster: vec![Chunk {
            walls: vec![wall("first"), wall("second")],
            ..crate::test_support::window::own()
        }],
        ..seated()
    }
}

/// **The Tab order runs the way the row reads**, and Space on a ringed row
/// fires exactly what a click fires.
///
/// Six window-level entries stand first, then the engine's own row, then the
/// `+` beside it (DESIGN §4.39 — the row is allocated before the control, so
/// the keyboard meets the name before the act on it), so the ninth Tab is the
/// first wall.
#[test]
fn a_tab_reaches_a_wall_s_row_and_space_on_it_aims() {
    let mut model = two_walls();
    let (rings, _) = ringed(&mut model, &[egui::Key::Tab; 9]);
    assert_eq!(rings.len(), 1, "{rings:?}");
    let window = Window::new();
    let mut body = |ctx: &egui::Context| crate::ui::render(ctx, &mut model);
    window.frame(Vec::new(), &mut body);
    for _ in 0..9 {
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

/// **The `+` is a control like any other, so a key reaches it** (F1): the
/// eighth Tab is the one beside the engine's row, and Space on it begins a
/// conversation — the selection dropped and the caret in the box.
///
/// **And the binding that opens a standing engine does NOT also fire**
/// (`super::super::opening`): something holds the keyboard, so egui fires that
/// control and nothing here stands in for a click egui can already make.
#[test]
fn a_tab_reaches_the_plus_and_space_on_it_begins_a_conversation() {
    let mut model = two_walls();
    model.standing = Some("a name no engine here has".to_owned());
    let window = Window::new();
    let mut body = |ctx: &egui::Context| crate::ui::render(ctx, &mut model);
    window.frame(Vec::new(), &mut body);
    for _ in 0..8 {
        window.frame(vec![press(egui::Key::Tab)], &mut body);
    }
    window.frame(vec![press(egui::Key::Space)], &mut body);
    window.frame(Vec::new(), &mut body);
    assert_eq!(model.conversation, None, "the selection is cleared");
    assert_eq!(
        window.focused(),
        Some(egui::Id::new(crate::ui::keys::BOX_ID)),
        "and the caret is in the box that begins one"
    );
}
