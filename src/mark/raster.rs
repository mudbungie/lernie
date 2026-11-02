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
//! The geometry is the walk's `f32`, and the only integers are the counts:
//! coverage is how many subsamples hit, alpha is that count scaled, and a
//! channel is a mean of bytes — so nothing is ever narrowed from a float
//! ([`super`] on why that matters). Each shape is read once into a
//! [`Figure`], which carries the bounds a sample is tested against before the
//! shape's own arithmetic runs: a trace is a few dozen segments, and the box
//! around each is what keeps a 256 px icon a fraction of a second rather than
//! several.

use super::{Hue, Point, Shape};

/// Subsamples per axis. The square of this is how many alpha levels an edge
/// can take; the toolkit's raster is asked for at 256 px and scaled down by
/// the shell, which is where the rest of the antialiasing happens.
const SUB: u8 = 4;
/// How many subsamples a pixel is cut into — [`SUB`] squared, which `tests`
/// holds it to.
const SAMPLES: u32 = 16;
/// The value a full byte holds, as the arithmetic below needs it.
const FULL: u32 = 255;

/// `shapes` rasterized `size`×`size` as straight-alpha RGBA8, row-major from
/// the top-left.
pub(super) fn rgba(shapes: &[Shape], size: u16) -> Vec<u8> {
    let figures: Vec<Figure> = shapes.iter().map(Figure::of).collect();
    let edge = f32::from(size);
    let mut out = Vec::with_capacity(usize::from(size) * usize::from(size) * 4);
    for row in 0..size {
        for column in 0..size {
            out.extend_from_slice(&pixel(&figures, column, row, edge));
        }
    }
    out
}

/// An axis-aligned box, which is the cheap question asked before the real one.
#[derive(Clone, Copy)]
struct Bounds {
    left: f32,
    top: f32,
    right: f32,
    bottom: f32,
}

impl Bounds {
    /// The box around `points`, grown by `pad` on every side. Around nothing
    /// it is inverted, and so holds nothing — which is the right answer for a
    /// path with no segments, without a case for it.
    fn around(points: impl Iterator<Item = Point>, pad: f32) -> Self {
        let mut held = Self {
            left: f32::INFINITY,
            top: f32::INFINITY,
            right: f32::NEG_INFINITY,
            bottom: f32::NEG_INFINITY,
        };
        for (x, y) in points {
            held.left = held.left.min(x - pad);
            held.top = held.top.min(y - pad);
            held.right = held.right.max(x + pad);
            held.bottom = held.bottom.max(y + pad);
        }
        held
    }

    fn holds(self, (x, y): Point) -> bool {
        x >= self.left && x <= self.right && y >= self.top && y <= self.bottom
    }
}

/// One span of a trace's centreline, with the box its band fits in.
struct Segment {
    from: Point,
    to: Point,
    bounds: Bounds,
}

/// A shape read once for the loop: the same geometry with its bounds beside it.
enum Figure {
    Disc {
        centre: Point,
        radius: f32,
        fill: Hue,
    },
    Band {
        segments: Vec<Segment>,
        half: f32,
        bounds: Bounds,
        fill: Hue,
    },
    Lens {
        outline: Vec<Point>,
        bounds: Bounds,
        fill: Hue,
    },
}

impl Figure {
    fn of(shape: &Shape) -> Self {
        match shape {
            Shape::Disc {
                cx,
                cy,
                radius,
                fill,
            } => Self::Disc {
                centre: (*cx, *cy),
                radius: *radius,
                fill: *fill,
            },
            Shape::Trace { path, width, fill } => {
                let half = width / 2.0;
                Self::Band {
                    segments: path
                        .windows(2)
                        .map(|pair| Segment {
                            from: pair.first().copied().unwrap_or_default(),
                            to: pair.last().copied().unwrap_or_default(),
                            bounds: Bounds::around(pair.iter().copied(), half),
                        })
                        .collect(),
                    half,
                    bounds: Bounds::around(path.iter().copied(), half),
                    fill: *fill,
                }
            }
            Shape::Lune { ribs, fill } => {
                let outline = Shape::outline(ribs);
                Self::Lens {
                    bounds: Bounds::around(outline.iter().copied(), 0.0),
                    outline,
                    fill: *fill,
                }
            }
        }
    }

