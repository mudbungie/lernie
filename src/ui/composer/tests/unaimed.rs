//! **The box with nothing aimed** (bl-2cb0): it stands, the window's own
//! verbs are typed into it and fire, and prose typed into it goes nowhere.
//!
//! Its own file because the subject is the one mode the rest of the suite
//! never reaches: every other composer test is seated on a wall.

use super::super::{NOWHERE, UNAIMED};
use crate::paint_probe::frame::{Window, locate_in, press};
use crate::ui::theme::glyph;
use crate::ui::{Listing, Model};

/// The whole hint at a window's width, where every part fits.
fn said() -> String {
    std::iter::once(NOWHERE)
        .chain(UNAIMED)
        .collect::<Vec<_>>()
        .join(crate::ui::chat::panel::PART)
}

/// **Type `text` into the whole window's box and press Enter**, the way an
/// operator does: a click on the hint puts the caret there.
fn typed(model: &mut Model, text: &str) -> Window {
    let window = Window::new();
    let mut body = |ctx: &egui::Context| crate::ui::render(ctx, model);
    // Two idle frames first: a bottom panel takes a frame to find its height.
    for _ in 0..2 {
        window.frame(Vec::new(), &mut body);
    }
    let at = locate_in(&window, &said(), &mut body).expect("the box is on the glass");
    crate::paint_probe::frame::click(&window, at, &mut body);
    window.frame(vec![egui::Event::Text(text.to_owned())], &mut body);
    window.frame(vec![press(egui::Key::Enter)], &mut body);
    window
}

/// **The field is on the glass with nothing aimed**, its glyph in the brand
/// and its hint saying nothing is aimed and how to reach a verb.
#[test]
fn the_box_stands_with_nothing_aimed() {
    let mut model = Model::default();
    let seen = crate::test_support::window::seen(&Window::new(), |ctx| {
        crate::ui::render(ctx, &mut model);
    });
    assert!(seen.iter().any(|run| run.text == said()), "{seen:?}");
    let lead = seen
        .iter()
        .find(|run| run.text == glyph::PROMPT)
        .expect("the glyph is on the glass");
    assert_eq!(lead.ink, crate::ui::theme::BRAND);
}

/// **`/att` and Enter open the queue on a seat aimed at nothing** — DESIGN
/// §4.19's seat *most likely to be asking the question*.
#[test]
fn a_slash_verb_that_needs_no_aim_fires_with_nothing_aimed() {
    let mut model = Model::default();
    typed(&mut model, "/att");
    assert!(model.showing(Listing::Queue), "the queue stands");
    assert_eq!(model.draft, "", "fired, so no longer a draft");
}

/// **Prose with nothing aimed composes nothing** and costs nothing: there is
/// no wall for a deposit or a start, and what was typed is still the
/// operator's.
#[test]
fn prose_with_nothing_aimed_deposits_nothing_and_stays() {
    let mut model = Model::default();
    typed(&mut model, "ship it");
    assert!(model.outbox.is_empty(), "{:?}", model.outbox);
    assert_eq!(model.draft.trim_end(), "ship it");
}

/// **The menu offers exactly the verbs that need no aim**: the window's own
/// are listed, and none whose subject is a wall or a conversation.
#[test]
fn the_menu_with_nothing_aimed_lists_only_the_window_verbs() {
    let words: Vec<&str> = crate::ui::slash::rows(&Model::default())
        .iter()
        .map(|row| row.word)
        .collect();
    for offered in [
        crate::ui::queue::WORD,
        crate::ui::commands::WORD,
        crate::ui::doctor::WORD,
        crate::ui::prices::WORD,
        crate::ui::board::WORD,
        crate::ui::find::WORD,
        crate::ui::trail::WORD,
    ] {
        assert!(words.contains(&offered), "{offered} is offered: {words:?}");
    }
    for withheld in [
        crate::ui::enroll::WORD,
        crate::ui::fleet::WORD,
        crate::ui::login::WORD,
        crate::ui::tuning::WORD,
        crate::ui::records::WORD,
        crate::ui::slash::PLAN,
        crate::verbs::PIN.word,
        crate::verbs::STOP.word,
    ] {
        assert!(
            !words.contains(&withheld),
            "{withheld} needs an aim: {words:?}"
        );
    }
}
