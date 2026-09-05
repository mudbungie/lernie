//! **The ball pane's fixtures** (bl-d2af) — the twelfth covering pane, and the
//! one whose subject is BOTH every channel and the aimed wall.
//!
//! Split from [`super::union`] at the design-time budget on the seam the pane
//! itself draws: those four panes are about nothing on the glass, and this one
//! is about that plus the wall the window is aimed at. It is here rather than
//! in [`super`] because two of its four reads fan, which is what that file's
//! own doc says its members have in common.

use crate::ui::Model;

use super::super::{own, seated};

/// One figure: a priced spend over a stated number of conversations.
pub(crate) fn figure(usd: Option<&str>, label: Option<&str>) -> crate::reply::spend::Figure {
    crate::reply::spend::Figure {
        tokens: crate::reply::steps::Spend {
            input: 11,
            output: 22,
            cache_read: 33,
            cache_write: 44,
            total: 99,
        },
        usd: usd.map(str::to_owned),
        attribution: crate::reply::spend::Attribution {
            kind: "conversations".to_owned(),
            label: label.map(str::to_owned),
        },
    }
}

/// One board row, in the column named and otherwise as quiet as a row can be.
pub(crate) fn column(id: &str, column: &str) -> crate::reply::board::BoardRow {
    crate::reply::board::BoardRow {
        id: id.to_owned(),
        column: column.to_owned(),
        state: "ready".to_owned(),
        title: format!("what {id} is for"),
        priority: 2,
        project: "lernie".to_owned(),
        workspace: None,
        claimant: None,
        parent: None,
        gates: Vec::new(),
        drones: Vec::new(),
        spend: None,
        rollup: None,
    }
}

/// **The pane with the authoring block open on a ball that does not exist
/// yet**, its two boxes FILLED — the fleet fixture's own rule: a world with
/// empty boxes photographs the block an operator meets and puts its one
/// control on no screen the parity walk visits.
pub(crate) fn filing() -> Model {
    let mut model = boarded();
    model.begin_filing();
    if let Some(block) = model.authoring.as_mut() {
        block.project = "lernie".to_owned();
        block.title = "the ball this would file".to_owned();
    }
    model
}

/// **The pane with the block open on the ball this wall holds**, its journal
/// box and its arming box filled, so all three of its acts are live.
pub(crate) fn amending() -> Model {
    let mut model = boarded();
    model.begin_amending(&bound("bl-1"));
    if let Some(block) = model.authoring.as_mut() {
        block.note = "what happened".to_owned();
        block.arm = "bl-1".to_owned();
    }
    model
}

/// One ball a wall holds, as quiet as such a row can be.
pub(crate) fn bound(id: &str) -> crate::reply::balls::BoundBall {
    crate::reply::balls::BoundBall {
        id: id.to_owned(),
        badge: Some("delivered".to_owned()),
        project: "lernie".to_owned(),
        owner: "alba".to_owned(),
        state: "bound".to_owned(),
        spend: figure(Some("$2.50"), None),
    }
}

/// **The seated model with the ball pane open and answered** (bl-d2af): a
/// claimed row carrying every line the pane can hang off one — the holder, the
/// epic, the drones, its own spend and its rollup — a gated row, a bare ready
/// row, an armed loop, a binding table with a held and an unheld row, and the
/// aimed wall's own two answers. Every sentence the pane can say, on one
/// screen.
pub(crate) fn boarded() -> Model {
    let claimed = crate::reply::board::BoardRow {
        state: "bound".to_owned(),
        workspace: Some("home".to_owned()),
        claimant: Some("alba".to_owned()),
        parent: Some("bl-epic".to_owned()),
        drones: vec![crate::reply::board::Drone {
            root_id: "20260830T051200Z-a1b2".to_owned(),
            name: "Cobalt".to_owned(),
        }],
        spend: Some(figure(Some("$1.50"), Some("over 2 conversations"))),
        rollup: Some(figure(None, None)),
        ..column("bl-1", "claimed")
    };
    let gated = crate::reply::board::BoardRow {
        gates: vec![crate::reply::board::Gate {
            id: "bl-gate".to_owned(),
            title: "the gate".to_owned(),
            mints: "close".to_owned(),
        }],
        ..column("bl-2", "gated")
    };
    Model {
        lookup: Some(crate::ui::Lookup::Board),
        columns: vec![crate::ui::Columns {
            channel: own().channel,
            board: crate::reply::board::Board {
                rows: vec![claimed, gated, column("bl-3", "ready")],
                fleet: vec![crate::reply::board::Fleet {
                    workspace: "home".to_owned(),
                    project: "lernie".to_owned(),
                    cap: 4,
                    count: 4,
                    room: false,
                    ceiling: Some("over budget".to_owned()),
                    label: "4/4 drones · tick 1m · last 30s ago".to_owned(),
                }],
            },
        }],
        bindings: vec![crate::ui::Bindings {
            channel: own().channel,
            rows: vec![
                crate::reply::balls::BallRow {
                    ball_id: "bl-1".to_owned(),
                    project: "lernie".to_owned(),
                    state: "bound".to_owned(),
                    title: Some("what bl-1 is for".to_owned()),
                    claimant: Some("alba".to_owned()),
                    workspace: Some("home".to_owned()),
                },
                crate::reply::balls::BallRow {
                    ball_id: "bl-3".to_owned(),
                    project: "lernie".to_owned(),
                    state: "ready".to_owned(),
                    title: None,
                    claimant: None,
                    workspace: None,
                },
            ],
        }],
        holding: Some(vec![bound("bl-1")]),
        marks: Some("balls/tasks".to_owned()),
        ..seated()
    }
}
