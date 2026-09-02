//! **The window's fixtures**: a model with something in every pane, and the two
//! ways a test looks at one.
//!
//! One home for them, because every pane's suite needs a roster to click in and
//! a conversation to select — and eight copies of a fixture is eight places a
//! field added to a row has to be filled in.

use crate::paint_probe::{self, frame::Window};
use crate::reply::convs::{AgentState, ConvRow, Tone};
use crate::reply::roster::{WorkspaceKind, WsRow};
use crate::reply::transcript::{Entry, EntryKind, Transcript};
use crate::ui::{Aim, Channel, Chunk, Model};

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
pub(crate) fn seated() -> Model {
    Model {
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
    let at = paint_probe::frame::locate_in(window, label, &mut body)
        .unwrap_or_else(|| panic!("nothing on the glass reads {label:?}"));
    paint_probe::frame::click(window, at, body);
}
