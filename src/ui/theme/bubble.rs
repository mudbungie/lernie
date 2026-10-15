//! **The four bubbles, by role** (DESIGN §4.45 item 7; `docs/STYLE.md` §5,
//! *a block*): the fill that stands under each block of the transcript, named
//! once here and derived from the palette — no new literal.
//!
//! The operator's is the one hued fill, the brand at half a tint: *I'm
//! reading the message I wrote* is answered by hue. The model's three are the
//! ground ladder read as an elevation ramp rising toward what is read — a
//! trail on the ground inside a hairline, the lowest step, because its calls
//! are bookkeeping once they have run; intermediate prose one rung up; the
//! turn's final prose on the top rung. Every one is faint, which is the
//! ruling's *not jarring*: a bubble says a role and is read through, never
//! read.
//!
//! **Which bubble a row gets is decided in the projection**
//! (`crate::ui::chat::rows`), where the turn's shape is known; this file only
//! says what each one looks like.

use egui::{Color32, Stroke};

use super::{BRAND, GROUND, HAIRLINE, RAISED, SURFACE, TINT_ALPHA};

/// The role a block of the transcript stands on.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Bubble {
    /// The operator's own turn: the brand's tint.
    Operator,
    /// The trail — the calls, their results, the reasoning: the ground inside
    /// a hairline.
    Trail,
    /// The model's prose a further trail follows within the same turn, and the
    /// live turn's prose while it streams: `SURFACE`.
    Intermediate,
    /// The turn's final prose: `RAISED`.
    Final,
}

/// **The fill and the stroke a bubble wears.** Three of the four carry no
/// stroke — a block is raised by a tint, never boxed — and the trail's
/// hairline is what keeps the lowest step visible at all on the ground it
/// stands on.
pub fn bubble(role: Bubble) -> (Color32, Stroke) {
    match role {
        Bubble::Operator => (BRAND.gamma_multiply(TINT_ALPHA / 2.0), Stroke::NONE),
        Bubble::Trail => (GROUND, Stroke::new(1.0, HAIRLINE)),
        Bubble::Intermediate => (SURFACE, Stroke::NONE),
        Bubble::Final => (RAISED, Stroke::NONE),
    }
}
