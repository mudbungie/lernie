//! **Assertion (a): the pane behind a control is still behind it at every
//! window size**, asked of the accessibility tree rather than of the pixels.
//!
//! # What "the settings panel" is in this seat
//!
//! The ball that filed this asked for *the settings panel, reachable from the
//! main screen in a bounded number of gestures at every matrix size*. **This
//! seat has no settings panel** — the window is a notice bar, the roster, the
//! composer and the conversation, and there is no preferences surface
//! anywhere in `crate::ui`. The premise was written from
//! the shape a desktop app usually has.
//!
//! What the assertion is *about* survives that intact: a window has surfaces
//! you navigate to and back from, and the way they break is that the control
//! opening one stops being reachable when the window narrows — the pane is
//! still there, still correct, and nobody can get to it.
//!
//! **The premise has since come true** (bl-4a2c). [`crate::ui::tuning`] is a
//! settings panel by any reading: a place you go to, act in, and come back
//! from, holding what a workspace's roles are set to. So the walk covers every
//! covering pane rather than the one — eleven of them since bl-a43a — and it is
//! still two gestures each, in and back out, because that bound is what the
//! assertion is, and it is the same bound whether the seat has one such pane or
//! seven.
//!
//! **The strips went and the words came** (bl-e016, DESIGN §4.42). Every
//! pane's control used to hang off one of three strips; every pane is now a
//! word typed into the one box, so the walk types. The bound tightened: two
//! gestures from the box whichever pane, the word and Escape.
//!
//! **The accordion moved nothing here** (bl-cff1, DESIGN §4.39). The walk's
//! counts were re-read against it and stand: all eleven controls hang off the
//! roster's own acts strip or off the aimed workspace's band, the aimed workspace is
//! always under the OPEN server, and a seat is never open over nothing — so
//! every control the walk spends is on the glass for the same two gestures it
//! always was. A server row is not a leg: it opens a section of the one list
//! rather than a pane to come back from.
//!
//! **And neither did the fold** (bl-b9a3). The middle column went, so the
//! columns below are two rather than three — but every control still hangs
//! off the same strip or the same band, and the one on the composer still
//! hangs off the conversation's own column. A conversation's row is not a leg
//! either: it selects, which is what the column beside it is already showing.
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

use crate::ui::{
    Model, board, clients, commands, config, doctor, enroll, find, fleet, keys, login, prices,
    queue, records, theme, trail, tuning, unmake,
};
use egui_kittest::Harness;
use egui_kittest::kittest::Queryable;

/// **One gesture the walk spends.**
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Gesture {
    /// A click on the control reading exactly this label.
    Click(&'static str),
    /// A slash verb typed into the box and fired with Enter (DESIGN §4.42,
    /// item 5) — the way into every pane.
    Type(&'static str),
    /// Escape, the way out of every overlay (`crate::ui::overlay`).
    Escape,
}

/// **One leg of the walk**: the gesture to spend, and what has to be there
/// after spending it.
pub(crate) struct Step {
    pub(crate) gesture: Gesture,
    /// A label that must be reachable once the frame has settled.
    pub(crate) then: &'static str,
}

/// **One pane: the word that opens it, and the heading it stands under.**
struct Covered {
    word: &'static str,
    heading: &'static str,
}

/// **The fifteen panes this walk visits.** Every one is two gestures from the
/// box: the word, and Escape back out. The enrollment is among them because
/// Escape closes it too — the one pane whose way out forgets something
/// (`Model::escape`).
const PANES: [Covered; 15] = [
    Covered {
        word: queue::WORD,
        heading: queue::HEADING,
    },
    Covered {
        word: tuning::WORD,
        heading: tuning::HEADING,
    },
    Covered {
        word: enroll::WORD,
        heading: enroll::HEADING,
    },
    Covered {
        word: login::WORD,
        heading: login::HEADING,
    },
    Covered {
        word: fleet::WORD,
        heading: fleet::HEADING,
    },
    Covered {
        word: unmake::WORD,
        heading: unmake::HEADING,
    },
    Covered {
        word: trail::WORD,
        heading: trail::HEADING,
    },
    Covered {
        word: board::WORD,
        heading: board::HEADING,
    },
    Covered {
        word: commands::WORD,
        heading: commands::HEADING,
    },
    Covered {
        word: doctor::WORD,
        heading: doctor::HEADING,
    },
    Covered {
        word: find::WORD,
        heading: find::HEADING,
    },
    Covered {
        word: prices::WORD,
        heading: prices::HEADING,
    },
    Covered {
        word: records::WORD,
        heading: records::HEADING,
    },
    Covered {
        word: clients::WORD,
        heading: clients::HEADING,
    },
    Covered {
        word: config::WORD,
        heading: config::HEADING,
    },
];

/// **The walk, and its length is the bound.** Two legs per pane: the word
/// typed, and Escape — at every size, because there is one shape (§4.42). A
/// leg appearing beyond these is the assertion telling you a pane moved
/// further away than the box.
pub(crate) fn walk() -> Vec<Step> {
    PANES
        .iter()
        .flat_map(|pane| {
            [
                Step {
                    gesture: Gesture::Type(pane.word),
                    then: pane.heading,
                },
                Step {
                    gesture: Gesture::Escape,
                    then: theme::glyph::PROMPT,
                },
            ]
        })
        .collect()
}

/// Whether anything reachable carries exactly this label.
fn reachable(harness: &Harness<'_, Model>, label: &'static str) -> bool {
    harness.query_all_by_label(label).next().is_some()
}

/// Spend one gesture, and say whether there was anything to spend it on.
fn spend(harness: &mut Harness<'_, Model>, gesture: Gesture) -> bool {
    match gesture {
        Gesture::Click(label) => {
            let Some(node) = harness.query_by_label(label) else {
                return false;
            };
            node.click();
        }
        Gesture::Type(word) => {
            // The caret into the box, the line into it, Enter on it — the
            // three things an operator does, in order, each a frame.
            let id = egui::Id::new(keys::BOX_ID);
            harness.ctx.memory_mut(|memory| memory.request_focus(id));
            harness.run();
            harness.input_mut().events.push(egui::Event::Text(format!(
                "{}{word}",
                crate::ui::slash::LEAD
            )));
            harness.run();
            harness.press_key(egui::Key::Enter);
        }
        Gesture::Escape => harness.press_key(egui::Key::Escape),
    }
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
        if !spend(harness, step.gesture) {
            out.push(format!(
                "{at}: no control reads {:?} — the pane behind it cannot be reached",
                step.gesture
            ));
            break;
        }
        if !reachable(harness, step.then) {
            out.push(format!(
                "{at}: one gesture, {:?}, did not bring {:?}",
                step.gesture, step.then
            ));
            break;
        }
    }
    out
}
