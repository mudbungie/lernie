//! **The fleet pane's fixtures** (bl-a43a) — the thirteenth covering pane, and
//! the fifth whose subject is the aimed wall.
//!
//! Its own file rather than [`super::board`]'s neighbour, on the seam DESIGN
//! draws: that pane is two widths and this one is one. What they share is the
//! wall they are aimed at, which is [`super::super::seated`]'s and is imported
//! from there rather than rebuilt.

use crate::ui::Model;

use super::super::seated;

/// One diff row, in the state named and as quiet as a row can be.
pub(crate) fn diff(ball: &str, state: &str) -> crate::reply::diff::Diff {
    crate::reply::diff::Diff {
        project: "lernie".to_owned(),
        ball_id: ball.to_owned(),
        handle: None,
        delivered: None,
        state: state.to_owned(),
        target: None,
        source: None,
        target_oid: None,
        source_oid: None,
        missing: Vec::new(),
        files: Vec::new(),
        truncated: None,
    }
}

/// One attempt, in the outcome named and carrying only what every row has.
pub(crate) fn attempt(ball: &str, outcome: &str) -> crate::reply::science::Attempt {
    crate::reply::science::Attempt {
        diff: diff(ball, "unreadable"),
        base: None,
        conversation: None,
        goal: None,
        governing: None,
        response: None,
        pins: Vec::new(),
        usage: crate::reply::science::Usage {
            input: 11,
            output: 22,
            cache_read: 33,
            cache_write: 44,
        },
        wall_secs: 90,
        steps: 4,
        verdicts: Vec::new(),
        compacted: None,
        outcome: crate::reply::science::Ending {
            state: outcome.to_owned(),
            commit: None,
            by: None,
        },
    }
}

/// **The seated model with the fleet pane open and answered** (bl-a43a): a
/// receipt in the op's own name, an attempt carrying every line the pane can
/// hang off one, a bare attempt, a changed row with both churn shapes on it,
/// and a row whose ref is not there yet. Every sentence the pane can say, on
/// one screen.
pub(crate) fn fleeting() -> Model {
    let whole = crate::reply::science::Attempt {
        base: Some("f00dbeef".to_owned()),
        conversation: Some("20260830T051200Z-a1b2".to_owned()),
        goal: Some("ship it".to_owned()),
        governing: Some("deadbeef".to_owned()),
        response: Some("done, tests green".to_owned()),
        pins: vec!["instructions/00-AGENTS.md".to_owned()],
        verdicts: vec![crate::reply::science::Verdict {
            sender: "judge-one".to_owned(),
            body: "candidate B reads cleaner".to_owned(),
        }],
        compacted: Some(12),
        outcome: crate::reply::science::Ending {
            state: "accepted".to_owned(),
            commit: Some("ccc".to_owned()),
            by: Some("at-0badcafe".to_owned()),
        },
        ..attempt("bl-1", "accepted")
    };
    let moved = crate::reply::diff::Diff {
        source: Some("work/bl-3".to_owned()),
        target: Some("main".to_owned()),
        handle: Some("at-0badcafe".to_owned()),
        delivered: Some("ccc".to_owned()),
        truncated: Some(true),
        files: vec![
            crate::reply::diff::Churn {
                path: "src/a.rs".to_owned(),
                added: Some(3),
                removed: Some(1),
                binary: None,
            },
            crate::reply::diff::Churn {
                path: "assets/x.png".to_owned(),
                added: None,
                removed: None,
                binary: Some(true),
            },
        ],
        ..diff("bl-3", "diff")
    };
    let gone = crate::reply::diff::Diff {
        source: Some("work/bl-2".to_owned()),
        target: Some("main".to_owned()),
        missing: vec!["work/bl-2".to_owned()],
        ..diff("bl-2", "absent")
    };
    Model {
        fleet: Some(crate::ui::Fleet {
            at: crate::ui::Aim {
                channel: "(this box's own engine)".to_owned(),
                address: "home".to_owned(),
            },
            project: "lernie".to_owned(),
            cap: 4,
            model: "claude-haiku-4-5".to_owned(),
            spread: 3,
            goal: String::new(),
            summary: String::new(),
            said: Some(crate::ui::Armed {
                op: "fleet".to_owned(),
                armed: true,
            }),
        }),
        attempts: Some(vec![whole, attempt("bl-9", "pending")]),
        work: Some(vec![moved, gone]),
        ..seated()
    }
}
