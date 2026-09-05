//! **The window's fixtures**: a model with something in every pane, and the two
//! ways a test looks at one.
//!
//! One home for them, because every pane's suite needs a roster to click in and
//! a conversation to select — and eight copies of a fixture is eight places a
//! field added to a row has to be filled in.
//!
//! **The covering panes' own fixtures are [`panes`]**, split at the
//! design-time budget on the seam this doc already draws: here is the window
//! at work and the rows it is built from, there is the window with one pane
//! standing over it. The first changes when a reply row grows a field; the
//! second when a pane lands.

use crate::paint_probe::{self, frame::Window};
use crate::reply::convs::{AgentState, ConvRow, Tone};
use crate::reply::roles::RoleRow;
use crate::reply::roster::{WorkspaceKind, WsRow};
use crate::reply::transcript::{Entry, EntryKind, Transcript};
use crate::ui::{Aim, Channel, Chunk, Model};

/// The covering panes' own fixtures — a model with each one open and answered.
pub(crate) mod panes;

pub(crate) use panes::{
    attempt, boarded, clearing, column, commanded, configured, deposit, diff, drilled, figure,
    finding, fleeting, helped, hit, lineage, machine, machines, notch, own_row, pinned, provider,
    queued, recorded, signing, step, trailed, trailing, tuned, waiting,
};

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
    let at = aim(window, label, &mut body);
    paint_probe::frame::click(window, at, body);
}

/// **Secondary-click the seat reading exactly `label`** — the gesture that
/// opens a row's context menu, aimed the same way and failing the same way.
pub(crate) fn right_click(window: &Window, label: &str, mut body: impl FnMut(&egui::Context)) {
    let at = aim(window, label, &mut body);
    paint_probe::frame::secondary(window, at, body);
}

/// Where a click on `label` lands, or a failure naming the label that is not
/// on the glass.
fn aim(window: &Window, label: &str, body: impl FnMut(&egui::Context)) -> egui::Pos2 {
    paint_probe::frame::locate_in(window, label, body)
        .unwrap_or_else(|| panic!("nothing on the glass reads {label:?}"))
}
