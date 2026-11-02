//! **The seat's mark as a door** (DESIGN §4.44 item 2; bl-036a): the walk of
//! `crate::mark` said in egui's words, in a [`ROW`]-square seat that is a
//! door like every other — named in the tree, washed under the pointer,
//! ringed while it holds the keyboard, and told.
//!
//! **Nothing here re-derives an edge or re-decides a hue.** The walk is pure
//! and pinned by its own suite; this file only chooses egui's primitive for
//! each shape — a trace goes out as its centreline and width so egui's own
//! stroker does the joins, the lune as one convex polygon, a disc as a
//! circle — exactly as the phone seat says the same list.

use egui::{Response, Sense, vec2};

use super::told;
use crate::mark::Shape;
use crate::ui::theme::{ROW, hue};

/// What the mark answers to in the tree — the one control with no words.
pub const MARK: &str = "mark";

/// **The mark, [`ROW`] across, as a door** whose `help` says where it leads.
pub fn mark(ui: &mut egui::Ui, help: &str) -> Response {
    let (rect, seat) = ui.allocate_exact_size(vec2(ROW, ROW), Sense::click());
    super::dress(ui, &seat, rect, MARK);
    paint(ui.painter(), rect);
    told(seat.on_hover_cursor(egui::CursorIcon::PointingHand), help)
}

/// Paint the walk into `rect`. The unit square it works in maps to the
/// largest centred square `rect` holds — the mark is round, and a stretched
/// one is a different picture.
fn paint(painter: &egui::Painter, rect: egui::Rect) {
    let side = rect.width().min(rect.height());
    let origin = rect.center() - vec2(side, side) / 2.0;
    let at = |(x, y): crate::mark::Point| origin + vec2(x * side, y * side);
    for shape in crate::mark::mark() {
        painter.add(match &shape {
            Shape::Trace { path, width, fill } => egui::Shape::line(
                path.iter().map(|point| at(*point)).collect(),
                egui::Stroke::new(width * side, hue(*fill)),
            ),
            Shape::Lune { ribs, fill } => egui::Shape::convex_polygon(
                Shape::outline(ribs).into_iter().map(at).collect(),
                hue(*fill),
                egui::Stroke::NONE,
            ),
            Shape::Disc {
                cx,
                cy,
                radius,
                fill,
            } => egui::Shape::circle_filled(at((*cx, *cy)), radius * side, hue(*fill)),
        });
    }
}
