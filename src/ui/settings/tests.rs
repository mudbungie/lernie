//! The pane: opened by its word and its chord, every knob painted, a slider
//! and a check writing the field, the pin on the bar's own fact, and the
//! defaults link.

use super::{DEFAULTS, HEADING, PINNED, WORD, render};
use crate::paint_probe::frame::Window;
use crate::settings::{KNOBS, Settings};
use crate::test_support::window::{click, pane, seated};
use crate::ui::{Lookup, Model};

/// A seat with the pane standing.
fn open() -> Model {
    Model {
        lookup: Some(Lookup::Settings),
        ..seated()
    }
}

/// A closed pane paints nothing and says so.
#[test]
fn a_shut_pane_paints_nothing_and_reports_it() {
    let mut model = seated();
    let mut stood = true;
    let painted = pane(|ui| stood = render(ui, &mut model));
    assert!(!stood);
    assert!(!painted.contains(HEADING), "{painted}");
}

/// **Every knob is a row**: its label and its sentence, the pin's row and the
/// defaults link beside them — the table painted, with no list kept here.
#[test]
fn every_knob_is_painted_with_its_sentence() {
    let mut model = open();
    let mut stood = false;
    let painted = pane(|ui| stood = render(ui, &mut model));
    assert!(stood);
    for word in [HEADING, PINNED, DEFAULTS] {
        assert!(painted.contains(word), "{word:?}:\n{painted}");
    }
    for knob in KNOBS {
        assert!(painted.contains(knob.label), "{}:\n{painted}", knob.key);
        assert!(painted.contains(knob.help), "{}:\n{painted}", knob.key);
    }
}

/// **The word and the chord open it, and Esc puts it down** — the one pane
/// on the menu that posts nothing.
#[test]
fn the_word_and_the_chord_open_it_and_escape_closes_it() {
    let mut model = seated();
    assert!(crate::ui::slash::fire(&mut model, WORD, ""));
    assert_eq!(model.lookup, Some(Lookup::Settings));
    assert!(model.covered());
    assert!(model.outbox.is_empty(), "nothing crosses the boundary");
    model.escape();
    assert!(!model.covered());
    let window = Window::new();
    let chord = egui::Event::Key {
        key: egui::Key::Comma,
        physical_key: None,
        pressed: true,
        repeat: false,
        modifiers: egui::Modifiers::COMMAND,
    };
    window.frame(vec![chord], |ctx| crate::ui::render(ctx, &mut model));
    assert_eq!(model.lookup, Some(Lookup::Settings), "Ctrl+, opens it");
    assert!(
        crate::ui::slash::rows(&model)
            .iter()
            .any(|row| row.word == WORD),
        "on the menu"
    );
}

/// **A check writes its field and the pin's row writes the bar's**: one fact,
/// two doors, and the defaults link puts the knobs back and leaves the pin,
/// which is not a knob. The pin is clicked last: pinned, the bar eases open
/// on the clock, and a click aimed on one frame lands beside what moved.
#[test]
fn a_check_writes_the_field_the_pin_row_writes_the_bar_and_defaults_reset() {
    let window = Window::new();
    let mut model = open();
    window.frame(Vec::new(), |ctx| crate::ui::render(ctx, &mut model));
    click(&window, "bubbles", |ctx| crate::ui::render(ctx, &mut model));
    assert!(
        !model.settings.on(crate::settings::Toggle::Bubbles),
        "the check flipped the field"
    );
    model.settings.wheel = 200.0;
    window.frame(Vec::new(), |ctx| crate::ui::render(ctx, &mut model));
    click(&window, DEFAULTS, |ctx| crate::ui::render(ctx, &mut model));
    assert_eq!(model.settings, Settings::default());
    click(&window, PINNED, |ctx| crate::ui::render(ctx, &mut model));
    assert!(model.bar.pinned, "the pin's row is the bar's own fact");
}

/// **A slider writes its field through the row's setter**, and the arrow
/// keys move the one that holds the keyboard — how the pane is worked with
/// no pointer (DESIGN §4.46 item 3).
#[test]
fn a_slider_moves_on_the_arrows_and_writes_its_field() {
    let window = Window::new();
    let mut model = open();
    window.frame(Vec::new(), |ctx| crate::ui::render(ctx, &mut model));
    // **The trough stands left of its number**: the slider lays its run,
    // then the value, then the label, so a point left of the value's run by
    // more than the gap is on the trough — a press there moves the knob to
    // where the pointer landed and hands the slider the keyboard.
    let value = crate::paint_probe::frame::locate_in(&window, "100 pt", |ctx| {
        crate::ui::render(ctx, &mut model);
    })
    .expect("the wheel's number is on the glass");
    let trough = egui::pos2(value.x - 60.0, value.y);
    crate::paint_probe::frame::click(&window, trough, |ctx| crate::ui::render(ctx, &mut model));
    let pressed = model.settings.wheel;
    assert!((pressed - 100.0).abs() > f32::EPSILON, "{pressed}");
    let right = egui::Event::Key {
        key: egui::Key::ArrowRight,
        physical_key: None,
        pressed: true,
        repeat: false,
        modifiers: egui::Modifiers::NONE,
    };
    window.frame(vec![right], |ctx| crate::ui::render(ctx, &mut model));
    assert!(
        (model.settings.wheel - pressed).abs() > f32::EPSILON,
        "{pressed} -> {}",
        model.settings.wheel
    );
}

/// **A row whose kind and value disagree paints no control**: the table is
/// judged before any pane paints it (`crate::settings::tests`), so the arm
/// exists to refuse a control that lies rather than to be reached.
#[test]
fn a_row_whose_kind_and_value_disagree_paints_no_control() {
    let lying = crate::settings::Knob {
        key: "lying",
        label: "a knob that lies",
        help: "its kind says a toggle and its value is a number",
        kind: crate::settings::Kind::Toggle,
        get: |_| crate::settings::Knobbed::At(1.0),
        set: |_, _| {},
    };
    let mut settings = Settings::default();
    let painted = pane(|ui| super::row(ui, &mut settings, &lying));
    assert!(painted.contains(lying.help), "{painted}");
    assert!(!painted.contains(lying.label), "no control:\n{painted}");
}
