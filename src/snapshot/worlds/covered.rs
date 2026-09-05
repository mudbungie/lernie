//! **The covered states** — one named world per pane that stands over the
//! conversation, each answered.
//!
//! Split from [`super`] at the 300-line cap on the seam that module's own doc
//! draws: [`super`] is *the window in the shapes it takes* — nothing dialled,
//! seated, at a wall with nothing selected, a wall pinned — and this is *the
//! window with a pane covering it*. The first is closed; the second grows once
//! per pane, which is why it is the half that had to move.
//!
//! **Every one of them is ANSWERED**, and that is the parity instrument rather
//! than a preference (yog's `docs/PARITY.md` §5, *unproven is red*): a control
//! that lives only on a screen this walk never visits fails honestly, and an
//! unanswered pane offers no controls at all.

use super::World;
use crate::test_support::window::panes::board::{amending, filing};
use crate::test_support::window::{
    boarded, clearing, commanded, configured, finding, fleeting, machines, queued, recorded, role,
    signing, trailing, tuned,
};
use crate::ui::{Edit, Model, Tuning};

/// **The window with the tuning pane open** — the settings surface this seat
/// spent its first year without (`crate::snapshot::reach` on the premise).
///
/// It is answered rather than waiting, because the controls are what this
/// world exists to put on the glass and there are none until a row is.
pub(super) fn tuning() -> World {
    World {
        name: "tuning",
        model: tuned(),
    }
}

/// **The window with one role's assignment being rewritten** — the third
/// covered state, and the only screen the `model` control exists on.
pub(super) fn assigning() -> World {
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
pub(super) fn records() -> World {
    World {
        name: "records",
        model: recorded(),
    }
}

/// **The window with the decision queue open** — the fifth covered state
/// (bl-f0ef), and the only screen `attention`'s answer, `seen`'s control and
/// every sentence a queue row can carry are on.
pub(super) fn queue() -> World {
    World {
        name: "queue",
        model: queued(),
    }
}

/// **The window with the trail open and answered** (bl-4c48) — the sixth
/// covered state, and the only screen a trail row's sentences are on.
///
/// It is answered for the reason every other pane's world is: what a world
/// exists to photograph is every sentence the pane can say, and an unanswered
/// trail says exactly one.
pub(super) fn trail() -> World {
    World {
        name: "trailing",
        model: trailing(),
    }
}

/// **The window with the ball pane open and answered** (bl-d2af) — the only
/// screen a board row's sentences, a loop's line, a binding row and the aimed
/// wall's own balls are on.
///
/// It is answered, and answered on BOTH its widths, for the reason every other
/// pane's world is: what a world exists to photograph is every sentence the
/// pane can say, and a pane answered on one width only would leave the other's
/// sentences on no screen this walk visits.
pub(super) fn board() -> World {
    World {
        name: "boarding",
        model: boarded(),
    }
}

/// **The window with the ball pane's authoring block open on a ball that does
/// not exist yet** (bl-f7ae) — the only screen `create`'s control is on.
///
/// Two worlds rather than one because the block has two subjects and never
/// both at once: this one offers the filing and the next one the three acts on
/// a ball that exists. Its boxes are filled for `fleeting`'s reason — a world
/// photographed unfilled puts every control it gates on no screen this walk
/// visits (yog's `docs/PARITY.md` §5, *unproven is red*).
pub(super) fn ball_filing() -> World {
    World {
        name: "filing",
        model: filing(),
    }
}

/// **The window with that block open on a ball the wall holds** — the only
/// screen `update`, `release` and `close` are on.
///
/// It is photographed **armed**, which is the opposite of the unmaking world's
/// choice and for the reason that world states: there the arming is the whole
/// subject of the photograph, and here it is one of three controls that would
/// otherwise be on no screen at all.
pub(super) fn ball_amending() -> World {
    World {
        name: "amending",
        model: amending(),
    }
}

/// **The window with the fleet pane open and answered** (bl-a43a) — the only
/// screen the five fleet controls, an attempt's sentences and a churn row are
/// on.
///
/// It is answered and its three boxes are FILLED, because two of the five
/// controls are disabled until theirs is: a world with empty boxes would
/// photograph the pane an operator meets and put two of its controls on no
/// screen this walk visits (yog's `docs/PARITY.md` §5, *unproven is red*).
pub(super) fn fleet() -> World {
    World {
        name: "fleeting",
        model: fleeting(),
    }
}

/// **The window with the commands pane open** — the sixth covered state
/// (bl-40ec), and the only screen every sentence a help row can carry is on.
pub(super) fn commands() -> World {
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
pub(super) fn find() -> World {
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
pub(super) fn login() -> World {
    World {
        name: "signing",
        model: signing(),
    }
}

/// **The window with the clients pane open and answered** (bl-e53c) — the
/// ninth covered state, and the only screen `clients`' control is reachable
/// from at all.
///
/// It is photographed answered, for the login world's reason: what the world
/// exists to put on the glass is every sentence the pane can say about a
/// machine — connected, not connected, and advertising nothing — and an
/// unanswered pane says one.
pub(super) fn clients() -> World {
    World {
        name: "machines",
        model: machines(),
    }
}

/// **The window with the config pane open, pointed at a file and answered**
/// (bl-5c53) — the eleventh covered state, and the only screen `config`'s own
/// control is reachable from.
///
/// It is photographed pointed at a destination, because the pane pointed at
/// nothing says one sentence and none of the settings, the fault or the bytes
/// this world exists to put on the glass.
pub(super) fn config() -> World {
    World {
        name: "config",
        model: configured(),
    }
}

/// **The window with the place a trail is cut in standing** (bl-b8f7) — the
/// only screen `clear-trail`'s control is on.
///
/// It is a world of its own for `crate::snapshot::parity`'s reason exactly:
/// the act lives one pane deeper than the roster's own ops row, so a walk that
/// only ever opened the trail would never see it, and a control on no screen
/// this set visits fails honestly.
pub(super) fn clearing_trail() -> World {
    World {
        name: "clearing",
        model: clearing(),
    }
}
