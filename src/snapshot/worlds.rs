//! **The named world states the matrix renders**, and the reason there are
//! many rather than one: a snapshot of one model is a photograph of one
//! moment, and what an operator needs to see is each shape the window takes.
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

use crate::test_support::window::{pinned, seated, workspace};
use crate::ui::{Channel, Chunk, Enrolling, Model, Unmaking};

/// The bar open, and its foot's list.
mod bar;
/// The column's own states: the ask, the dock, the tabs, the notice.
mod column;
/// The covered states — one world per pane that stands over the conversation.
mod covered;

use column::{asked_world, docked, noticed, tabbed};
use covered::{
    assigning, ball_amending, ball_filing, board, clearing_trail, clients, commands, config,
    doctor, dropped, find, fleet, login, prices, queue, records, settings, trail, tuning,
};

/// One named state of the window, as the matrix files it.
pub(crate) struct World {
    /// The name that goes in the filename and in a complaint.
    pub(crate) name: &'static str,
    /// The model the frame paints from.
    pub(crate) model: Model,
}

/// **The window with nothing in it**: no channel heard from, no workspace, no
/// conversation. It is the first thing a new operator sees, and the state in
/// which every pane has to say what it has instead of showing it.
///
/// **Its roster is seeded the way `src/main.rs` seeds one** — off a data root
/// holding nothing at all — rather than left empty (bl-dfda): an empty roster
/// is unreachable on a real box (`crate::seat::channels` answers this box's
/// own slot regardless, bl-08b6), and photographing it photographed a fixture.
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

/// **The window at work**: one channel, one workspace aimed at, one conversation
/// selected and one thing said in it. This is "the main screen" every other
/// assertion is stated against.
fn working() -> World {
    World {
        name: "seated",
        model: seated(),
    }
}

/// **The window on a workspace with nothing selected** — the composer's other
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
        .unwrap_or_else(|| panic!("the seated fixture is aimed at a workspace"));
    model.enroll = Some(Enrolling::at(aim));
    World {
        name: "enrolling",
        model,
    }
}

/// **The window with one server open and another closed** (bl-cff1; DESIGN
/// §4.39) — the accordion's other state, and the only screen a CLOSED server's
/// row is on at all.
///
/// It is a world for `crate::snapshot::parity`'s reason, one row kind over: a
/// seat with one server is always open over it, so the fold — a row with its
/// workspaces beneath it beside a row with nothing beneath it — is a shape the walk
/// never visits unless a world is this. The second server holds workspaces
/// deliberately: a server folded over NOTHING would photograph the same
/// picture a server with nothing in it does.
fn accordion() -> World {
    let mut model = seated();
    model.roster.push(Chunk {
        channel: Channel {
            name: "lab".to_owned(),
            named_there: None,
            dials: Some("127.0.0.1:7737".to_owned()),
        },
        held: crate::ui::Held::Heard,
        workspaces: vec![workspace("bench"), workspace("annex")],
        ..Chunk::default()
    });
    World {
        name: "accordion",
        model,
    }
}

/// **The completion menu over a seated window** (DESIGN §4.42, item 5) —
/// the one screen every verb's row is on, which is where the parity walk
/// reads the tags the strips used to carry.
fn slashing() -> World {
    let mut model = seated();
    model.draft = crate::ui::slash::LEAD.to_string();
    World {
        name: "slashing",
        model,
    }
}

/// **The same menu over a pinned workspace**, because the row offered there is
/// `unpin` and nowhere else is (§4.25).
fn unpinning() -> World {
    let mut model = pinned();
    model.draft = crate::ui::slash::LEAD.to_string();
    World {
        name: "unpinning",
        model,
    }
}

/// **The switcher over a seated window** (DESIGN §4.42, item 7): the one
/// screen the roster's rows and their tags are on.
fn switching() -> World {
    let mut model = seated();
    model.switching = true;
    World {
        name: "switching",
        model,
    }
}

/// **The aimed workspace pinned** (bl-7782): the only screen `unpin` is on.
fn pinned_workspace() -> World {
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
        .unwrap_or_else(|| panic!("the seated fixture is aimed at a workspace"));
    model.unmaking = Some(Unmaking::at(aim));
    World {
        name: "unmaking",
        model,
    }
}

/// **The window sent to the deletion's arming box** (bl-f251) — a row menu's
/// *delete…*, which lands the cursor in the box on the composer
/// (`crate::ui::model::fill`), and so the one screen on which the composer's
/// strip stands open with the acts it holds.
///
/// It is a world for the pinned workspace's reason: the strip is behind one
/// control, so the floor pair, the retarget, the flag and the deletion live
/// on no screen the walk visits unless one is this. Photographed as the
/// operator meets it — sent there, the strip open, the arming box empty and
/// holding the caret.
fn filling() -> World {
    let mut model = seated();
    let conversation = model
        .conversation
        .clone()
        .unwrap_or_else(|| panic!("the seated fixture has a conversation selected"));
    model.fill_in(&conversation, crate::ui::Fill::Arming);
    World {
        name: "filling",
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
        trail(),
        clearing_trail(),
        board(),
        ball_filing(),
        ball_amending(),
        fleet(),
        commands(),
        doctor(),
        find(),
        prices(),
        login(),
        clients(),
        config(),
        settings(),
        accordion(),
        pinned_workspace(),
        slashing(),
        unpinning(),
        switching(),
        dropped(),
        asked_world(),
        docked(),
        tabbed(),
        noticed(),
        unmaking(),
        filling(),
        bar::opened(),
        bar::footed(),
    ]
}
