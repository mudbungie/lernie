//! **The pixel loop**: supersampling, and nothing else.
//!
//! There is no compositing model here and no coverage curve. Each pixel is cut
//! into a [`SUB`]×[`SUB`] grid and every subsample asks the shape list one
//! question — *which of you, if any, is on top of me* — which is the painter's
//! order read from the back. The pixel is then the average of what its
//! subsamples saw.
//!
//! **That is exact rather than approximate**, and it is why the seam between
//! two touching shapes is seamless: a per-shape coverage composite blends each
//! edge against what is under it and leaves a lighter line where two edges
//! meet, because both are partly transparent at the same subpixel. Here nothing
//! is partly anything — a subsample belongs to exactly one shape or to none.
//!
//! And it keeps the whole file in integers ([`super`] on why that matters):
//! coverage is a count, alpha is that count scaled, and a channel is a mean of
//! bytes.

use super::Ink;
use super::shape::{SPAN, Shape};

/// Subsamples per axis. The square of this is how many alpha levels an edge can
/// take, and 64 is where a 16 px edge stops looking stepped.
const SUB: u32 = 8;
/// How many subsamples a pixel is cut into.
const SAMPLES: u32 = SUB * SUB;
/// The value a full byte holds, as the arithmetic below needs it.
const FULL: u32 = 255;

/// `shapes` rasterized `size`×`size` as straight-alpha RGBA8, row-major from
/// the top-left.
pub(super) fn rgba(shapes: &[Shape], size: u16) -> Vec<u8> {
    let edge = i64::from(size);
    // The sample denominator: a subsample's canvas coordinate is its numerator
    // below over this, and `Shape::holds` multiplies the geometry up to meet it
    // rather than dividing the sample down.
    let den = 2 * edge * i64::from(SUB);
    let mut out = Vec::with_capacity(usize::from(size) * usize::from(size) * 4);
    for y in 0..edge {
        for x in 0..edge {
            out.extend_from_slice(&pixel(shapes, x, y, den));
        }
    }
    out
}

/// What one pixel's subsamples saw.
#[derive(Default)]
struct Seen {
    red: u32,
    green: u32,
    blue: u32,
    hits: u32,
}

impl Seen {
    /// Take one subsample.
    fn take(&mut self, ink: Ink) {
        self.red += u32::from(ink.red);
        self.green += u32::from(ink.green);
        self.blue += u32::from(ink.blue);
        self.hits += 1;
    }

    /// The pixel: each channel the mean of what covered it, and alpha the share
    /// of it that was covered at all. Both roundings are half-up, written as
    /// the division they are.
    fn pixel(&self) -> [u8; 4] {
        if self.hits == 0 {
            return [0; 4];
        }
        let mean = |total: u32| byte((total + self.hits / 2) / self.hits);
        [
            mean(self.red),
            mean(self.green),
            mean(self.blue),
            byte((self.hits * FULL + SAMPLES / 2) / SAMPLES),
        ]
    }
}

/// One pixel of the mark.
fn pixel(shapes: &[Shape], x: i64, y: i64, den: i64) -> [u8; 4] {
    let mut seen = Seen::default();
    for sy in 0..SUB {
        for sx in 0..SUB {
            if let Some(ink) = topmost(shapes, at(x, sx), at(y, sy), den) {
                seen.take(ink);
            }
        }
    }
    seen.pixel()
}

/// One subsample's canvas coordinate, as a numerator over the denominator
/// [`rgba`] fixed: the centre of subcell `step` inside pixel `whole`.
fn at(whole: i64, step: u32) -> i64 {
    SPAN * (2 * (whole * i64::from(SUB) + i64::from(step)) + 1)
}

/// **The last shape covering this point**, which is the one on top. `None` is
/// the canvas, and the canvas is transparent.
fn topmost(shapes: &[Shape], x: i64, y: i64, den: i64) -> Option<Ink> {
    shapes
        .iter()
        .rev()
        .find(|shape| shape.holds(x, y, den))
        .map(Shape::ink)
}

/// A value already inside a byte's range, as a byte.
///
/// The clamp **is** the conversion rather than a check on it: every caller
/// above divides a sum of bytes by the count that produced it, or scales a
/// count by its own maximum, so the value cannot exceed 255 — and a total
/// function that saturates says so better than an arm no test can reach.
fn byte(value: u32) -> u8 {
    u8::try_from(value.min(FULL)).unwrap_or(u8::MAX)
}
