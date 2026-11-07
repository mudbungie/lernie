//! **The fields that take text** (`docs/STYLE.md` §5): the one-line box every
//! covering pane spends, and the composer — the conversation pane's focal
//! element, with the send inside it (bl-f251).
//!
//! Split from [`super`] at the design-time budget on the seam the two shapes
//! already have: a row, a rule and a section are paint that takes no keyboard,
//! and these two are the paint that does — the ring while they hold the caret
//! is the one stroke the language leaves on the glass, and both spell it here.

use egui::{Color32, Key, KeyboardShortcut, Modifiers, Response, Stroke};

use super::super::{BRAND, HAIRLINE, RADIUS, SURFACE, space};

/// **A field that takes text**: `SURFACE` with no stroke at rest, the brand
/// ring while it holds the caret, and `glow`'s tint under it where the pane
/// has something to say about the moment. The box wears `id`, which is what
/// the keyboard's gate compares against (`crate::ui::keys`).
pub fn field(
    ui: &mut egui::Ui,
    id: egui::Id,
    text: &mut String,
    hint: &str,
    glow: Option<Color32>,
    help: &str,
) -> Response {
    let seat = frame(ui, id, glow, space::XS)
        .show(ui, |ui| {
            ui.add(
                egui::TextEdit::singleline(text)
                    .id(id)
                    .frame(false)
                    .desired_width(f32::INFINITY)
                    .hint_text(hint),
            )
        })
        .inner;
    super::told(seat, help)
}

/// **The prompt line** (`docs/STYLE.md` §5): the glyph in the brand at the
/// left edge, the draft beside it a line per line typed, the hint in weak ink
/// while it is empty; `SURFACE` under it inside a hairline at `RADIUS` with
/// `S` of air (DESIGN §4.48 item 4), the brand ring while it holds the
/// caret, the tint handed in while the conversation asks. As tall as its
/// lines and never taller. **The field is the control**: Enter is read
/// beside it by the composer, and what stands at its trailing end — the
/// pills and the send door — is `trailing`, laid after the field in the
/// width it names, so the Tab order is the reading order.
pub fn prompt(
    ui: &mut egui::Ui,
    id: egui::Id,
    text: &mut String,
    hint: &str,
    glow: Option<Color32>,
    help: &str,
    (reserve, trailing): (f32, impl FnOnce(&mut egui::Ui)),
) -> Response {
    // **A slash line keeps the caret through Tab**: the menu over the box reads
    // Tab as *complete the word* (`crate::ui::slash`), and a Tab that walked
    // the focus away would complete it into nothing.
    let lock = text.starts_with(crate::ui::slash::LEAD);
    let seat = frame(ui, id, glow, space::S)
        .stroke(Stroke::new(1.0, ring(ui, id).unwrap_or(HAIRLINE)))
        .show(ui, |ui| {
            // **The trailing end shares the field's line while the field
            // keeps a third of the box**, and stands on its own line under
            // it, at the right end, where it would not: a box narrower than
            // its controls still lays every one of them on the glass.
            let room = ui.available_width();
            let beside = room - reserve >= room / 3.0;
            let mut trailing = Some(trailing);
            let field = ui
                .horizontal_top(|ui| {
                    ui.label(egui::RichText::new(super::super::glyph::PROMPT).color(BRAND));
                    let leave = if beside { reserve } else { 0.0 };
                    let field = ui.add(
                        egui::TextEdit::multiline(text)
                            .id(id)
                            .frame(false)
                            .desired_rows(1)
                            .margin(MARGIN)
                            .desired_width(ui.available_width() - leave - MARGIN.sum().x)
                            .hint_text(hint)
                            .lock_focus(lock)
                            .return_key(KeyboardShortcut::new(Modifiers::SHIFT, Key::Enter)),
                    );
                    if let Some(trailing) = trailing.take_if(|_| beside) {
                        trailing(ui);
                    }
                    field
                })
                .inner;
            if let Some(trailing) = trailing {
                ui.horizontal_wrapped(|ui| {
                    ui.add_space((ui.available_width() - reserve).max(0.0));
                    trailing(ui);
                });
            }
            field
        })
        .inner;
    super::told(seat, help)
}

/// The air egui lays around a text edit's words, named so the width the
/// trailing end leaves can count it.
const MARGIN: egui::Margin = egui::Margin::symmetric(space::XS, space::XXS);

/// The brand, while the field `id` holds the caret.
fn ring(ui: &egui::Ui, id: egui::Id) -> Option<Color32> {
    ui.memory(|memory| memory.has_focus(id)).then_some(BRAND)
}

fn frame(ui: &egui::Ui, id: egui::Id, glow: Option<Color32>, down: f32) -> egui::Frame {
    let ring = ring(ui, id).map_or(Stroke::NONE, |ink| Stroke::new(1.0, ink));
    egui::Frame::none()
        .fill(glow.unwrap_or(SURFACE))
        .stroke(ring)
        .rounding(RADIUS)
        .inner_margin(egui::Margin::symmetric(space::S, down))
}
