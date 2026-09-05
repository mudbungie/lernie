//! **The covering panes whose subject is EVERY CHANNEL** this box holds — the
//! decision queue, the window's own two, and the trail.
//!
//! Split from [`super`] on the seam DESIGN draws between §4.17/§4.18's panes
//! and §4.19/§4.21/§4.27's: these four are about nothing on the glass, so no
//! aim and no selection can invalidate them, and each is a union across
//! channels stamped with the channel each answer came down.

use crate::reply::convs::AgentState;
use crate::ui::Model;

use super::super::{own, seated};

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

/// One trail row: a command that ran, and where its alarm stands.
pub(crate) fn trailed(argv: &str, standing: &str) -> crate::reply::ops::OpRow {
    crate::reply::ops::OpRow {
        ts: "1700".to_owned(),
        origin: "balls".to_owned(),
        standing: standing.to_owned(),
        failed: standing != crate::reply::ops::CLEAN,
        exit_label: "exit 1".to_owned(),
        exit: 1,
        argv: argv.to_owned(),
        cwd: "/ws/home".to_owned(),
        stdout: String::new(),
        stderr: String::new(),
    }
}

/// **The seated model with the trail open and answered** (bl-4c48): an alarm
/// still standing with the child's own complaint under it, a handoff that is
/// not a failure, a clean run that says nothing, and a standing this build has
/// never seen. Every sentence the pane can say, on one screen.
pub(crate) fn trailing() -> Model {
    let standing = crate::reply::ops::OpRow {
        stderr: "the gate said no".to_owned(),
        ..trailed("bl close bl-1", "live")
    };
    let handed = crate::reply::ops::OpRow {
        origin: "conversation".to_owned(),
        exit_label: "detached — handed off, no exit to observe".to_owned(),
        exit: -2,
        failed: false,
        ..trailed("litany prompt c-1", "detached")
    };
    let grown = crate::reply::ops::OpRow {
        failed: false,
        ..trailed("bz login", "quarantined")
    };
    Model {
        lookup: Some(crate::ui::Lookup::Trailing),
        trails: vec![crate::ui::Trail {
            channel: own().channel,
            rows: vec![
                standing,
                handed,
                trailed("bl list", crate::reply::ops::CLEAN),
                grown,
            ],
        }],
        ..seated()
    }
}