    fn fill(&self) -> Hue {
        match self {
            Self::Disc { fill, .. } | Self::Band { fill, .. } | Self::Lens { fill, .. } => *fill,
        }
    }

    /// **Is the point inside?** A disc by its radius; a band wherever the
    /// point is within half the width of any segment, round caps and joins
    /// included, which is exactly what a stroker with round caps paints; a
    /// lens by the even-odd rule over its outline.
    fn holds(&self, point: Point) -> bool {
        match self {
            Self::Disc { centre, radius, .. } => {
                let (dx, dy) = (point.0 - centre.0, point.1 - centre.1);
                dx.mul_add(dx, dy * dy) <= radius * radius
            }
            Self::Band {
                segments,
                half,
                bounds,
                ..
            } => {
                bounds.holds(point)
                    && segments.iter().any(|segment| {
                        segment.bounds.holds(point)
                            && gap2(segment.from, segment.to, point) <= half * half
                    })
            }
            Self::Lens {
                outline, bounds, ..
            } => bounds.holds(point) && crossings(outline, point) % 2 == 1,
        }
    }
}

/// The squared distance from `point` to the segment `from`–`to`. A segment of
/// no length has no direction to project onto and answers `NaN`, which no
/// comparison accepts — so it covers nothing, which is what nothing covers.
/// The walk's arcs never produce one; this is what happens rather than a case.
fn gap2(from: Point, to: Point, point: Point) -> f32 {
    let (ux, uy) = (to.0 - from.0, to.1 - from.1);
    let (px, py) = (point.0 - from.0, point.1 - from.1);
    let span = ux.mul_add(ux, uy * uy);
    let along = (px.mul_add(ux, py * uy) / span).clamp(0.0, 1.0);
    let (dx, dy) = (along.mul_add(-ux, px), along.mul_add(-uy, py));
    dx.mul_add(dx, dy * dy)
}

/// How many edges of the closed `outline` a ray from `point` to the right
/// crosses — odd means inside.
fn crossings(outline: &[Point], (x, y): Point) -> u32 {
    let mut count = 0;
    for (from, to) in outline.iter().zip(outline.iter().cycle().skip(1)) {
        if (from.1 > y) != (to.1 > y) {
            let reach = (y - from.1) / (to.1 - from.1);
            if x < reach.mul_add(to.0 - from.0, from.0) {
                count += 1;
            }
        }
    }
    count
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
    fn take(&mut self, fill: Hue) {
        self.red += u32::from(fill[0]);
        self.green += u32::from(fill[1]);
        self.blue += u32::from(fill[2]);
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
fn pixel(figures: &[Figure], column: u16, row: u16, edge: f32) -> [u8; 4] {
    let mut seen = Seen::default();
    for down in 0..SUB {
        for across in 0..SUB {
            let point = (sample(column, across, edge), sample(row, down, edge));
            if let Some(fill) = topmost(figures, point) {
                seen.take(fill);
            }
        }
    }
    seen.pixel()
}

/// One subsample's unit-square coordinate: the centre of subcell `step`
/// inside pixel `whole`, on a canvas `edge` pixels across.
fn sample(whole: u16, step: u8, edge: f32) -> f32 {
    (f32::from(step) + 0.5).mul_add(1.0 / f32::from(SUB), f32::from(whole)) / edge
}

/// **The last figure covering this point**, which is the one on top. `None` is
/// the canvas, and the canvas is transparent.
fn topmost(figures: &[Figure], point: Point) -> Option<Hue> {
    figures
        .iter()
        .rev()
        .find(|figure| figure.holds(point))
        .map(Figure::fill)
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

#[cfg(test)]
pub(super) const fn samples() -> (u8, u32) {
    (SUB, SAMPLES)
}
