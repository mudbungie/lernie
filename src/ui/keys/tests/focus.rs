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
    ringed_by(model, keys.iter().map(|key| vec![press(*key)]).collect())
}

/// The same, after one frame per entry of `frames`, each with its events.
fn ringed_by(model: &mut Model, frames: Vec<Vec<egui::Event>>) -> (Vec<egui::Rect>, String) {
    let window = Window::new();
    let mut body = |ctx: &egui::Context| crate::ui::render(ctx, model);
    window.frame(Vec::new(), &mut body);
    for events in frames {
        window.frame(events, &mut body);
    }
    let output = window.frame(Vec::new(), &mut body);
    let rings = crate::paint_probe::strokes_of(&output)
        .into_iter()
        .filter(|(_, ink)| *ink == crate::ui::theme::BRAND)
        .map(|(rect, _)| rect)
        .collect();
    (rings, crate::paint_probe::text_of(&output))
}

/// **At rest the one thing ringed is the prompt line**, because the caret
/// lives there (bl-b1a1), and a Tab takes the keyboard on from it — onto
/// what is laid after the box, or round to the bar's first control where
/// nothing is (the seated window lays no strip and no fold after it), which
/// egui gives on the frame after — and the box does not take it back.
#[test]
fn the_box_is_ringed_at_rest_and_a_tab_moves_on_from_it() {
    let mut model = seated();
    let (rings, _) = ringed(&mut model, &[]);
    assert_eq!(rings.len(), 1, "the box holds the caret at rest: {rings:?}");
    let window = Window::new();
    let box_id = egui::Id::new(crate::ui::keys::BOX_ID);
    for key in [None, Some(egui::Key::Tab), None] {
        let events = key.map(|key| vec![press(key)]).unwrap_or_default();
        window.frame(events, |ctx| crate::ui::render(ctx, &mut model));
    }
    assert!(
        window.focused().is_some_and(|id| id != box_id),
        "the Tab took the keyboard on, and the box did not take it back"
    );
}

/// **The caret comes back to the box** after every gesture that took it
/// away — the switcher choosing, Esc leaving the box, an overlay closing —
/// and stays off it while something stands over the prompt (bl-b1a1).
#[test]
fn the_caret_comes_back_to_the_box() {
    let window = Window::new();
    let id = egui::Id::new(crate::ui::keys::BOX_ID);
    let mut model = seated();
    let frame = |model: &mut Model, events: Vec<egui::Event>| {
        window.frame(events, |ctx| crate::ui::render(ctx, model));
    };
    frame(&mut model, Vec::new());
    assert_eq!(window.focused(), Some(id), "at launch");
    let unaimed = Window::new();
    unaimed.frame(Vec::new(), |ctx| {
        crate::ui::render(ctx, &mut Model::default());
    });
    assert_eq!(
        unaimed.focused(),
        Some(id),
        "with nothing aimed, the box still takes it"
    );
    model.notice = Some(crate::ui::Notice::Refused("no".to_owned()));
    frame(&mut model, vec![press(egui::Key::Escape)]);
    assert_eq!(model.notice, None, "Esc in the box is the notice's dismiss");
    frame(&mut model, Vec::new());
    assert_eq!(window.focused(), Some(id), "after Esc left the box");
    model.begin_trail();
    frame(&mut model, Vec::new());
    frame(&mut model, Vec::new());
    assert_ne!(window.focused(), Some(id), "an overlay unpaints the box");
    frame(&mut model, vec![press(egui::Key::Escape)]);
    frame(&mut model, Vec::new());
    assert!(!model.covered());
    assert_eq!(window.focused(), Some(id), "after the overlay closed");
    "/re".clone_into(&mut model.draft);
    model.notice = Some(crate::ui::Notice::Refused("no".to_owned()));
    frame(&mut model, vec![press(egui::Key::Escape)]);
    frame(&mut model, Vec::new());
    assert_eq!(model.draft, "", "the menu took that Esc");
    assert!(
        model.notice.is_some(),
        "and only the menu: the notice is next"
    );
    assert_eq!(window.focused(), Some(id), "after Esc cleared the menu");
}

