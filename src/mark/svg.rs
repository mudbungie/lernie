//! **The vector emitter**: the walk's shape list as one SVG element each, in
//! the walk's own order, on a canvas of [`SPAN`] units a side.
//!
//! Nothing here re-decides a shape. A trace is the walk's own sampled points
//! under the SVG stroker with round caps and joins — which is what the phone's
//! drawable and the window's egui stroker ask of theirs — the lune is the
//! walk's own ribs closed into one path, a disc is a circle, and the hues are
//! the walk's own bytes.
//!
//! **Every number is written at a fixed precision** ([`place`]), because the
//! checked-in file is pinned byte for byte and a float printed at its full
//! length would carry a target's last-bit arithmetic into the test. At one
//! decimal of a thousandth the file holds the picture to a ten-thousandth of
//! its side, which no renderer resolves, and a last-bit difference rounds away.

use super::{Hue, Point, Shape};

/// The canvas, in the units every coordinate below is written in.
const SPAN: f32 = 1000.0;

/// The whole document: the canvas, one element per shape, the close.
pub(super) fn document(shapes: &[Shape]) -> String {
    let mut out = format!(
        "<svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 {SPAN:.0} {SPAN:.0}\" \
         width=\"{SPAN:.0}\" height=\"{SPAN:.0}\">\n"
    );
    for shape in shapes {
        out.push_str(&element(shape));
        out.push('\n');
    }
    out.push_str("</svg>\n");
    out
}

/// The SVG element one shape is.
fn element(shape: &Shape) -> String {
    match shape {
        Shape::Trace { path, width, fill } => format!(
            "  <path d=\"{}\" fill=\"none\" stroke=\"{}\" stroke-width=\"{}\" \
             stroke-linecap=\"round\" stroke-linejoin=\"round\"/>",
            polyline(path),
            hex(*fill),
            place(*width)
        ),
        Shape::Lune { ribs, fill } => format!(
            "  <path d=\"{} Z\" fill=\"{}\"/>",
            polyline(&Shape::outline(ribs)),
            hex(*fill)
        ),
        Shape::Disc {
            cx,
            cy,
            radius,
            fill,
        } => format!(
            "  <circle cx=\"{}\" cy=\"{}\" r=\"{}\" fill=\"{}\"/>",
            place(*cx),
            place(*cy),
            place(*radius),
            hex(*fill)
        ),
    }
}

/// The walk's points as a move-and-lines path.
fn polyline(path: &[Point]) -> String {
    path.iter()
        .enumerate()
        .map(|(index, (x, y))| {
            let verb = if index == 0 { 'M' } else { 'L' };
            format!("{verb}{},{}", place(*x), place(*y))
        })
        .collect::<Vec<String>>()
        .join(" ")
}

/// A unit-square length on the canvas, at the one precision the file holds.
fn place(unit: f32) -> String {
    format!("{:.1}", unit * SPAN)
}

/// A hue as the six-digit spelling the vector source wants.
fn hex(fill: Hue) -> String {
    let [red, green, blue] = fill;
    format!("#{red:02x}{green:02x}{blue:02x}")
}
