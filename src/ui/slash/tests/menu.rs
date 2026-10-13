//! The menu over the box: what it shows while the draft is a slash line, and
//! what the keys and a click on a row do.

use super::super::{HINT, LEAD, NOTHING, Row, rows};
use crate::test_support::window::{painted, seated};
use crate::ui::Model;
use serde_json::json;

/// **The menu stands over the box while the draft is a slash line**, every
/// offered row on it with the keys under them, and says so when nothing
/// answers.
#[test]
fn the_menu_stands_while_the_draft_is_a_slash_line() {
    let mut model = seated();
    let quiet = painted(&mut model);
    assert!(!quiet.contains(HINT), "{quiet}");
    model.draft = LEAD.to_string();
    let text = painted(&mut model);
    assert!(text.contains(HINT), "{text}");
    for row in rows(&model) {
        assert!(text.contains(&row.line()), "{}:\n{text}", row.line());
    }
    model.draft = "/zzz".to_owned();
    let text = painted(&mut model);
    assert!(text.contains(NOTHING), "{text}");
}

/// **The keys over the menu**: ↓ moves the choice, Tab completes the chosen
/// word into the box, Enter fires it and clears the box, Escape clears it.
#[test]
fn the_keys_move_complete_fire_and_clear() {
    use crate::paint_probe::frame::{Window, press};
    let window = Window::new();
    let key = |model: &mut Model, key| {
        window.frame(vec![press(key)], |ctx| crate::ui::render(ctx, model));
    };
    let mut model = seated();
    model.draft = "/re".to_owned();
    // The caret is in the box, as it is whenever a slash line is being typed.
    window.frame(Vec::new(), |ctx| crate::ui::render(ctx, &mut model));
    window.focus(egui::Id::new(crate::ui::keys::BOX_ID));
    key(&mut model, egui::Key::ArrowDown);
    key(&mut model, egui::Key::Tab);
    assert_eq!(model.draft, "/revoke ");
    key(&mut model, egui::Key::Enter);
    assert_eq!(model.draft, "", "fired, so no longer a draft");
    assert_eq!(
        model.outbox.last().map(|p| p.envelope["op"].clone()),
        Some(json!("revoke"))
    );
    model.draft = "/flag".to_owned();
    key(&mut model, egui::Key::Enter);
    assert_eq!(
        model.draft, "/flag",
        "a flag without its reason fires nothing and stays"
    );
    // **And says why, on the glass, until the draft moves** (bl-df70).
    let why =
        super::super::wanting(&model, crate::verbs::FLAG.word, "").expect("a reason is wanted");
    let said = crate::paint_probe::text_of(&window.frame(Vec::new(), |ctx| {
        crate::ui::render(ctx, &mut model);
    }));
    assert!(said.contains(&why), "{said}");
    model.draft = "/flag ".to_owned();
    let moved = crate::paint_probe::text_of(&window.frame(Vec::new(), |ctx| {
        crate::ui::render(ctx, &mut model);
    }));
    assert!(!moved.contains(&why), "{moved}");
    model.draft = "/flag".to_owned();
    key(&mut model, egui::Key::Escape);
    assert_eq!(model.draft, "");
    // Escape is also egui's *leave the box*, so the caret goes back in.
    window.focus(egui::Id::new(crate::ui::keys::BOX_ID));
    model.draft = "/".to_owned();
    key(&mut model, egui::Key::ArrowUp);
    key(&mut model, egui::Key::Tab);
    assert_eq!(
        model.draft, "/records ",
        "up from the first row stays on it"
    );
}

/// **A click on a row fires it**, exactly as Enter on it does.
#[test]
fn a_click_on_a_row_fires_it() {
    let window = crate::paint_probe::frame::Window::new();
    let mut model = seated();
    model.draft = "/st".to_owned();
    // Two idle frames first: the menu stands on the composer's panel, and a
    // resizable panel takes a frame to find its height.
    for _ in 0..2 {
        window.text(|ctx| crate::ui::render(ctx, &mut model));
    }
    let line = Row::new(crate::verbs::STOP.word, crate::verbs::STOP.summary, &[]).line();
    crate::test_support::window::click(&window, &line, |ctx| crate::ui::render(ctx, &mut model));
    assert_eq!(model.draft, "");
    assert_eq!(
        model.outbox.last().map(|p| p.envelope["op"].clone()),
        Some(json!("stop"))
    );
}
