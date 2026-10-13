//! The switcher: Ctrl+X opens it, the roster stands in it, walking keeps it,
//! choosing closes it.

use super::{HEADING, WORD};
use crate::paint_probe::frame::{Window, press};
use crate::test_support::window::{click, escape, painted, seated};
use crate::ui::Model;

/// The one chord, pressed on the whole window.
fn ctrl_x(window: &Window, model: &mut Model) {
    window.frame(
        vec![egui::Event::Key {
            key: egui::Key::X,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers: egui::Modifiers::COMMAND,
        }],
        |ctx| crate::ui::render(ctx, model),
    );
}

/// **Ctrl+X toggles it, from the box too**, and `/conversations` opens it.
#[test]
fn ctrl_x_toggles_it_and_the_word_opens_it() {
    let mut model = seated();
    let window = Window::new();
    ctrl_x(&window, &mut model);
    assert!(model.switching);
    window.focus(egui::Id::new(crate::ui::keys::BOX_ID));
    ctrl_x(&window, &mut model);
    assert!(!model.switching);
    assert!(crate::ui::slash::fire(&mut model, WORD, ""));
    assert!(model.switching);
}

/// **The roster stands in it, under its name, and the prompt stands down.**
#[test]
fn the_roster_stands_in_it() {
    let mut model = seated();
    model.switching = true;
    let text = painted(&mut model);
    for word in [HEADING, "home", "port the paint probe"] {
        assert!(text.contains(word), "{word:?}:\n{text}");
    }
    assert!(!text.contains(crate::ui::composer::HINT), "{text}");
}

/// **Walking keeps it open; a click on a conversation, Enter on the
/// selection and Esc close it.**
#[test]
fn walking_keeps_it_open_and_choosing_closes_it() {
    let mut model = seated();
    model.switching = true;
    let window = Window::new();
    window.frame(vec![press(egui::Key::ArrowDown)], |ctx| {
        crate::ui::render(ctx, &mut model);
    });
    assert!(model.switching, "the walk keeps it");
    window.frame(vec![press(egui::Key::Enter)], |ctx| {
        crate::ui::render(ctx, &mut model);
    });
    assert!(!model.switching, "Enter on the selection chooses");

    let mut model = seated();
    model.switching = true;
    let label = crate::ui::convs::headline(&model.convs[0]);
    click(&window, &label, |ctx| crate::ui::render(ctx, &mut model));
    assert!(!model.switching, "a click chooses");
    assert_eq!(model.conversation.as_deref(), Some("20260830T051200Z-a1b2"));

    let mut model = seated();
    model.switching = true;
    escape(&window, |ctx| crate::ui::render(ctx, &mut model));
    assert!(!model.switching, "Esc leaves");
}

/// **The `+` chooses too**: it begins a conversation, which is what the
/// switcher was opened for.
#[test]
fn the_plus_closes_it_on_the_way_to_the_box() {
    let mut model = seated();
    model.switching = true;
    let window = Window::new();
    click(&window, "+", |ctx| crate::ui::render(ctx, &mut model));
    assert!(!model.switching);
    assert_eq!(model.conversation, None);
}

/// **A keyboard walk onto an engine's row brings it into view** in the
/// switcher, as it did in the column: the row the cursor stands on is
/// scrolled to when the walk says the list owes it a place.
#[test]
fn the_walk_brings_the_standing_engine_into_view() {
    let mut model = seated();
    model.switching = true;
    model.standing = Some("(this box's own engine)".to_owned());
    model.reveal = true;
    let text = painted(&mut model);
    assert!(text.contains("(this box's own engine)"), "{text}");
}
