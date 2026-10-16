//! **Every fired row of the table drives its act through a real window**
//! (DESIGN §4.46 item 3), and the table reads as the operator types it.

use super::seated;
use crate::paint_probe::frame::{Window, chord};
use crate::settings::SCALE;
use crate::test_support::window::{conv, said};
use crate::ui::chat::scroll;
use crate::ui::keys::{Act, BINDINGS, Binding, Chord, Keyed, chord_for, cited, cited_word};
use crate::ui::{Lookup, Model};

/// Every act the table can name.
const ACTS: [Act; 17] = [
    Act::Switch,
    Act::Model,
    Act::Dock,
    Act::Settings,
    Act::Help,
    Act::Pin,
    Act::New,
    Act::CloseTab,
    Act::NextTab,
    Act::PrevTab,
    Act::PageDown,
    Act::PageUp,
    Act::End,
    Act::Head,
    Act::ScaleUp,
    Act::ScaleDown,
    Act::ScaleReset,
];

/// Press one chord on a real window, then rest a frame.
fn chorded(window: &Window, model: &mut Model, key: egui::Key, modifiers: egui::Modifiers) {
    window.frame(vec![chord(key, modifiers)], |ctx| {
        crate::ui::render(ctx, model);
    });
    window.frame(Vec::new(), |ctx| crate::ui::render(ctx, model));
}

/// Press the row that fires `act`.
fn fire(window: &Window, model: &mut Model, act: Act) {
    let row = BINDINGS
        .iter()
        .find_map(|row| match row.keyed {
            Keyed::Fired(chord, fires) if fires == act => Some(chord),
            _ => None,
        })
        .expect("every act has a row");
    let modifiers = egui::Modifiers {
        command: row.ctrl,
        ctrl: row.ctrl,
        shift: row.shift,
        ..egui::Modifiers::NONE
    };
    chorded(window, model, row.key, modifiers);
}

#[test]
fn every_act_has_a_row_and_the_chords_spell_as_the_operator_types_them() {
    for act in ACTS {
        assert!(!chord_for(act).is_empty(), "{act:?} has no row");
    }
    assert_eq!(chord_for(Act::Settings), "Ctrl+,");
    assert_eq!(chord_for(Act::ScaleUp), "Ctrl+=");
    assert_eq!(chord_for(Act::ScaleDown), "Ctrl+-");
    assert_eq!(chord_for(Act::ScaleReset), "Ctrl+0");
    assert_eq!(chord_for(Act::PageDown), "PageDown");
    assert_eq!(chord_for(Act::Help), "F1");
    let shifted = Chord {
        key: egui::Key::Enter,
        ctrl: false,
        shift: true,
    };
    assert_eq!(shifted.spell(), "Shift+Enter");
    assert_eq!(Chord::plain(egui::Key::Escape).spell(), "Esc");
    // **The constructors at run time**, not only in the table's const
    // evaluation, so the builder's count reads them as lines that ran.
    let row = Binding::fired(Chord::ctrl(egui::Key::B), Act::Pin, "pins");
    assert_eq!(row.says(), "Ctrl+B");
    assert_eq!(Binding::told("Enter", "sends").says(), "Enter");
    let told: Vec<String> = BINDINGS
        .iter()
        .filter(|row| matches!(row.keyed, Keyed::Told(_)))
        .map(Binding::says)
        .collect();
    assert!(told.contains(&"Shift+Enter".to_owned()), "{told:?}");
}

#[test]
fn a_sentence_cites_its_chords_and_a_word_cites_the_chord_that_opens_it() {
    assert_eq!(cited("does", &[]), "does");
    assert_eq!(
        cited("does", &[Act::Pin, Act::Dock]),
        "does · Ctrl+B / Ctrl+T"
    );
    assert_eq!(
        cited_word("opens", crate::ui::settings::WORD),
        "opens · Ctrl+,"
    );
    assert_eq!(cited_word("opens", crate::ui::doctor::WORD), "opens");
}

#[test]
fn the_chords_open_the_switcher_the_model_the_dock_the_settings_and_help() {
    let window = Window::new();
    let mut model = seated();
    fire(&window, &mut model, Act::Switch);
    assert!(model.switching);
    model.chose();
    fire(&window, &mut model, Act::Model);
    assert!(model.dropup.is_some(), "Ctrl+O opens the model's dropup");
    model.escape();
    fire(&window, &mut model, Act::Settings);
    assert_eq!(model.lookup, Some(Lookup::Settings));
    fire(&window, &mut model, Act::Help);
    assert_eq!(model.lookup, Some(Lookup::Commands));
    model.escape();
    let before = crate::ui::dock::unfolded(window.ctx(), false);
    fire(&window, &mut model, Act::Dock);
    assert_ne!(crate::ui::dock::unfolded(window.ctx(), false), before);
}

#[test]
fn ctrl_b_pins_the_bar_and_ctrl_n_begins_where_plus_new_would() {
    let window = Window::new();
    let mut model = seated();
    fire(&window, &mut model, Act::Pin);
    assert!(model.bar.pinned);
    fire(&window, &mut model, Act::Pin);
    assert!(!model.bar.pinned);
    assert!(model.conversation.is_some());
    fire(&window, &mut model, Act::New);
    assert_eq!(model.conversation, None, "a start is no conversation");
    // Under a pane, `+ new` is off the glass and so is the chord.
    let mut covered = seated();
    covered.lookup = Some(Lookup::Commands);
    fire(&window, &mut covered, Act::New);
    assert!(covered.conversation.is_some());
}

