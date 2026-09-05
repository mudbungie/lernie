//! **The covering panes' fixtures**: the seated model with each pane open and
//! answered, and the rows each needs.
//!
//! Split from [`super`] at the design-time budget on the seam that module's
//! own doc draws. Every one of them is `..seated()`, because a pane covers the
//! window at work rather than replacing it — and each is answered rather than
//! waiting, for the reason `crate::snapshot::worlds` states: what a pane's
//! world exists to photograph is every sentence it can say, and an unanswered
//! pane says only that nobody has answered.

use crate::reply::convs::AgentState;
use crate::ui::{Login, Model, Tuning};

use super::{own, role, seated};

/// **The seated model with the tuning pane open and answered.** The pane's
/// own screen, and the one the `effort` and `priority` controls live on.
pub(crate) fn tuned() -> Model {
    Model {
        roles: Some(vec![role("worker"), role("compactor")]),
        tuning: Some(Tuning::Rows),
        ..seated()
    }
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
        records: true,
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

/// One queue row, quiet: on the wall `seated` is aimed at, asking for nothing
/// anybody wrote down.
pub(crate) fn waiting(workspace: &str, agent: &str) -> crate::reply::queue::QueueRow {
    crate::reply::queue::QueueRow {
        workspace: workspace.to_owned(),
        agent: agent.to_owned(),
        display: agent.to_owned(),
        state: AgentState::Quiescent,
        uncertain: true,
        preview: String::new(),
        age_secs: 7,
        pending: 0,
        signals: Vec::new(),
        failure: None,
        flag: None,
        held: None,
    }
}

/// **The seated model with the decision queue open and answered** (bl-f0ef):
/// a flagged row carrying every line the pane can hang off one — the raise,
/// the failure clause, the parked invocation and the signals — a quiet row,
/// and a row on a wall this seat holds no name for. Every sentence the pane
/// can say, on one screen.
pub(crate) fn queued() -> Model {
    let raised = crate::reply::queue::QueueRow {
        display: "port the paint probe".to_owned(),
        state: AgentState::Stopped,
        uncertain: false,
        preview: "it stopped on the third attempt".to_owned(),
        age_secs: 5,
        pending: 2,
        signals: vec!["held".to_owned(), "mail".to_owned(), "flagged".to_owned()],
        failure: Some("Unauthorized".to_owned()),
        flag: Some(crate::reply::queue::Flag {
            at: "2026-09-01T22:10Z".to_owned(),
            reason: "it is rewriting an unrelated crate".to_owned(),
        }),
        held: Some(crate::reply::queue::Held {
            tool: "Bash".to_owned(),
            tool_use: "toolu_1".to_owned(),
            reason: "writes".to_owned(),
        }),
        ..waiting("home", "20260830T051200Z-a1b2")
    };
    Model {
        queue: true,
        waiting: vec![crate::ui::Asking {
            channel: own().channel,
            rows: vec![raised, waiting("home", "c-2"), waiting("elsewhere", "c-3")],
        }],
        ..seated()
    }
}

/// One help row, in the classification that owes a control.
pub(crate) fn helped(verb: &str, surface: &str) -> crate::reply::help::HelpRow {
    crate::reply::help::HelpRow {
        verb: verb.to_owned(),
        usage: format!("/{verb} <workspace>"),
        summary: format!("what {verb} is for"),
        detail: format!("the page for {verb}, at the length a page runs to"),
        surface: surface.to_owned(),
    }
}

/// **The seated model with the commands pane open and answered** (bl-40ec):
/// an op every seat owes a control and one spoken by programs, which is every
/// sentence a row can carry.
pub(crate) fn commanded() -> Model {
    Model {
        lookup: Some(crate::ui::Lookup::Commands),
        pages: vec![crate::ui::Pages {
            channel: own().channel,
            rows: vec![
                helped("message", "control"),
                helped("invocations", "machine"),
            ],
        }],
        ..seated()
    }
}

/// One hit, in a conversation on a wall the engine names by its own path —
/// which is the whole of yog bl-ef16 in one value.
pub(crate) fn hit(at: &str) -> crate::reply::search::Hit {
    crate::reply::search::Hit {
        at: at.to_owned(),
        field: "summary".to_owned(),
        excerpt: "the gate said no".to_owned(),
        offset: 12,
        project: None,
        id: None,
        workspace: Some("/ws/home".to_owned()),
        agent: Some("20260830T051200Z-a1b2".to_owned()),
    }
}

/// **The seated model with the find pane open and answered** (bl-40ec): a
/// needle already spent, one hit, and a subject the engine could not read —
/// every sentence the pane can say, on one screen.
pub(crate) fn finding() -> Model {
    Model {
        lookup: Some(crate::ui::Lookup::Finding),
        needle: "gate".to_owned(),
        found: vec![crate::ui::Hits {
            channel: own().channel,
            found: crate::reply::search::Found {
                needle: "gate".to_owned(),
                rows: vec![hit("conversation")],
                unreadable: vec!["p: balls unlistable".to_owned()],
            },
        }],
        ..seated()
    }
}
