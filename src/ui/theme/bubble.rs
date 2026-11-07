//! **The bubbles, nested** (DESIGN §4.48 item 2, amending §4.45 item 7;
//! `docs/STYLE.md` §5, *a block*): the fill and the stroke under each block
//! of the transcript, named once here and derived from the palette — no new
//! literal.
//!
//! The operator's is the one hued fill, the brand at half a tint inside a
//! hairline: *I'm reading the message I wrote* is answered by hue, and the
//! stroke is the baseline's prompt bubble, the one block a person wrote. The
//! rest is a ramp read **inward**: the working — the layer that will collapse
//! — is the ground inside a hairline; each step inside it stands on
//! `SURFACE`, intermediate prose being one such step; a step's content (a
//! call's output, the command card) is the ground again. The turn's final
//! prose stands on `RAISED`, the top rung, because it is what is read. Every
//! one is faint, which is the ruling's *not jarring*: a bubble says a role
//! and is read through, never read.
//!
//! **Which bubble a row gets is decided in the projection**
//! (`crate::ui::chat::rows`), where the turn's shape is known; the two the
//! painter lays inside a row — the step and the card — are the shape of a
//! step, which every step has. This file only says what each looks like.

use egui::{Color32, Stroke};

use super::{BRAND, GROUND, HAIRLINE, RAISED, SURFACE, TINT_ALPHA};

/// The role a block of the transcript stands on.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Bubble {
    /// The operator's own turn: the brand's tint inside a hairline.
    Operator,
    /// The working — every row between the operator's turn and the final
    /// prose, as one bubble: the ground inside a hairline.
    Trail,
    /// The model's prose a further step follows within the same turn, and
    /// the live turn's prose while it streams: a step, so `SURFACE`.
    Intermediate,
    /// **One step inside the working** — a call, a thought, a note: the
    /// sub-bubble, on `SURFACE`.
    Step,
    /// **A step's content** — a call's output under its call line, the
    /// command card: the ground again, with no stroke.
    Card,
    /// The turn's final prose: `RAISED`.
    Final,
}

/// **The fill and the stroke a bubble wears.** Two carry a hairline — the
/// operator's, and the working's, which is what keeps the lowest step
/// visible at all on the ground it stands on — and the rest are raised by a
/// tint alone, never boxed.
pub fn bubble(role: Bubble) -> (Color32, Stroke) {
    let hairline = Stroke::new(1.0, HAIRLINE);
    match role {
        Bubble::Operator => (BRAND.gamma_multiply(TINT_ALPHA / 2.0), hairline),
        Bubble::Trail => (GROUND, hairline),
        Bubble::Intermediate | Bubble::Step => (SURFACE, Stroke::NONE),
        Bubble::Card => (GROUND, Stroke::NONE),
        Bubble::Final => (RAISED, Stroke::NONE),
    }
}