/// Two conversations open as tabs, the second selected.
fn tabbed() -> Model {
    let mut model = seated();
    model
        .convs
        .push(conv("20260830T052000Z-b2c3", "tidy the rule"));
    for root in ["20260830T051200Z-a1b2", "20260830T052000Z-b2c3"] {
        model.select(root);
    }
    model
}

#[test]
fn the_tab_chords_step_the_strip_and_close_the_selected_tab() {
    let window = Window::new();
    let mut model = tabbed();
    assert_eq!(model.conversation.as_deref(), Some("20260830T052000Z-b2c3"));
    fire(&window, &mut model, Act::PrevTab);
    assert_eq!(model.conversation.as_deref(), Some("20260830T051200Z-a1b2"));
    fire(&window, &mut model, Act::PrevTab);
    assert_eq!(
        model.conversation.as_deref(),
        Some("20260830T051200Z-a1b2"),
        "the strip saturates"
    );
    fire(&window, &mut model, Act::NextTab);
    assert_eq!(model.conversation.as_deref(), Some("20260830T052000Z-b2c3"));
    fire(&window, &mut model, Act::CloseTab);
    assert_eq!(model.strip().len(), 1);
    assert_eq!(model.conversation.as_deref(), Some("20260830T051200Z-a1b2"));
    // Nothing selected: the next tab is the first, and there is nothing to close.
    model.conversation = None;
    fire(&window, &mut model, Act::CloseTab);
    assert_eq!(model.strip().len(), 1);
    fire(&window, &mut model, Act::NextTab);
    assert_eq!(model.conversation.as_deref(), Some("20260830T051200Z-a1b2"));
}

#[test]
fn the_scale_chords_move_the_knob_within_its_bounds_and_put_it_back() {
    let window = Window::new();
    let mut model = seated();
    fire(&window, &mut model, Act::ScaleUp);
    assert!(
        (model.settings.scale - 1.1).abs() < 1e-5,
        "{}",
        model.settings.scale
    );
    fire(&window, &mut model, Act::ScaleDown);
    assert!((model.settings.scale - SCALE).abs() < 1e-5);
    model.settings.scale = 2.0;
    fire(&window, &mut model, Act::ScaleUp);
    assert!(
        (model.settings.scale - 2.0).abs() < 1e-5,
        "clamped at the ceiling"
    );
    fire(&window, &mut model, Act::ScaleReset);
    assert!((model.settings.scale - SCALE).abs() < 1e-5);
    // The keyboard whose `=` is `+` under Shift presses Ctrl+Plus.
    chorded(
        &window,
        &mut model,
        egui::Key::Plus,
        egui::Modifiers::COMMAND,
    );
    assert!(
        (model.settings.scale - 1.1).abs() < 1e-5,
        "{}",
        model.settings.scale
    );
    // A plain key is not its chord: `=` alone goes to the box.
    model.settings.scale = SCALE;
    chorded(
        &window,
        &mut model,
        egui::Key::Equals,
        egui::Modifiers::NONE,
    );
    assert!((model.settings.scale - SCALE).abs() < 1e-5);
}

/// A transcript far taller than a short window.
fn tall() -> Model {
    let mut model = seated();
    model.transcript.entries = (0..120)
        .map(|i| said("op", &format!("line {i} of a long conversation")))
        .collect();
    model
}

#[test]
fn page_keys_scroll_the_transcript_and_the_ends_reach_its_ends() {
    let window = Window::sized(800.0, 400.0);
    let mut model = tall();
    for _ in 0..3 {
        window.frame(Vec::new(), |ctx| crate::ui::render(ctx, &mut model));
    }
    let bottom = scroll::offset(window.ctx()).expect("the transcript noted where it stands");
    assert!(bottom > 400.0, "stuck to the bottom: {bottom}");
    fire(&window, &mut model, Act::PageUp);
    let up = scroll::offset(window.ctx()).expect("noted");
    assert!(up < bottom, "a page back: {up} < {bottom}");
    fire(&window, &mut model, Act::Head);
    assert_eq!(scroll::offset(window.ctx()), Some(0.0));
    fire(&window, &mut model, Act::PageDown);
    let down = scroll::offset(window.ctx()).expect("noted");
    assert!(down > 0.0 && down < bottom, "a page on: {down}");
    fire(&window, &mut model, Act::End);
    let end = scroll::offset(window.ctx()).expect("noted");
    assert!((end - bottom).abs() < 1.0, "the end: {end} vs {bottom}");
    // Under a pane the transcript is not what the hand is on. (The pane
    // stands the composer down, so the area is taller and its end moves;
    // what is asserted is that the key moved nothing from there.)
    model.lookup = Some(Lookup::Commands);
    for _ in 0..2 {
        window.frame(Vec::new(), |ctx| crate::ui::render(ctx, &mut model));
    }
    let under = scroll::offset(window.ctx()).expect("noted");
    fire(&window, &mut model, Act::Head);
    let held = scroll::offset(window.ctx()).expect("noted");
    assert!(
        (held - under).abs() < 1.0,
        "held under the pane: {held} vs {under}"
    );
}
