//! **The named world states the matrix renders**, and the reason there are
//! thirteen rather than one.
//!
//! A snapshot of one model is a photograph of one moment; what an operator
//! actually needs to see is the window in each of the shapes it takes. These
//! twelve are the shapes: nothing dialled yet, seated at a wall with a
//! conversation open, the wall with none selected, and the eight states of the
//! seven panes that cover the conversation.
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

use crate::test_support::window::{
    commanded, finding, pinned, queued, recorded, role, seated, signing, tuned,
};
use crate::ui::{Edit, Enrolling, Model, Tuning, Unmaking};

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

/// **The window with the commands pane open** — the sixth covered state
/// (bl-40ec), and the only screen every sentence a help row can carry is on.
fn commands() -> World {
    World {
        name: "commands",
        model: commanded(),
    }
}

/// **The window with the find pane open and answered** — the seventh covered
/// state (bl-40ec), and the only screen `search`'s control is on.
///
/// It is photographed with a needle already in the box, because the control is
/// what this world exists to put on the glass and it is disabled until there
/// is one. The unarmed state is the pane's own suite's, where an assertion can
/// name the sentence beside the greyed control.
fn find() -> World {
    World {
        name: "finding",
        model: finding(),
    }
}

/// **The window with the login pane open and answered** — the eighth covered
/// state (bl-e3c5), and the only screen `login`'s and `models`' controls are
/// on.
///
/// It is photographed mid-flow, with a run being followed and a row asked what
/// it offers, because that is the state every sentence on the pane is reachable
/// in: an unanswered login pane says one thing and has no control at all.
fn login() -> World {
    World {
        name: "signing",
        model: signing(),
    }
}

/// **The window with the aimed wall pinned** (bl-7782) — not a covered state
/// at all, and the only screen the `unpin` control is on.
///
/// It is a world for `crate::snapshot::parity`'s reason exactly: the pin pair
/// are assertions, so each row carries the one act that is not already true of
/// it, and the world at work carries only `pin`. A control that lives on a
/// screen this walk never visits fails honestly.
fn pinned_wall() -> World {
    World {
        name: "pinned",
        model: pinned(),
    }
}

/// **The window with an unmaking standing** — the sixth covered state
/// (bl-48fa), and the only screen `delete-workspace`'s control is on.
///
/// It is photographed **unarmed**, which is the state the pane opens in and the
/// one an operator actually meets: the box empty, the sentence saying what
/// would arm it, and the control on the glass and not live. A world armed would
/// photograph the half-second before the act instead of the pane.
fn unmaking() -> World {
    let mut model = seated();
    let aim = model
        .aim
        .clone()
        .unwrap_or_else(|| panic!("the seated fixture is aimed at a wall"));
    model.unmaking = Some(Unmaking::at(aim));
    World {
        name: "unmaking",
        model,
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
        commands(),
        find(),
        login(),
        pinned_wall(),
        unmaking(),
    ]
}
