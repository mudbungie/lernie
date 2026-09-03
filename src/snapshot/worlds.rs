//! **The named world states the matrix renders**, and the reason there are
//! eight rather than one.
//!
//! A snapshot of one model is a photograph of one moment; what an operator
//! actually needs to see is the window in each of the shapes it takes. These
//! eight are the shapes: nothing dialled yet, seated at a wall with a
//! conversation open, the wall with none selected, and the five states of the
//! four panes that cover the conversation.
//!
//! **The set is part of the parity instrument** and grows with it (yog's
//! `docs/PARITY.md` §5, *unproven is red*): a control that lives only on a
//! screen this walk never visits fails honestly, so the two tuning worlds
//! below are not extra photographs — they are the screens `effort`,
//! `priority` and `model` are reachable on at all.
//!
//! They are built from [`crate::test_support::window`]'s fixtures rather than
//! from a second set of their own — a fixture with a field the row grew is a
//! fixture that stops compiling, and two of them is two places to fill it in.

use crate::test_support::window::{queued, recorded, role, seated, tuned};
use crate::ui::{Edit, Enrolling, Model, Tuning};

/// One named state of the window, as the matrix files it.
pub(crate) struct World {
    /// The name that goes in the filename and in a complaint.
    pub(crate) name: &'static str,
    /// The model the frame paints from.
    pub(crate) model: Model,
}

/// **The window with nothing in it**: no channel heard from, no wall, no
/// conversation. It is the first thing a new operator sees, and the state in
/// which every pane has to say what it has instead of showing it.
///
/// **Its roster is seeded the way `src/main.rs` seeds one** — off a data root
/// holding nothing at all — rather than left empty (bl-dfda). An empty roster
/// is unreachable on a real box, because `crate::seat::channels` answers a
/// section for this box's own slot whether or not anything is provisioned in
/// it (bl-08b6), and a `Vec` with no chunk in it is a state the pane has no
/// sentence for: the narrow shape put that column alone on the glass and
/// photographed a blank window, which is a picture of a fixture rather than of
/// the seat.
fn unprovisioned() -> World {
    let scratch = crate::test_support::Scratch::new();
    World {
        name: "unprovisioned",
        model: Model {
            roster: crate::seat::channels(scratch.path()),
            ..Model::default()
        },
    }
}

/// **The window at work**: one channel, one wall aimed at, one conversation
/// selected and one thing said in it. This is "the main screen" every other
/// assertion is stated against.
fn working() -> World {
    World {
        name: "seated",
        model: seated(),
    }
}

/// **The window on a wall with nothing selected** — the composer's other
/// subject.
///
/// It is a world rather than a variation because the seat has one box with two
/// subjects (`crate::ui::composer`): with a conversation selected the box
/// deposits, and with none it *begins* one. Every screen a control lives on has
/// to be a screen this walk visits — yog's `docs/PARITY.md` §5 states it as
/// *unproven is red* — and the start control lives only here.
fn beginning() -> World {
    let mut model = seated();
    model.conversation = None;
    model.transcript = crate::reply::transcript::Transcript::default();
    World {
        name: "beginning",
        model,
    }
}

/// **The window with the enrollment open** — the one pane in this seat that
/// covers another, and so the one state where what is on the glass is not what
/// the layout underneath it says.
fn enrolling() -> World {
    let mut model = seated();
    let aim = model
        .aim
        .clone()
        .unwrap_or_else(|| panic!("the seated fixture is aimed at a wall"));
    model.enroll = Some(Enrolling::at(aim));
    World {
        name: "enrolling",
        model,
    }
}

/// **The window with the tuning pane open** — the settings surface this seat
/// spent its first year without (`crate::snapshot::reach` on the premise).
///
/// It is answered rather than waiting, because the controls are what this
/// world exists to put on the glass and there are none until a row is.
fn tuning() -> World {
    World {
        name: "tuning",
        model: tuned(),
    }
}

/// **The window with one role's assignment being rewritten** — the third
/// covered state, and the only screen the `model` control exists on.
fn assigning() -> World {
    World {
        name: "assigning",
        model: Model {
            tuning: Some(Tuning::Editing(Edit::of(&role("worker")))),
            ..tuned()
        },
    }
}

/// **The window with the records pane open** — the fourth covered state
/// (bl-2cf7), answered rather than waiting for the reason the tuning world
/// is: what this world exists to photograph is every sentence the pane can
/// say, and an unanswered pane says exactly two.
fn records() -> World {
    World {
        name: "records",
        model: recorded(),
    }
}

/// **The window with the decision queue open** — the fifth covered state
/// (bl-f0ef), and the only screen `attention`'s answer, `seen`'s control and
/// every sentence a queue row can carry are on.
fn queue() -> World {
    World {
        name: "queue",
        model: queued(),
    }
}

/// Every world the matrix renders, in the order it renders them.
pub(crate) fn all() -> Vec<World> {
    vec![
        unprovisioned(),
        working(),
        beginning(),
        enrolling(),
        tuning(),
        assigning(),
        records(),
        queue(),
    ]
}
