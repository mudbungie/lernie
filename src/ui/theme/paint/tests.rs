//! The anatomy on the glass: a row's words, ink, rule and tint; a block's
//! rule; a section's word; a field's ring and glow.

use super::{elbow, field, rail, row, ruled, section};
use crate::paint_probe::frame::Window;
use crate::test_support::window::{click, seen};
use crate::ui::theme::{BRAND, INK, INK_WEAK, RAISED, RULE, State, accent, tint};

/// Every fill that reached the glass, widest first.
fn fills(window: &Window, body: impl FnMut(&egui::Context)) -> Vec<(egui::Rect, egui::Color32)> {
    crate::paint_probe::fills_of(&window.frame(Vec::new(), body))
}

/// **A row's words reach the glass in the ink it was given, elided at its
/// width**, and its state stands as a rule at its left edge.
#[test]
fn a_row_paints_its_words_in_its_ink_and_its_state_as_a_rule() {
    let window = Window::sized(200.0, 100.0);
    let body = |ctx: &egui::Context| {
        egui::CentralPanel::default().show(ctx, |ui| {
            row(
                ui,
                "a conversation whose name is far too long for two hundred points",
                INK_WEAK,
                Some(accent(State::Attention)),
                false,
                0.0,
            );
        });
    };
    let glyphs = seen(&window, body);
    let run = glyphs.first().expect("the words reached the glass");
    assert!(run.text.ends_with('…'), "elided: {:?}", run.text);
    assert_eq!(run.ink, INK_WEAK);
    let rule = fills(&window, body)
        .into_iter()
        .find(|(rect, ink)| *ink == accent(State::Attention) && rect.width() <= RULE + 0.5);
    assert!(rule.is_some(), "the attention rule stands at the edge");
}

/// **A chosen row is raised and wears the brand**, whatever its state says —
/// the selection is where the operator is.
#[test]
fn a_chosen_row_is_raised_and_wears_the_brand() {
    let window = Window::sized(300.0, 100.0);
    let body = |ctx: &egui::Context| {
        egui::CentralPanel::default().show(ctx, |ui| {
            row(ui, "chosen", INK, Some(accent(State::Error)), true, 0.0);
        });
    };
    let fills = fills(&window, body);
    assert!(fills.iter().any(|(_, ink)| *ink == RAISED), "raised");
    assert!(fills.iter().any(|(_, ink)| *ink == BRAND), "the brand rule");
    assert!(
        !fills.iter().any(|(_, ink)| *ink == accent(State::Error)),
        "and not the state's"
    );
}

/// **A row is a control**: a click on its words lands on it.
#[test]
fn a_row_takes_a_click_on_its_words() {
    let window = Window::sized(300.0, 100.0);
    let mut clicked = false;
    click(&window, "take me", |ctx| {
        egui::CentralPanel::default().show(ctx, |ui| {
            clicked |= row(ui, "take me", INK, None, false, 0.0).clicked();
        });
    });
    assert!(clicked);
}

/// **A ruled block's rule spans its body**, and a section's word is on the
/// glass in weak ink under its hairline.
#[test]
fn a_block_is_ruled_and_a_section_is_worded() {
    let window = Window::sized(300.0, 300.0);
    let body = |ctx: &egui::Context| {
        egui::CentralPanel::default().show(ctx, |ui| {
            ruled(ui, BRAND, |ui| {
                ui.label("first line");
                ui.label("second line");
            });
            section(ui, "steps");
            rail(ui, 4.0, 0.0, 10.0);
            elbow(ui, 4.0, 0.0, 5.0, 12.0);
        });
    };
    let glyphs = seen(&window, body);
    let first = glyphs
        .iter()
        .find(|run| run.text == "first line")
        .expect("first");
    let second = glyphs
        .iter()
        .find(|run| run.text == "second line")
        .expect("second");
    let rule = fills(&window, body)
        .into_iter()
        .find(|(rect, ink)| *ink == BRAND && rect.width() <= RULE + 0.5)
        .expect("the rule is on the glass");
    assert!(rule.0.min.y <= first.laid.min.y + 1.0 && rule.0.max.y >= second.laid.max.y - 1.0);
    let word = glyphs
        .iter()
        .find(|run| run.text == "steps")
        .expect("the section's word");
    assert_eq!(word.ink, INK_WEAK);
}

/// **A field glows with what it was given and rings when it holds the
/// caret**, and the words typed into it are the caller's.
#[test]
fn a_field_glows_and_rings() {
    let window = Window::sized(300.0, 100.0);
    let id = egui::Id::new("a field");
    let mut text = String::new();
    let glow = tint(State::Attention);
    let body = |ctx: &egui::Context| {
        egui::CentralPanel::default().show(ctx, |ui| {
            field(ui, id, &mut text, "say it", Some(glow));
        });
    };
    assert!(
        fills(&window, body).iter().any(|(_, ink)| *ink == glow),
        "glows"
    );
    let mut text = String::new();
    let body = |ctx: &egui::Context| {
        egui::CentralPanel::default().show(ctx, |ui| {
            field(ui, id, &mut text, "say it", None);
        });
    };
    click(&window, "say it", body);
    assert_eq!(window.focused(), Some(id), "the click put the caret in it");
}
