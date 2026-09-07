//! **The fields that take text** (`docs/STYLE.md` §5): the one-line box every
//! covering pane spends, and the composer — the conversation pane's focal
//! element, with the send inside it (bl-f251).
//!
//! Split from [`super`] at the design-time budget on the seam the two shapes
//! already have: a row, a rule and a section are paint that takes no keyboard,
//! and these two are the paint that does — the ring while they hold the caret
//! is the one stroke the language leaves on the glass, and both spell it here.

use egui::{Color32, Key, KeyboardShortcut, Modifiers, Response, Stroke};

use super::super::{BRAND, COMPOSER_ROWS, RADIUS, SURFACE, State, space, tint};

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
) -> Response {
    frame(ui, id, glow, space::XS)
        .show(ui, |ui| {
            ui.add(
                egui::TextEdit::singleline(text)
                    .id(id)
                    .frame(false)
                    .desired_width(f32::INFINITY)
                    .hint_text(hint),
            )
        })
        .inner
}

/// **The composer**: [`COMPOSER_ROWS`] lines tall, the send inside it at the
/// bottom right in the brand on a brand wash, and the glow where the field
/// alone used to carry it. Enter is the caller's to read — the field breaks a
/// line on **Shift+Enter** and consumes nothing else — so the one key that
/// sends is decided beside the act it fires, not here.
///
/// **The height is allocated, never left to the layout.** A bottom panel
/// remembers the rect its content took and lays the next frame out in it;
/// a bottom-aligned layout given the whole of that rect places its items at
/// its foot and reports the rect whole, so the panel grew by one row every
/// frame, forever, and the transcript's tail walked off the glass under it.
/// Bounding the child to the rows it asked for makes the height a fact of
/// the content, and a draft longer than the rows still grows it — from the
/// words, which is the one direction growth may come from.
///
/// The two responses are the box and the send, in that order: the caller
/// reads focus off the first and the click off the second.
pub fn composer(
    ui: &mut egui::Ui,
    id: egui::Id,
    text: &mut String,
    hint: &str,
    glow: Option<Color32>,
    send: &str,
) -> (Response, Response) {
    let height = f32::from(COMPOSER_ROWS) * ui.text_style_height(&egui::TextStyle::Body);
    frame(ui, id, glow, space::S)
        .show(ui, |ui| {
            ui.allocate_ui_with_layout(
                egui::vec2(ui.available_width(), height),
                egui::Layout::right_to_left(egui::Align::BOTTOM),
                |ui| {
                    let deposit = ui.add(
                        egui::Button::new(egui::RichText::new(send).color(BRAND))
                            .fill(tint(State::Working)),
                    );
                    let entry = ui.add(
                        egui::TextEdit::multiline(text)
                            .id(id)
                            .frame(false)
                            .desired_rows(usize::from(COMPOSER_ROWS))
                            .desired_width(ui.available_width())
                            .hint_text(hint)
                            .return_key(KeyboardShortcut::new(Modifiers::SHIFT, Key::Enter)),
                    );
                    (entry, deposit)
                },
            )
            .inner
        })
        .inner
}

/// The box both fields stand in: the glow or `SURFACE`, the ring while `id`
/// holds the keyboard, and `down` of padding above and below the words.
fn frame(ui: &egui::Ui, id: egui::Id, glow: Option<Color32>, down: f32) -> egui::Frame {
    let focused = ui.memory(|memory| memory.has_focus(id));
    let ring = if focused {
        Stroke::new(1.0, BRAND)
    } else {
        Stroke::NONE
    };
    egui::Frame::none()
        .fill(glow.unwrap_or(SURFACE))
        .stroke(ring)
        .rounding(RADIUS)
        .inner_margin(egui::Margin::symmetric(space::S, down))
}
