//! **The three primitives the mark is made of**, each answering the same two
//! questions: is this point inside me, and what element am I in SVG.
//!
//! Two answers, one geometry, and the geometry is stated once. A shape that
//! rasterized one way and vectorized another would be two pictures wearing one
//! name, and nothing in the tree could say which was the mark.
//!
//! **Everything is an integer**, in thousandths of the canvas — see [`super`]
//! for why that is the whole design and not an implementation detail.

use super::Ink;

/// The canvas, in the units every coordinate below is written in.
pub(super) const SPAN: i64 = 1000;

/// One flat-filled figure.
///
/// Three, because three is what the mark has: the engine held elsewhere, the
/// seat that is here, and the one wire between them. A fourth arrives with a
/// mark that needs it.
pub(super) enum Shape {
    /// A filled circle.
    Disc { cx: i64, cy: i64, r: i64, ink: Ink },
    /// An annulus: a circle of radius `r` drawn `half` thick either side.
    Ring {
        cx: i64,
        cy: i64,
        r: i64,
        half: i64,
        ink: Ink,
    },
    /// A capsule: everything within `half` of the segment `a`–`b`, round caps
    /// included, which is exactly what an SVG round-capped line paints.
    Bar {
        ax: i64,
        ay: i64,
        bx: i64,
        by: i64,
        half: i64,
        ink: Ink,
    },
}

impl Shape {
    /// The colour this shape lays down.
    pub(super) fn ink(&self) -> Ink {
        match self {
            Self::Disc { ink, .. } | Self::Ring { ink, .. } | Self::Bar { ink, .. } => *ink,
        }
    }

    /// **Is the point `(x, y)` inside?** The point is given as a numerator over
    /// `den`, so a sample taken anywhere on any subpixel grid is compared
    /// against the same geometry with no rounding on either side: every
    /// coordinate below is multiplied up to the sample's denominator rather
    /// than the sample being divided down to the geometry's.
    pub(super) fn holds(&self, x: i64, y: i64, den: i64) -> bool {
        match self {
            Self::Disc { cx, cy, r, .. } => square(x - cx * den, y - cy * den) <= (r * den).pow(2),
            Self::Ring {
                cx, cy, r, half, ..
            } => {
                let from = square(x - cx * den, y - cy * den);
                from <= ((r + half) * den).pow(2) && from >= ((r - half) * den).pow(2)
            }
            Self::Bar {
                ax,
                ay,
                bx,
                by,
                half,
                ..
            } => {
                let (ax, ay) = (x - ax * den, y - ay * den);
                let (bx, by) = (x - bx * den, y - by * den);
                let (ux, uy) = (ax - bx, ay - by);
                let along = ax * ux + ay * uy;
                let span = square(ux, uy);
                let reach = (half * den).pow(2);
                // Past either end the nearest point is that end, so the cap is
                // the same disc test. Between them the perpendicular distance
                // is `|a|² − along²/span`, and multiplying the comparison
                // through by `span` keeps it exact — the one product that can
                // outgrow an `i64`, which is why it is taken in `i128`.
                if along <= 0 {
                    square(ax, ay) <= reach
                } else if along >= span {
                    square(bx, by) <= reach
                } else {
                    i128::from(square(ax, ay) - reach) * i128::from(span)
                        <= i128::from(along).pow(2)
                }
            }
        }
    }

    /// The SVG element this shape is, at the canvas's own scale.
    pub(super) fn element(&self) -> String {
        match self {
            Self::Disc { cx, cy, r, ink } => format!(
                "  <circle cx=\"{cx}\" cy=\"{cy}\" r=\"{r}\" fill=\"{}\"/>",
                ink.hex()
            ),
            Self::Ring {
                cx,
                cy,
                r,
                half,
                ink,
            } => format!(
                "  <circle cx=\"{cx}\" cy=\"{cy}\" r=\"{r}\" fill=\"none\" \
                 stroke=\"{}\" stroke-width=\"{}\"/>",
                ink.hex(),
                half * 2
            ),
            Self::Bar {
                ax,
                ay,
                bx,
                by,
                half,
                ink,
            } => format!(
                "  <line x1=\"{ax}\" y1=\"{ay}\" x2=\"{bx}\" y2=\"{by}\" \
                 stroke=\"{}\" stroke-width=\"{}\" stroke-linecap=\"round\"/>",
                ink.hex(),
                half * 2
            ),
        }
    }
}

/// The squared length of `(x, y)`.
fn square(x: i64, y: i64) -> i64 {
    x.pow(2) + y.pow(2)
}