/// **The bar's controls come first in the Tab order** (DESIGN §4.44): it is
/// painted before every other panel — its foot, its pin, its `+ new` and the
/// one conversation's row — so a walk from nothing held crosses them first.
const BAR: usize = 4;

/// A window on one engine holding two walls, which is the shape both beats
/// below Tab their way down.
fn two_walls() -> Model {
    Model {
        switching: true,
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
/// The engine's own row stands first and the first wall second; the `+`
/// that stood beside the engine is `+ new` under each wall now (§4.42 item
/// 7), and the strips that stood ahead of them are rows of the menu.
#[test]
fn a_tab_reaches_a_wall_s_row_and_space_on_it_aims() {
    let mut model = two_walls();
    let (rings, _) = ringed(&mut model, &[egui::Key::Tab; BAR + 2]);
    assert_eq!(rings.len(), 1, "{rings:?}");
    let window = Window::new();
    let mut body = |ctx: &egui::Context| crate::ui::render(ctx, &mut model);
    window.frame(Vec::new(), &mut body);
    for _ in 0..BAR + 2 {
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

/// **`+ new` is a control like any other, so a key reaches it** (F1): the
/// third Tab is the one under the first wall, and Space on it begins a
/// conversation there — the wall aimed, the selection dropped and the caret
/// in the box.
///
/// **And the binding that opens a standing engine does NOT also fire**
/// (`super::super::opening`): something holds the keyboard, so egui fires that
/// control and nothing here stands in for a click egui can already make.
#[test]
fn a_tab_reaches_new_and_space_on_it_begins_a_conversation() {
    let mut model = two_walls();
    model.standing = Some(crate::ui::Stand::Engine(
        "a name no engine here has".to_owned(),
    ));
    let window = Window::new();
    let mut body = |ctx: &egui::Context| crate::ui::render(ctx, &mut model);
    window.frame(Vec::new(), &mut body);
    for _ in 0..BAR + 3 {
        window.frame(vec![press(egui::Key::Tab)], &mut body);
    }
    window.frame(vec![press(egui::Key::Space)], &mut body);
    window.frame(Vec::new(), &mut body);
    assert_eq!(model.conversation, None, "the selection is cleared");
    assert_eq!(
        model.aim.as_ref().map(|aim| aim.address.as_str()),
        Some("first"),
        "on the wall it stood under"
    );
    assert_eq!(
        window.focused(),
        Some(egui::Id::new(crate::ui::keys::BOX_ID)),
        "and the caret is in the box that begins one"
    );
}

/// One Shift+Tab: the key that walks the Tab order backward.
fn back() -> egui::Event {
    egui::Event::Key {
        key: egui::Key::Tab,
        physical_key: None,
        pressed: true,
        repeat: false,
        modifiers: egui::Modifiers::SHIFT,
    }
}

/// **A Shift+Tab off the box lands on the rule, ringed** (bl-8e3e), and
/// Enter or Space there opens its dropup. The Tab order is the laying order,
/// and the rule is laid before the box since it stands under it (DESIGN
/// §4.45 item 3; `crate::ui::shell`): so the walk back from the box is the
/// rule read right to left — the name, then priority, which is the second
/// stop — and the walk on from it is the transcript.
#[test]
fn a_shift_tab_lands_on_a_ringed_selector_that_enter_and_space_open() {
    for key in [egui::Key::Enter, egui::Key::Space] {
        let mut model = crate::test_support::window::selecting();
        let (rings, _) = ringed_by(&mut model, vec![vec![back()], vec![back()]]);
        assert_eq!(
            rings.len(),
            1,
            "one ring, on what took the keyboard: {rings:?}"
        );
        let window = Window::new();
        let mut body = |ctx: &egui::Context| crate::ui::render(ctx, &mut model);
        for events in [Vec::new(), vec![back()], vec![back()], vec![press(key)]] {
            window.frame(events, &mut body);
        }
        assert_eq!(
            model.dropup.map(|open| open.selector),
            Some(crate::ui::Selector::Priority),
            "{key:?} on the focused selector opens it"
        );
    }
}
