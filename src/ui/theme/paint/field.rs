//! **The fields that take text** (`docs/STYLE.md` §5): the one-line box every
//! covering pane spends, and the composer — the conversation pane's focal
//! element, with the send inside it (bl-f251).
//!
//! Split from [`super`] at the design-time budget on the seam the two shapes
//! already have: a row, a rule and a section are paint that takes no keyboard,
//! and these two are the paint that does — the ring while they hold the caret
//! is the one stroke the language leaves on the glass, and both spell it here.

use egui::{Color32, Key, KeyboardShortcut, Modifiers, Response, Stroke};

use super::super::{BRAND, RADIUS, SURFACE, space};

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

/// **The composer**: `rows` lines tall — [`COMPOSER_ROWS`](super::super::COMPOSER_ROWS)
/// until the operator
/// drags the panel's top edge (DESIGN §4.39) — the act inside it at the
/// bottom right in the brand on a brand wash, and the glow where the field
/// alone used to carry it. Enter is the caller's to read — the field breaks a
/// line on **Shift+Enter** and consumes nothing else — so the one key that
/// fires is decided beside the act, not here.
///
/// **`send` is a parameter because the box has two modes and not two shapes**
/// (DESIGN §4.39): the deposit words it `send` and the start words it
/// `start`, and the word is the whole of what differs between them — which is
/// what makes *one composer* an assertion a test can read off the glass as a
/// height rather than a claim in prose.
///
/// **The height is HANDED to the field and never read back off it.** A bottom
/// panel
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
/// **The prompt line** (`docs/STYLE.md` §5): the glyph in the brand at the
/// left edge, the draft beside it a line per line typed, the hint in weak ink
/// while it is empty; `SURFACE` under it, the brand ring while it holds the
/// caret, the tint handed in while the conversation asks. As tall as its
/// lines and never taller. **The field is the control**: there is nothing
/// inside it to click, and Enter is read beside it by the composer.
pub fn prompt(
    ui: &mut egui::Ui,
    id: egui::Id,
    text: &mut String,
    hint: &str,
    glow: Option<Color32>,
) -> Response {
    // **A slash line keeps the caret through Tab**: the menu over the box reads
    // Tab as *complete the word* (`crate::ui::slash`), and a Tab that walked
    // the focus away would complete it into nothing.
    let lock = text.starts_with(crate::ui::slash::LEAD);
    frame(ui, id, glow, space::S)
        .show(ui, |ui| {
            ui.horizontal_top(|ui| {
                ui.label(egui::RichText::new(super::super::glyph::PROMPT).color(BRAND));
                ui.add(
                    egui::TextEdit::multiline(text)
                        .id(id)
                        .frame(false)
                        .desired_rows(1)
                        .desired_width(ui.available_width())
                        .hint_text(hint)
                        .lock_focus(lock)
                        .return_key(KeyboardShortcut::new(Modifiers::SHIFT, Key::Enter)),
                )
            })
            .inner
        })
        .inner
}

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
