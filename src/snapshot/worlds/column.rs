//! **The column's own states** — the ask above the prompt, the dock
//! unfolded, two tabs open, a notice standing — split from [`super`] at the
//! 300-line cap on the seam the names draw: these are shapes the column
//! takes with no pane over it.

use super::World;
use crate::test_support::window::seated;

/// **The ask above the prompt** (DESIGN §4.42, item 8): the one screen its
/// numbered rows and their tags are on with no pane open.
pub(super) fn asked_world() -> World {
    World {
        name: "asked",
        model: crate::test_support::window::asked(),
    }
}

/// **The dock unfolded over a conversation at work** (DESIGN §4.42, item 9).
pub(super) fn docked() -> World {
    let mut model = seated();
    model.convs[0].state = crate::reply::convs::AgentState::InFlight;
    World {
        name: "docked",
        model,
    }
}

/// **Two conversations open as tabs** (DESIGN §4.44, item 2): the one screen
/// the strip and its tags are on.
pub(super) fn tabbed() -> World {
    let mut model = seated();
    let second = crate::test_support::window::conv("20260830T052000Z-b2c3", "tidy the rule");
    model.convs.push(second);
    for root in ["20260830T052000Z-b2c3", "20260830T051200Z-a1b2"] {
        model.select(root);
    }
    model.transcript = seated().transcript;
    World {
        name: "tabbed",
        model,
    }
}

/// **A notice longer than the rule** (bl-7e3e): it stands whole above the
/// rule, and the rule keeps its facts and its name.
pub(super) fn noticed() -> World {
    let mut model = seated();
    model.notice = Some(crate::ui::Notice::InDoubt {
        op: "message".to_owned(),
        why: "the connection closed before the reply".to_owned(),
    });
    World {
        name: "noticed",
        model,
    }
}
