//! The switcher: Ctrl+X opens it, the roster stands in it, walking keeps it,
//! choosing closes it.

use super::{HEADING, WORD};
use crate::paint_probe::frame::{Window, press};
use crate::test_support::window::{click, escape, painted, seated};
use crate::ui::Model;

/// **The one chord, as the platform delivers it**: egui-winit turns Ctrl+X
/// into `Event::Cut` and sends no key at all (bl-8c49), so a beat that
/// pressed `Key::X` with the command modifier proved a chord no window sees.
fn ctrl_x(window: &Window, model: &mut Model) {
    window.frame(vec![egui::Event::Cut], |ctx| crate::ui::render(ctx, model));
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

/// **The chord is the switcher's, not the box's**: with the caret in a draft
/// whose words are all selected, Ctrl+X opens the switcher and cuts nothing.
#[test]
fn ctrl_x_in_the_box_cuts_nothing() {
    let mut model = seated();
    "keep every word".clone_into(&mut model.draft);
    let window = Window::new();
    let id = egui::Id::new(crate::ui::keys::BOX_ID);
    window.focus(id);
    window.frame(Vec::new(), |ctx| {
        crate::ui::render(ctx, &mut model);
        let mut state = egui::text_edit::TextEditState::load(ctx, id).unwrap_or_default();
        state
            .cursor
            .set_char_range(Some(egui::text::CCursorRange::two(
                egui::text::CCursor::new(0),
                egui::text::CCursor::new(model.draft.len()),
            )));
        state.store(ctx, id);
    });
    ctrl_x(&window, &mut model);
    window.frame(Vec::new(), |ctx| crate::ui::render(ctx, &mut model));
    assert!(model.switching);
    assert_eq!(model.draft, "keep every word");
}

/// **Ctrl+X over an open pane replaces it** (bl-339d): the pane is put
/// down, all the way, so choosing a conversation shows that conversation —
/// and over the enrollment the chord does nothing.
#[test]
fn ctrl_x_replaces_the_pane_that_stood() {
    let window = Window::new();
    let mut model = seated();
    model.begin_trail();
    model.begin_clearing();
    ctrl_x(&window, &mut model);
    assert!(model.switching);
    assert!(
        !model.trailing() && !model.clearing(),
        "put down, step and all"
    );
    model.chose();
    assert!(!model.covered());
    model.begin_enrollment();
    ctrl_x(&window, &mut model);
    assert!(!model.switching && model.enroll.is_some());
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
    // Down onto the workspace's `+ new` and Up back onto its one conversation.
    for key in [egui::Key::ArrowDown, egui::Key::ArrowUp] {
        window.frame(vec![press(key)], |ctx| crate::ui::render(ctx, &mut model));
    }
    assert!(model.switching, "the walk keeps it");
    assert!(model.standing.is_none() && model.conversation.is_some());
    window.frame(vec![press(egui::Key::Enter)], |ctx| {
        crate::ui::render(ctx, &mut model);
    });
    assert!(!model.switching, "Enter on the selection chooses");
    assert!(model.conversation.is_some(), "and keeps it");

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

/// **`+ new` chooses too**: it begins a conversation, which is what the
/// switcher was opened for.
#[test]
fn the_plus_closes_it_on_the_way_to_the_box() {
    let mut model = seated();
    model.switching = true;
    let window = Window::new();
    click(&window, crate::ui::roster::workspace::NEW, |ctx| {
        crate::ui::render(ctx, &mut model);
    });
    assert!(!model.switching);
    assert_eq!(model.conversation, None);
}

/// **A keyboard walk onto a server's row brings it into view** in the
/// switcher, as it did in the column: the row the cursor stands on is
/// scrolled to when the walk says the list owes it a place.
#[test]
fn the_walk_brings_the_standing_server_into_view() {
    let mut model = seated();
    model.switching = true;
    model.standing = Some(crate::ui::Stand::Server(
        "(this box's own server)".to_owned(),
    ));
    model.reveal = true;
    let text = painted(&mut model);
    assert!(text.contains("(this box's own server)"), "{text}");
}
