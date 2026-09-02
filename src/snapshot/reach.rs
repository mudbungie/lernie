//! **Assertion (a): the pane behind a control is still behind it at every
//! window size**, asked of the accessibility tree rather than of the pixels.
//!
//! # What "the settings panel" is in this seat
//!
//! The ball that filed this asked for *the settings panel, reachable from the
//! main screen in a bounded number of gestures at every matrix size*. **This
//! seat has no settings panel** — the window is a notice bar, the roster, the
//! conversation list, the composer and the conversation, and there is no
//! preferences surface anywhere in `crate::ui`. The premise was written from
//! the shape a desktop app usually has.
//!
//! What the assertion is *about* survives that intact: a window has surfaces
//! you navigate to and back from, and the way they break is that the control
//! opening one stops being reachable when the window narrows — the pane is
//! still there, still correct, and nobody can get to it. This seat has exactly
//! one such surface, [`crate::ui::enroll`], and it is the right subject on its
//! merits: it is the only pane that covers another, the only one reached by a
//! control rather than by looking, and the only one with a way back.
//!
//! # Why the accessibility tree and not the glass
//!
//! [`crate::test_support::window::click`] aims at painted glyphs, which is the
//! right instrument for *"is this word on the screen"*. It cannot answer *"can
//! this be acted on"*: a run of text that reached the glass may be a label, and
//! a control an operator can reach by keyboard may have no glyph of its own.
//! The accessibility tree is the set of things that ARE controls, which is the
//! set the question is about.

use crate::ui::{Model, enroll};
use egui_kittest::Harness;
use egui_kittest::kittest::Queryable;

/// **One leg of the walk**: the control to spend, and what has to be there
/// after spending it.
pub(crate) struct Step {
    /// The label of the control this leg clicks.
    pub(crate) gesture: &'static str,
    /// A label that must be reachable once the frame has settled.
    pub(crate) then: &'static str,
}

/// **The walk, and its length is the bound.** Two legs: one gesture in, one
/// gesture back out. A third leg appearing here is the assertion telling you
/// the pane moved further away.
pub(crate) const WALK: [Step; 2] = [
    Step {
        gesture: enroll::OPEN,
        then: enroll::HEADING,
    },
    Step {
        gesture: enroll::CLOSE,
        then: enroll::OPEN,
    },
];

/// Whether anything reachable carries exactly this label.
fn reachable(harness: &Harness<'_, Model>, label: &'static str) -> bool {
    harness.query_by_label(label).is_some()
}

/// Spend one gesture on the control reading exactly `label`, if there is one.
fn click(harness: &mut Harness<'_, Model>, label: &'static str) -> bool {
    let Some(node) = harness.query_by_label(label) else {
        return false;
    };
    node.click();
    harness.run();
    true
}

/// **Walk it, and say what broke.** An empty answer is the assertion holding.
///
/// It stops at the first leg that fails rather than carrying on: once a gesture
/// did not land, every later complaint is about a window that is not in the
/// state the walk assumes, and three consequential complaints hide the one
/// fact.
pub(crate) fn complaints(at: &str, harness: &mut Harness<'_, Model>, walk: &[Step]) -> Vec<String> {
    let mut out = Vec::new();
    for step in walk {
        if !click(harness, step.gesture) {
            out.push(format!(
                "{at}: no control reads {:?} — the pane behind it cannot be reached",
                step.gesture
            ));
            break;
        }
        if !reachable(harness, step.then) {
            out.push(format!(
                "{at}: one gesture on {:?} did not bring {:?}",
                step.gesture, step.then
            ));
            break;
        }
    }
    out
}
