//! **Assertion (b): where the layout says there is something, the glass is not
//! blank.**
//!
//! This is the detector for the defect the commit under this one fixed — a seat
//! that painted a **giant black box in the middle of the window**. Every glyph
//! assertion in the suite passed through it, because a slab painted over a
//! window is not a missing word: the words were all laid out, and something was
//! drawn on top of them.
//!
//! # Why the subject is a content rectangle and not the image
//!
//! The obvious detector — scan the whole frame for a large region of near-zero
//! variance — was built first and measured, and it cannot work. A window has
//! **legitimately quiet ground**: an empty conversation pane is one enormous
//! flat area by design. Measured over this matrix, the largest flat square in a
//! sound 1400x900 window is 880 points, or 98% of its shorter side. There is no
//! threshold that separates that from a slab, because they are the same
//! measurement.
//!
//! What separates them is **what the layout claimed was there**. The
//! accessibility tree names every node that has content to show and where it
//! put it; so the question is asked one rectangle at a time, of the rectangles
//! that are supposed to have something in them. A slab over the window turns
//! all of them flat at once. An empty pane turns none of them flat, because it
//! has no content rectangles in it to turn.
//!
//! That reading is also the ball's own words — *a large rect of near-zero pixel
//! variance INSIDE THE LAYOUT* — taken literally.
//!
//! # What it catches beyond a slab
//!
//! The same measurement is the one that catches text painted in the background
//! colour, a label whose ink and ground are the same theme token, and a control
//! that reserved its rectangle and drew nothing in it. All four are the same
//! defect from the operator's side: the layout is right and the glass is empty.

use crate::ui::Model;
use egui_kittest::Harness;
use egui_kittest::kittest::{Queryable, by};
use image::RgbaImage;

/// **How still a rectangle has to be to count as blank**, as a per-channel
/// range. Not zero: a panel background carries a hairline gradient and a
/// rounded corner antialiases, and a detector that only fires on
/// mathematically identical pixels fires on nothing a renderer produces.
pub(crate) const TOL: u8 = 8;

/// **The smallest rectangle worth judging**, per side. Under it there is not
/// enough glass for "flat" to mean anything — a separator is one still line by
/// construction, and a control with three visible pixels at the window's edge
/// is [`super::clipped`]'s complaint, not this one.
pub(crate) const MIN: u32 = 6;

/// **The last whole pixel at or before `v`**, clamped into `0..=hi`.
///
/// By search rather than by cast. The house lint set denies a lossy numeric
/// conversion and the only home for a suppression is the manifest, so the
/// conversion is done the way `crate::ui::enroll::points` does it in the other
/// direction — exactly, by a route the lint has nothing to say about. A window
/// is a couple of thousand pixels on its longest side and this runs four times
/// per node, so the scan costs nothing worth naming.
fn whole(v: f64, hi: u32) -> u32 {
    (0..=hi).rev().find(|&n| f64::from(n) <= v).unwrap_or(0)
}

/// **The part of a node's rectangle that is actually on the image**, or `None`
/// when too little of it is.
///
/// The bounds arrive as four numbers rather than as a rectangle because the
/// type they come in belongs to a crate this one does not declare.
pub(crate) fn visible(
    bounds: (f64, f64, f64, f64),
    width: u32,
    height: u32,
) -> Option<(u32, u32, u32, u32)> {
    let (x0, y0, x1, y1) = bounds;
    let (left, top) = (whole(x0, width), whole(y0, height));
    let (right, bottom) = (whole(x1, width), whole(y1, height));
    (right >= left + MIN && bottom >= top + MIN).then_some((left, top, right, bottom))
}

/// Whether every channel across this rectangle varies by no more than [`TOL`].
pub(crate) fn flat(image: &RgbaImage, rect: (u32, u32, u32, u32)) -> bool {
    let (left, top, right, bottom) = rect;
    let mut lo = [u8::MAX; 4];
    let mut hi = [0_u8; 4];
    for y in top..bottom.min(image.height()) {
        for x in left..right.min(image.width()) {
            let px = image.get_pixel(x, y).0;
            for ((low, high), value) in lo.iter_mut().zip(hi.iter_mut()).zip(px.iter()) {
                *low = (*low).min(*value);
                *high = (*high).max(*value);
            }
        }
    }
    lo.iter()
        .zip(hi.iter())
        .all(|(low, high)| high - low <= TOL)
}

/// **The words a node carries**, wherever the tree keeps them.
///
/// It is two places and both are load-bearing. A control's text is its
/// `label`; a plain label's text is its `value`, with `label` left empty and
/// `label_comes_from_value` set. Reading only the first is how the first cut of
/// this detector judged every control in the window and not one label in it —
/// which is to say, judged almost nothing, quietly, and passed.
fn words(node: &egui_kittest::kittest::Node<'_>) -> Option<String> {
    node.label()
        .or_else(|| node.value())
        .filter(|text| !text.trim().is_empty())
}

/// **Every rectangle the layout put content in, read off the glass.** An empty
/// answer is the assertion holding.
///
/// Only **leaves** are judged, and that is what keeps one blank word from being
/// three complaints: a label wraps a text run and a button wraps its caption,
/// so a parent's rectangle is its child's rectangle plus padding, and the ink
/// is at the bottom. Judging the leaf names the thing that is missing rather
/// than the box around it.
pub(crate) fn complaints(at: &str, image: &RgbaImage, harness: &Harness<'_, Model>) -> Vec<String> {
    harness
        .query_all(by())
        .filter(|node| !node.is_hidden())
        .filter(|node| node.query_all(by().recursive(false)).next().is_none())
        .filter_map(|node| {
            let label = words(&node)?;
            let bounds = node.bounding_box()?;
            let rect = visible(
                (bounds.x0, bounds.y0, bounds.x1, bounds.y1),
                image.width(),
                image.height(),
            )?;
            flat(image, rect).then(|| {
                let (left, top, right, bottom) = rect;
                format!(
                    "{at}: {:?} reading {label:?} holds {}x{} at {left},{top} and every pixel in it is the same — the layout put something there and the glass is blank",
                    node.role(),
                    right - left,
                    bottom - top
                )
            })
        })
        .collect()
}
