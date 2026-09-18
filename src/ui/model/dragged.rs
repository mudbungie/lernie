//! **The edges the operator has dragged**, and nothing else.
//!
//! Every field is an option because absence is an answer and not a missing
//! number: an edge nobody has moved has no width of its own, and what it is
//! shown at is `crate::ui::shell::policy`'s (DESIGN §4.39). So the default is
//! the whole of a first run, and a seat that has never been dragged holds
//! nothing here to keep in step with the policy.
//!
//! **It is per-seat state and it is durable** — REMOTE §7's plainest member —
//! so this is also the shape that goes in the place file beside the aim
//! (`crate::place`, DESIGN §4.13).

/// What the operator has dragged, in the seat's own points and rows.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct Dragged {
    /// **The roster pane's own width**, where the operator set one.
    pub roster: Option<f32>,
    /// **The conversation list's**, on the same terms.
    pub convs: Option<f32>,
    /// **How many rows the composer's field stands at**, where the operator
    /// dragged its top edge. `crate::ui::theme::COMPOSER_ROWS` is the default,
    /// which is what absence here means.
    pub rows: Option<u8>,
}
