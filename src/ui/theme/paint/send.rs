//! **The send door** (DESIGN §4.48 item 4): a round seat [`ROW`] across at
//! the box's trailing end, the prompt's glyph on the brand — the baseline's
//! round send button, and the pointer's spelling of the Enter the field
//! already reads. It is a door like every other ([`super::dress`]): named
//! in the tree, washed under the pointer, ringed while it holds the
//! keyboard, and told.

use egui::{Align2, FontId, Response, Sense, vec2};

use super::told;
use crate::ui::theme::{BRAND, GROUND, INK_FAINT, ROW, glyph};

/// What the send door answers to in the tree.
pub const SEND: &str = "send";

/// **The round send door**, `live` while there is something to send — the
/// brand under the glyph — and in faint ink at rest, so an empty box's door
/// says it would send nothing.
pub fn send(ui: &mut egui::Ui, live: bool, help: &str) -> Response {
    let (rect, seat) = ui.allocate_exact_size(vec2(ROW, ROW), Sense::click());
    super::dress(ui, &seat, rect, SEND);
    let seat_ink = if live { BRAND } else { INK_FAINT };
    let painter = ui.painter();
    painter.circle_filled(rect.center(), ROW / 2.0, seat_ink);
    let font = FontId::proportional(ROW * 0.75);
    painter.text(
        rect.center(),
        Align2::CENTER_CENTER,
        glyph::PROMPT,
        font,
        GROUND,
    );
    told(seat.on_hover_cursor(egui::CursorIcon::PointingHand), help)
}
