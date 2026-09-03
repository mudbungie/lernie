//! **The window's fixtures**: a model with something in every pane, and the two
//! ways a test looks at one.
//!
//! One home for them, because every pane's suite needs a roster to click in and
//! a conversation to select — and eight copies of a fixture is eight places a
//! field added to a row has to be filled in.

use crate::paint_probe::{self, frame::Window};
use crate::reply::convs::{AgentState, ConvRow, Tone};
use crate::reply::roles::RoleRow;
use crate::reply::roster::{WorkspaceKind, WsRow};
use crate::reply::transcript::{Entry, EntryKind, Transcript};
use crate::ui::{Aim, Channel, Chunk, Model, Tuning};

/// One workspace row, named and otherwise quiet.
pub(crate) fn wall(name: &str) -> WsRow {
    WsRow {
        workspace: name.to_owned(),
        kind: WorkspaceKind::Named,
        attention: 0,
        agents: 2,
        running: false,
        pinned: None,
    }
}

/// One conversation row at the top level.
pub(crate) fn conv(id: &str, display: &str) -> ConvRow {
    ConvRow {
        root_id: id.to_owned(),
        display: display.to_owned(),
        name: Some(display.to_owned()),
        state: AgentState::Quiescent,
        uncertain: false,
        preview: String::new(),
        age_secs: 42,
        attention: 0,
        members: 1,
        depth: 0,
        tone: Tone::Plain,
        failure: None,
    }
}

/// One role, tuned to the middle of everything: a level the pane has a seat
/// for, and the priority lane off.
pub(crate) fn role(name: &str) -> RoleRow {
    RoleRow {
        role: name.to_owned(),
        provider: "housevendor".to_owned(),
        model: "house-model-1".to_owned(),
        priority: false,
        effort: Some("medium".to_owned()),
    }
}

/// One delivered transcript entry.
pub(crate) fn said(sender: &str, body: &str) -> Entry {
    Entry {
        name: format!("001-{sender}.md"),
        raw: body.to_owned(),
        kind: EntryKind::Delivered {
            sender: sender.to_owned(),
            epitaph: None,
            body: body.to_owned(),
        },
    }
}

/// This box's own engine, holding one wall.
pub(crate) fn own() -> Chunk {
    Chunk {
        channel: Channel {
            name: "(this box's own engine)".to_owned(),
            named_there: None,
            dials: None,
        },
        held: crate::ui::Held::Heard,
        stale: None,
        growth: None,
        walls: vec![wall("home")],
    }
}

/// **A model with something in every pane**: one channel, one wall aimed at,
/// one conversation selected, and one thing said in it.
///
/// **It is standing on the conversation column** (bl-dfda), which the broad
/// shape does not read at all and the narrow one paints: a seat that has aimed
/// and selected is a seat looking at what it selected, and the column is where
/// the operator's own navigation is recorded.
pub(crate) fn seated() -> Model {
    Model {
        column: crate::ui::Column::Conversation,
        roster: vec![own()],
        convs: vec![conv("20260830T051200Z-a1b2", "port the paint probe")],
        transcript: Transcript {
            entries: vec![said("op", "port it")],
        },
        aim: Some(Aim {
            channel: "(this box's own engine)".to_owned(),
            address: "home".to_owned(),
        }),
        conversation: Some("20260830T051200Z-a1b2".to_owned()),
        ..Model::default()
    }
}

/// **The seated model with the tuning pane open and answered.** The pane's
/// own screen, and the one the `effort` and `priority` controls live on.
pub(crate) fn tuned() -> Model {
    Model {
        roles: Some(vec![role("worker"), role("compactor")]),
        tuning: Some(Tuning::Rows),
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
        auth_failed: false,
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
        auth_failed: true,
        auth_row: Some("housevendor".to_owned()),
        wound: "no_response".to_owned(),
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

/// Everything one idle frame of the whole window painted.
pub(crate) fn painted(model: &mut Model) -> String {
    Window::new().text(|ctx| crate::ui::render(ctx, model))
}

/// **What one idle frame of the whole window put ON THE GLASS**, each run
/// narrowed to what its clip rect let through.
///
/// [`painted`] answers what was laid out, which is a different question: a
/// panel emits the rows that overflow it and then clips them away, so a list
/// that does not scroll reads back complete from the text projection and is
/// half missing from the screen.
pub(crate) fn seen(window: &Window, body: impl FnMut(&egui::Context)) -> Vec<paint_probe::Seen> {
    paint_probe::seen_of(&window.frame(Vec::new(), body))
}

/// Everything one idle frame of one pane painted.
pub(crate) fn pane(body: impl FnMut(&mut egui::Ui)) -> String {
    paint_probe::paint(body)
}

/// **Click the seat reading exactly `label`.**
///
/// The coordinate comes off the painted glyphs, never off the string that went
/// in — a run the toolkit elided reads back whole, so a click aimed by input
/// text lands confidently on a seat whose painted text is not what it named.
///
/// A label the frame never painted **fails the test where it was asked for**,
/// rather than handing back a `false` every caller would have to remember to
/// check: a test that clicks a seat that is not on the glass is a broken test,
/// and the useful moment to say so is here, with the label in the message.
pub(crate) fn click(window: &Window, label: &str, mut body: impl FnMut(&egui::Context)) {
    let at = paint_probe::frame::locate_in(window, label, &mut body)
        .unwrap_or_else(|| panic!("nothing on the glass reads {label:?}"));
    paint_probe::frame::click(window, at, body);
}
