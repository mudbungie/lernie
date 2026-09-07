//! The anatomy on the glass: a row's words, ink, rule and tint; a block's
//! rule; a section's word; a field's ring and glow.

use super::{composer, elbow, field, prose, rail, row, ruled, section};
use crate::paint_probe::frame::Window;
use crate::test_support::window::{click, seen};
use crate::ui::theme::{
    BRAND, COMPOSER_ROWS, INK, INK_WEAK, RAISED, RULE, State, accent, tint, type_scale,
};

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

/// **The row that holds the keyboard wears the brand ring** (bl-2d6b), and a
/// row at rest wears no stroke at all.
#[test]
fn a_row_rings_while_it_holds_the_keyboard_and_not_otherwise() {
    let window = Window::sized(300.0, 100.0);
    let mut body = |ctx: &egui::Context| {
        egui::CentralPanel::default().show(ctx, |ui| {
            row(ui, "first", INK, None, false, 0.0);
            row(ui, "second", INK, None, false, 0.0);
        });
    };
    let ringed = |output: &egui::FullOutput| {
        crate::paint_probe::strokes_of(output)
            .into_iter()
            .filter(|(_, ink)| *ink == BRAND)
            .count()
    };
    assert_eq!(ringed(&window.frame(Vec::new(), &mut body)), 0, "at rest");
    window.frame(
        vec![crate::paint_probe::frame::press(egui::Key::Tab)],
        &mut body,
    );
    let output = window.frame(Vec::new(), &mut body);
    assert_eq!(ringed(&output), 1, "one row holds the keyboard");
    let ring = crate::paint_probe::strokes_of(&output)
        .into_iter()
        .find(|(_, ink)| *ink == BRAND)
        .map(|(rect, _)| rect)
        .expect("the ring");
    let first = seen(&window, &mut body)
        .into_iter()
        .find(|run| run.text == "first")
        .expect("the first row");
    assert!(
        ring.contains_rect(first.laid),
        "the ring is around the first row"
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

/// **The composer stands its rows tall with the send inside it** (bl-f251):
/// the field's fill spans at least [`COMPOSER_ROWS`] lines of body type, the
/// send's words lie within that fill in the brand, and a shifted Enter breaks
/// a line where a bare one leaves the text alone.
#[test]
fn the_composer_is_rows_tall_with_the_send_inside_it() {
    let window = Window::sized(400.0, 200.0);
    let id = egui::Id::new("the composer");
    let mut text = String::new();
    let mut body = |ctx: &egui::Context| {
        egui::CentralPanel::default().show(ctx, |ui| {
            composer(ui, id, &mut text, "say it", None, "send");
        });
    };
    let output = window.frame(Vec::new(), &mut body);
    let send = crate::paint_probe::seen_of(&output)
        .into_iter()
        .find(|run| run.text == "send")
        .expect("the send is on the glass");
    assert_eq!(send.ink, BRAND);
    let field = crate::paint_probe::fills_of(&output)
        .into_iter()
        .find(|(rect, _)| rect.contains_rect(send.laid) && rect.width() > 300.0)
        .expect("the send lies inside the field");
    let rows = f32::from(COMPOSER_ROWS);
    assert!(field.0.height() >= rows * type_scale::BODY, "{field:?}");
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

/// **Prose is laid at the leading** (bl-f251): a paragraph that wraps to
/// two lines stands two leadings tall on the glass, where a label of the
/// same words stands two font heights — and the words reach the glass in
/// the ink they were given.
#[test]
fn prose_is_laid_at_the_leading_and_a_label_is_not() {
    let window = Window::sized(200.0, 200.0);
    let words = "a paragraph long enough that two hundred points cannot hold it on one line";
    let body = |ctx: &egui::Context| {
        egui::CentralPanel::default().show(ctx, |ui| {
            prose(ui, words, INK_WEAK);
            ui.label(words);
        });
    };
    let glyphs = seen(&window, body);
    let (paragraph, label) = (&glyphs[0], &glyphs[1]);
    assert_eq!(paragraph.ink, INK_WEAK);
    assert!(
        paragraph.laid.height() >= 2.0 * type_scale::LEADING - 1.0,
        "{:?}",
        paragraph.laid
    );
    assert!(
        paragraph.laid.height() > label.laid.height() + type_scale::LEADING - type_scale::BODY,
        "prose {:?} over label {:?}",
        paragraph.laid,
        label.laid
    );
}
