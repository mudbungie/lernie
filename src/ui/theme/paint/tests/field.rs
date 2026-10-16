//! **The two fields that take text**: the box's glow and ring, and the
//! prompt's glyph and its line break — split from [`super`] at the cap.

use super::super::{field, prompt};
use super::fills;
use crate::paint_probe::frame::Window;
use crate::test_support::window::click;
use crate::ui::theme::{BRAND, State, glyph, tint, type_scale};

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
            field(ui, id, &mut text, "say it", Some(glow), "takes words");
        });
    };
    assert!(
        fills(&window, body).iter().any(|(_, ink)| *ink == glow),
        "glows"
    );
    let mut text = String::new();
    let body = |ctx: &egui::Context| {
        egui::CentralPanel::default().show(ctx, |ui| {
            field(ui, id, &mut text, "say it", None, "takes words");
        });
    };
    click(&window, "say it", body);
    assert_eq!(window.focused(), Some(id), "the click put the caret in it");
}

/// **The prompt wears its glyph in the brand and is as tall as its lines**
/// (§4.42): the glyph stands inside the field's fill, the field is one line
/// of body type tall while nothing is typed, and a shifted Enter breaks a
/// line where a bare one leaves the text alone.
#[test]
fn the_prompt_wears_its_glyph_and_shift_enter_breaks_a_line() {
    let window = Window::sized(400.0, 200.0);
    let id = egui::Id::new("the prompt");
    let mut text = String::new();
    let mut body = |ctx: &egui::Context| {
        egui::CentralPanel::default().show(ctx, |ui| {
            prompt(ui, id, &mut text, "say it", None, "takes words");
        });
    };
    let output = window.frame(Vec::new(), &mut body);
    let lead = crate::paint_probe::seen_of(&output)
        .into_iter()
        .find(|run| run.text == glyph::PROMPT)
        .expect("the glyph is on the glass");
    assert_eq!(lead.ink, BRAND);
    let field = crate::paint_probe::fills_of(&output)
        .into_iter()
        .find(|(rect, _)| {
            rect.contains_rect(lead.laid) && rect.width() > 300.0 && rect.height() < 100.0
        })
        .expect("the glyph lies inside the field");
    assert!(field.0.height() < 3.0 * type_scale::BODY, "{field:?}");
    let at = crate::paint_probe::frame::locate_in(&window, "say it", &mut body).expect("hint");
    crate::paint_probe::frame::click(&window, at, &mut body);
    window.frame(
        vec![egui::Event::Key {
            key: egui::Key::Enter,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers: egui::Modifiers::SHIFT,
        }],
        &mut body,
    );
    window.frame(
        vec![crate::paint_probe::frame::press(egui::Key::Enter)],
        &mut body,
    );
    window.frame(Vec::new(), &mut body);
    assert_eq!(
        text, "\n",
        "one break from the shifted key and none from the bare one"
    );
}
