//! **The named world states the matrix renders**, and the reason there are
//! three rather than one.
//!
//! A snapshot of one model is a photograph of one moment; what an operator
//! actually needs to see is the window in each of the shapes it takes. These
//! three are the shapes: nothing dialled yet, seated at a wall with a
//! conversation open, and the one pane that covers another.
//!
//! They are built from [`crate::test_support::window`]'s fixtures rather than
//! from a second set of their own — a fixture with a field the row grew is a
//! fixture that stops compiling, and two of them is two places to fill it in.

use crate::test_support::window::seated;
use crate::ui::{Enrolling, Model};

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
fn unprovisioned() -> World {
    World {
        name: "unprovisioned",
        model: Model::default(),
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

/// Every world the matrix renders, in the order it renders them.
pub(crate) fn all() -> Vec<World> {
    vec![unprovisioned(), working(), enrolling()]
}
