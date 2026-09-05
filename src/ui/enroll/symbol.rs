//! **The symbol as geometry**: how big a module is, where it goes, and what it
//! is painted with.
//!
//! Split from the pane (bl-5e0e) on the seam the pane already had: everything
//! above is about what an operator can say, everything here is about what a
//! camera can read, and the two fail for different reasons.
//!
//! **Pixels are the wrong altitude to be right at, except here.** A symbol
//! drawn at the wrong scale still carries the same bytes, so the pane's own
//! suite asserts the module matrix and leaves the picture alone. What this
//! module decides is the one thing that IS about pixels: the grid a decoder
//! reads the matrix off, which is why its suite rasterizes and reads every
//! module back.

use crate::qr::Symbol;
use crate::ui::theme;

/// **The smallest module the symbol may be drawn at**, in device pixels. One
/// pixel is a floor rather than a preference: below it two modules share a
/// pixel and the grid stops being one. A pane with less room than that gets a
/// symbol that overflows it, which is legible when scrolled to; a symbol shrunk
/// to fit would be a picture of a symbol.
const FLOOR: f32 = 1.0;
/// The quiet zone, in modules — the standard's four, which a decoder uses to
/// find the symbol's edge.
const QUIET: usize = 4;

/// **The symbol, as one mesh on a painted ground**, drawn as large as the pane
/// allows and on a whole-pixel grid. The quiet zone is painted rather than
/// assumed: a decoder uses it to find the symbol's edge, and a pane's own
/// background is not it.
pub(super) fn paint(ui: &mut egui::Ui, symbol: &Symbol) {
    let across = symbol.side() + 2 * QUIET;
    let pitch = pitch(
        ui.available_width().min(ui.available_height()),
        across,
        ui.ctx().pixels_per_point(),
    );
    let side = span(across, pitch);
    let (rect, _) = ui.allocate_exact_size(egui::vec2(side, side), egui::Sense::hover());
    ui.painter().rect_filled(rect, 0.0, theme::PAPER);
    ui.painter().add(dark(symbol, rect, pitch));
}

/// **The module pitch: a whole number of device pixels, and as many of them as
/// the pane has room for** (bl-5e0e).
///
/// It replaced a 4-point constant and both halves of that constant were wrong,
/// each for a measured reason.
///
/// *As many as there is room for*, because the symbol's whole job is to be read
/// by a camera once before the material is forgotten, and the pane offered no
/// way to make it bigger: a realistic REMOTE §8.4 envelope picks a 153-module
/// symbol, which at four points a module is a 644-point square standing in the
/// central panel — wider than the whole window in two of the three sizes this
/// seat is photographed at.
///
/// *Whole device pixels*, because a fractional pitch puts a module boundary
/// inside a pixel, and neighbouring modules of the same nominal size then come
/// out different widths — a distorted grid, which is the one thing a decoder
/// cannot correct for. The symbol's ORIGIN needs no such rounding: at a whole
/// pixel pitch every module covers the same number of pixel centres wherever
/// the grid starts, so snapping it would buy nothing.
///
/// Room is measured in points and the answer is given in points, with the
/// device between them — which is why the scale is a parameter rather than a
/// constant: the bleed the mesh below dissolves is worst at one pixel per
/// point, and that is the display this seat is most likely to be read from.
fn pitch(room: f32, across: usize, per_point: f32) -> f32 {
    (room * per_point / count(across)).floor().max(FLOOR) / per_point
}

/// **Every dark module in ONE mesh**, which is where the bleed went (bl-5e0e).
///
/// egui feathers every filled shape by a device pixel, half of it proud of the
/// edge, so a module drawn as its own rectangle is painted half a pixel over
/// each neighbour: on the old four-point pitch at one pixel per point a light
/// module enclosed by dark ones lost about 44% of its area, and the loss is
/// worst exactly where module pixels are scarcest — which is where a camera
/// needs them most. A `Mesh` is emitted by the tessellator verbatim: no
/// feathering, no ring of extra vertices around each module, and one shape
/// rather than one per module.
fn dark(symbol: &Symbol, rect: egui::Rect, pitch: f32) -> egui::Shape {
    let mut mesh = egui::Mesh::default();
    for y in 0..symbol.side() {
        for x in 0..symbol.side() {
            if symbol.dark(x, y) {
                mesh.add_colored_rect(module(rect, x, y, pitch), theme::INK);
            }
        }
    }
    egui::Shape::mesh(mesh)
}

/// Where one module sits inside the symbol's rectangle, quiet zone included.
fn module(rect: egui::Rect, x: usize, y: usize, pitch: f32) -> egui::Rect {
    egui::Rect::from_min_size(
        rect.min + egui::vec2(span(x + QUIET, pitch), span(y + QUIET, pitch)),
        egui::vec2(pitch, pitch),
    )
}

/// `n` modules, in screen points, at a pitch [`pitch`] chose.
fn span(n: usize, pitch: f32) -> f32 {
    count(n) * pitch
}

/// `n` modules as a number. Through `u16` rather than by a cast: the house lint
/// set denies a lossy numeric cast and its only home would be a manifest-wide
/// relaxation, and the widest symbol there is measures 185 modules with its
/// quiet zone — so the conversion is exact and the saturation is unreachable
/// by arithmetic rather than by hope.
fn count(n: usize) -> f32 {
    f32::from(u16::try_from(n).unwrap_or(u16::MAX))
}

#[cfg(test)]
mod tests;
