//! **The covering panes whose subject is a FOCUS**: the aimed wall, one of its
//! rows, or the selected conversation.
//!
//! Split from [`super`] at the design-time budget, and split again from
//! [`union`] at it on the seam DESIGN itself draws (bl-4c48): a pane about a
//! focus is retired when that focus moves, and a pane about every channel has
//! no focus to move. The two halves change for different reasons — the first
//! when a row grows a fact, the second when a channel-wide op lands a surface.
//!
//! Every one of them is `..seated()`, because a pane covers the window at work
//! rather than replacing it — and each is answered rather than waiting, for
//! the reason `crate::snapshot::worlds` states: what a pane's world exists to
//! photograph is every sentence it can say, and an unanswered pane says only
//! that nobody has answered.

/// The covering panes whose subject is every channel this box holds.
pub(crate) mod union;

pub(crate) use union::{commanded, finding, helped, hit, queued, trailed, trailing, waiting};

use crate::ui::{Login, Model, Tuning};

use super::{role, seated};

/// **The seated model with the tuning pane open and answered.** The pane's
/// own screen, and the one the `effort` and `priority` controls live on.
pub(crate) fn tuned() -> Model {
    Model {
        roles: Some(vec![role("worker"), role("compactor")]),
        tuning: Some(Tuning::Rows),
        ..seated()
    }
}

/// **The seated model with the aimed wall pinned** (bl-7782) — the one screen
/// the `unpin` control exists on, because the pin pair are assertions and each
/// row carries exactly the one that is not already true of it.
pub(crate) fn pinned() -> Model {
    let mut model = seated();
    for chunk in &mut model.roster {
        for row in &mut chunk.walls {
            row.pinned = Some(0);
        }
    }
    model
}

/// One provider row, open to a sign-in and taking both tuning knobs.
pub(crate) fn provider(name: &str) -> crate::reply::providers::ProviderRow {
    crate::reply::providers::ProviderRow {
        name: name.to_owned(),
        fact: "credential present".to_owned(),
        blocked: None,
        effort: true,
        priority: true,
    }
}

/// **The seated model with the login pane open and answered** (bl-e3c5): a row
/// that can be signed in to and is being followed mid-flow, a row the engine
/// has blocked, and a row asked what it offers — every sentence the pane can
/// say about a row, on one screen.
pub(crate) fn signing() -> Model {
    let blocked = crate::reply::providers::ProviderRow {
        fact: "no credential".to_owned(),
        blocked: Some("no login flow".to_owned()),
        effort: false,
        priority: false,
        ..provider("otherhouse")
    };
    Model {
        login: Some(Login {
            following: Some("housevendor".to_owned()),
            asking: Some("otherhouse".to_owned()),
        }),
        providers: Some(vec![provider("housevendor"), blocked]),
        offered: Some(vec!["house-model-1".to_owned()]),
        signin: Some(crate::reply::login::Signin {
            lines: vec![
                crate::reply::login::Line {
                    text: "open https://provider.invalid/auth".to_owned(),
                    err: true,
                },
                crate::reply::login::Line {
                    text: "waiting for the browser".to_owned(),
                    err: false,
                },
            ],
            outcome: None,
            fallback: None,
        }),
        ..seated()
    }
}

/// One machine, connected or not, offering one tool that takes a caller-named
/// directory and one that does not.
pub(crate) fn machine(client: &str, present: bool) -> crate::reply::clients::ClientRow {
    crate::reply::clients::ClientRow {
        client: client.to_owned(),
        present,
        tools: vec![
            crate::reply::clients::ToolRow {
                name: "Bash".to_owned(),
                description: "run a command".to_owned(),
                subject_cwd: false,
            },
            crate::reply::clients::ToolRow {
                name: "bash".to_owned(),
                description: "run a command in the conversation's cwd".to_owned(),
                subject_cwd: true,
            },
        ],
    }
}

/// **The seated model with the clients pane open and answered** (bl-e53c): a
/// machine connected right now with both consents on its set, one that is not
/// connected, and one that has advertised nothing — every sentence the pane
/// can say about a row, on one screen.
pub(crate) fn machines() -> Model {
    let bare = crate::reply::clients::ClientRow {
        tools: Vec::new(),
        ..machine("phone", false)
    };
    Model {
        listing: Some(crate::ui::Listing::Clients),
        machines: Some(vec![machine("laptop", true), machine("desk", false), bare]),
        ..seated()
    }
}

/// One step, complete and quiet — the row with nothing to complain about.
pub(crate) fn step(seq: &str) -> crate::reply::steps::StepRow {
    crate::reply::steps::StepRow {
        seq: seq.to_owned(),
        framing: "complete".to_owned(),
        attempts: 1,
        tokens: crate::reply::steps::Spend {
            input: 11,
            output: 22,
            cache_read: 33,
            cache_write: 44,
            total: 99,
        },
        commit: Some("abcdef1".to_owned()),
        started_at: Some("2026-08-30T05:12Z".to_owned()),
        ended_at: Some("2026-08-30T05:14Z".to_owned()),
        auth_row: None,
        wound: crate::reply::steps::NONE.to_owned(),
        wound_reason: None,
    }
}

/// **The seated model with the records pane open and answered** (bl-2cf7):
/// a quiet step and a wounded one, and a walked worktree with work landing
/// elsewhere — every sentence the pane can say, on one screen.
pub(crate) fn recorded() -> Model {
    let wounded = crate::reply::steps::StepRow {
        seq: "002".to_owned(),
        framing: "failed".to_owned(),
        attempts: 2,
        commit: None,
        started_at: None,
        ended_at: None,
        auth_row: Some("housevendor".to_owned()),
        wound: crate::reply::steps::REFUSED.to_owned(),
        wound_reason: Some("no bytes".to_owned()),
        ..step("002")
    };
    Model {
        listing: Some(crate::ui::Listing::Records),
        steps: Some(crate::reply::steps::Steps {
            rows: vec![step("001"), wounded],
            orphan: "mail".to_owned(),
            orphan_reason: Some("driver died".to_owned()),
        }),
        files: Some(crate::reply::files::Files {
            listing: Some(crate::reply::files::Listing {
                rows: vec![
                    crate::reply::files::FileRow {
                        path: "src".to_owned(),
                        size: 0,
                        dir: true,
                    },
                    crate::reply::files::FileRow {
                        path: "src/a.rs".to_owned(),
                        size: 12,
                        dir: false,
                    },
                ],
                truncated: true,
            }),
            preview: None,
            working_dir: Some("/home/u/elsewhere".to_owned()),
        }),
        ..seated()
    }
}
