//! The mark and the offered ids, read off the glass, and the click that fills
//! the field.

use super::OFFERS;
use crate::paint_probe::frame::Window;
use crate::reply::{Read, Reply};
use crate::test_support::window::{click, role, seen, tuned};
use crate::ui::theme::{State, accent};
use crate::ui::tuning::render;
use crate::ui::{Edit, Model, Tuning};

/// The tuning pane with `housevendor` having answered `ids`.
fn listed(ids: &[&str]) -> Model {
    let mut model = tuned();
    let ids = ids.iter().map(|id| (*id).to_owned()).collect();
    model.offering("housevendor", Read::Answer(Reply::Models(ids)));
    model
}

/// What one frame of the pane put on the glass, as `(glyphs, ink)` runs.
fn glass(window: &Window, model: &mut Model) -> Vec<(String, egui::Color32)> {
    seen(window, |ctx| {
        egui::CentralPanel::default().show(ctx, |ui| {
            render(ui, model);
        });
    })
    .into_iter()
    .map(|run| (run.text, run.ink))
    .collect()
}

/// **A row whose provider's listing lacks its model says so, on the row, in
/// the error accent** — and with no listing nothing is said, because absence
/// is not evidence.
#[test]
fn a_row_on_a_model_its_provider_does_not_offer_is_marked_in_the_error_accent() {
    let sentence = "housevendor does not offer house-model-1";
    let window = Window::new();
    let runs = glass(&window, &mut listed(&["house-model-2"]));
    assert!(
        runs.contains(&(sentence.to_owned(), accent(State::Error))),
        "{runs:?}"
    );
    let runs = glass(&window, &mut tuned());
    assert!(!runs.iter().any(|(text, _)| text == sentence), "{runs:?}");
    let runs = glass(&window, &mut listed(&["house-model-1"]));
    assert!(!runs.iter().any(|(text, _)| text == sentence), "{runs:?}");
}

/// **The editor offers the provider's ids, and a click on one fills the model
/// field** — the field stays the draft, so `set` sends what was picked.
#[test]
fn the_editor_offers_the_listed_ids_and_a_click_fills_the_model_field() {
    let mut model = listed(&["house-model-2", "house-model-3"]);
    model.tuning = Some(Tuning::Editing(Edit::of(&role("worker"))));
    let window = Window::new();
    let runs = glass(&window, &mut model);
    for word in [OFFERS, "house-model-2", "house-model-3"] {
        assert!(
            runs.iter().any(|(text, _)| text == word),
            "{word:?}: {runs:?}"
        );
    }
    click(&window, "house-model-3", |ctx| {
        egui::CentralPanel::default().show(ctx, |ui| {
            render(ui, &mut model);
        });
    });
    assert_eq!(
        model.editing("worker").map(|edit| edit.model).as_deref(),
        Some("house-model-3")
    );
}

/// **A provider with no listing offers nothing to pick** — the editor is the
/// two fields it always was.
#[test]
fn a_provider_with_no_listing_offers_no_ids() {
    let mut model = tuned();
    model.tuning = Some(Tuning::Editing(Edit::of(&role("worker"))));
    let runs = glass(&Window::new(), &mut model);
    assert!(!runs.iter().any(|(text, _)| text == OFFERS), "{runs:?}");
}
