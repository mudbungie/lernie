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
//! still there, still correct, and nobody can get to it.
//!
//! **The premise has since come true** (bl-4a2c). [`crate::ui::tuning`] is a
//! settings panel by any reading: a place you go to, act in, and come back
//! from, holding what a wall's roles are set to. So the walk covers both
//! covering panes rather than the one — and it is still two gestures each, in
//! and back out, because that bound is what the assertion is.
//!
//! **And the bound is a fact about the SHAPE, so it is asked per shape**
//! (bl-dfda). The narrow layout puts one column on the glass at a time, which
//! is the very thing that makes a pane's control unreachable when a window
//! narrows — so the walk asks for it there: one gesture to the column the
//! control lives on, then the same two. Three per pane, stated, is the
//! assertion; a pane that needed a fourth is the defect it exists to catch.
//!
//! # Why the accessibility tree and not the glass
//!
//! [`crate::test_support::window::click`] aims at painted glyphs, which is the
//! right instrument for *"is this word on the screen"*. It cannot answer *"can
//! this be acted on"*: a run of text that reached the glass may be a label, and
//! a control an operator can reach by keyboard may have no glyph of its own.
//! The accessibility tree is the set of things that ARE controls, which is the
//! set the question is about.

use crate::ui::{Column, Model, Shape, enroll, records, tuning};
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

/// **One covering pane, and the column its opening control lives on.**
///
/// The column is what makes the walk answerable in both shapes: in the broad
/// shape every column is on the glass, so it costs nothing to be told; in the
/// narrow shape it is exactly the gesture that has to be spent first.
struct Covered {
    column: Column,
    open: &'static str,
    close: &'static str,
    heading: &'static str,
}

/// **The three covered panes, in the order the walk visits them.**
///
/// The tuning pane goes first because both roster-row controls stand the other
/// down while one is open — so a walk that opened the enrollment first would
/// find the tuning control gone and complain about a seat that is behaving
/// exactly as designed. The records pane goes last for the mirror of that
/// reason: its control hangs off the composer, which every covering pane stands
/// down, so it is walked once both roster-row panes have been closed again
/// (bl-2cf7).
const PANES: [Covered; 3] = [
    Covered {
        column: Column::Channels,
        open: tuning::OPEN,
        close: tuning::CLOSE,
        heading: tuning::HEADING,
    },
    Covered {
        column: Column::Channels,
        open: enroll::OPEN,
        close: enroll::CLOSE,
        heading: enroll::HEADING,
    },
    Covered {
        column: Column::Conversation,
        open: records::OPEN,
        close: records::CLOSE,
        heading: records::HEADING,
    },
];

/// **The walk, and its length is the bound** — which is a fact about the SHAPE
/// and so is answered per shape (bl-dfda).
///
/// Two legs per pane, one gesture in and one back out. The narrow shape adds
/// exactly one more per pane, and adds it unconditionally rather than only
/// where the column has to change: **go to the column the control lives on,
/// open, close**. That the step is sometimes a click on the column already
/// showing is the point — a bound that varied with where the walk happened to
/// be standing would be a bound nobody could state.
///
/// A leg appearing beyond these is the assertion telling you a pane moved
/// further away than the shape it is in accounts for.
pub(crate) fn walk(shape: Shape) -> Vec<Step> {
    PANES
        .iter()
        .flat_map(|pane| {
            let nav = matches!(shape, Shape::Narrow).then(|| Step {
                gesture: pane.column.word(),
                then: pane.open,
            });
            nav.into_iter().chain([
                Step {
                    gesture: pane.open,
                    then: pane.heading,
                },
                Step {
                    gesture: pane.close,
                    then: pane.open,
                },
            ])
        })
        .collect()
}

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
