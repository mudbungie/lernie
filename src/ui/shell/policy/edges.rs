//! **What an operator's drag is worth**, and what the policy still owns of it
//! (DESIGN §4.39, bl-46e5).
//!
//! bl-fef8 ruled that a dragged width is *a second home for a fact the policy
//! owns*. §4.39 reverses it: REMOTE §7 makes per-seat UI state the seat's own,
//! and a width an operator set is the plainest member of that list. What the
//! policy keeps is the **default** — [`super::widths`]' yield, which is what a
//! pane is shown at until somebody drags it — and the **floors**: the least a
//! pane may be while it stands beside another, and the most it may be while
//! the conversation keeps [`super::CHAT_FLOOR`].
//!
//! So this is a pure function of the window and whatever the seat holds, and
//! nothing here is a second regime: with no drag at all every answer below is
//! exactly [`super::widths`]'.

use super::{CHAT_FLOOR, SIDE_FLOOR, widths};

/// **The least of the conversation that stands above the composer**, in
/// points — [`super::CHAT_FLOOR`] one axis over, and the bound the composer's
/// top edge stops at. A composer dragged past it leaves the pane the window
/// exists for with nothing to show, and the drag is written down, so the next
/// run would open on a window whose transcript is gone.
pub const TAIL_FLOOR: f32 = 120.0;

/// **What a list pane may be dragged between**, given `beside` — what the
/// other list pane is shown at. The floor is [`SIDE_FLOOR`], because a pane
/// under it shows nothing at all; the cap is what still leaves the
/// conversation [`CHAT_FLOOR`] beside the two of them, and it yields to the
/// floor for the policy's own reason — two panes showing nothing buys the
/// chat pane a width it still cannot use.
pub fn span(window: f32, beside: f32) -> (f32, f32) {
    (SIDE_FLOOR, (window - CHAT_FLOOR - beside).max(SIDE_FLOOR))
}

/// **What the two list panes are SHOWN at**: the dragged width where the seat
/// holds one, else the yield's.
///
/// **Only a drag is clamped**, because only a drag can be out of date: the
/// yield is this window's own answer and clamping it would be the policy
/// arguing with itself — at a window too narrow for [`CHAT_FLOOR`] the yield
/// already spends the overrun on the conversation, which is *past that nothing
/// yields* stated in one place.
///
/// The clamp is why a window briefly made small does not cost the operator
/// their drag: what is narrowed is the shown width and never the held one, so
/// the pane comes back to what they set when the window does. Each pane is
/// judged against what the OTHER asked for — the second against what the first
/// was shown — so two edges both dragged past the frame settle rather than
/// fight over one window.
pub fn shown(window: f32, dragged: (Option<f32>, Option<f32>)) -> (f32, f32) {
    let (yielded, yielded_convs) = widths(window);
    let asked = dragged.1.unwrap_or(yielded_convs);
    let roster = dragged
        .0
        .map_or(yielded, |held| clamped(held, span(window, asked)));
    let convs = dragged
        .1
        .map_or(yielded_convs, |held| clamped(held, span(window, roster)));
    (roster, convs)
}

/// One width, held between a span's ends.
fn clamped(width: f32, span: (f32, f32)) -> f32 {
    width.clamp(span.0, span.1)
}

/// **How many rows the composer's field stands at** for a top edge dragged to
/// `asked` points of panel height, where `chrome` is everything the panel
/// holds besides the field's own lines and `line` is one line of body type.
///
/// The answer is in ROWS and not in points, which is DESIGN §4.38's bound kept
/// rather than worked around: the panel is handed a height it computes from a
/// row count, and never reads one back off its content. Half a line of grace,
/// so the edge lands on the row nearest the pointer rather than the one below
/// it.
pub fn rows(asked: f32, chrome: f32, line: f32, window: f32) -> u8 {
    fits(asked + line / 2.0, chrome, line).min(fits(window - TAIL_FLOOR, chrome, line))
}

/// **How many whole lines of `line` stand in `room` once `chrome` has its
/// share** — counted rather than divided, because `f32 as u8` is the one
/// narrowing this crate's lint set denies with no home for a suppression but
/// the manifest (`crate::mark`'s own reasoning), and a bounded count is a
/// cheaper answer than a crate-wide relaxation. Never nothing: a field of no
/// rows is a composer with no box.
fn fits(room: f32, chrome: f32, line: f32) -> u8 {
    (1..=u8::MAX)
        .take_while(|rows| chrome + f32::from(*rows) * line <= room)
        .last()
        .unwrap_or(1)
}

#[cfg(test)]
mod tests;
