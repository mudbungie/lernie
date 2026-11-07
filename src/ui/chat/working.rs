//! **A turn's working, in three stages** (DESIGN §4.45 item 5; `docs/STYLE.md`
//! §5, *a block*): verbose while it happens, compact once it is done, one line
//! once the answer stands.
//!
//! Split from [`super::trail`] on the seam the design draws: that file paints
//! ONE step whole, and this decides which stage each step and the whole
//! working are in — every row between the operator's turn and its final prose
//! (the calls, their results, the reasoning and the prose between), gathered
//! by the pane into one block on the trail's bubble.
//!
//! **What a step shows is decided by its state, never by its size** (bl-90d0's
//! rule kept). A step in flight — a call without its result, the reasoning
//! streaming at the end of the running turn, or **the model's own prose
//! anywhere in the running turn** (bl-2101: *printed directly*, standing
//! whole until the turn settles, whatever steps follow it) — stands whole. A
//! finished step is compact — its sub-bubble one line tall (DESIGN §4.48
//! item 2): a call is its call line with its exit badge; a finished
//! thinking, or an intermediate prose once its turn settled, is its first
//! line elided. A settled turn's working is its bubble one line tall, the
//! counts of what it hides by kind. Each compact step and the one line are
//! doors that open what they hide in place.
//!
//! **The working is one bubble and each step a sub-bubble inside it**: the
//! pane lays the working on the ground inside a hairline with `XXS` of air,
//! and this lays each step on `SURFACE` with none, so a step's line — a row
//! — fills its sub-bubble exactly and the eye reads *this is one step of
//! that working*.
//!
//! **A toggle is remembered and the stages decide only where nobody toggled**:
//! the working's default is open while any of its rows is live, and each
//! step's default is its stage. Both are the toolkit's memory of a view state,
//! which no wire gesture corresponds to — and a memory of ONE conversation's
//! row: the ids here are keyed by the row's index under the conversation's id
//! that [`super::render`] pushes (bl-b83d), so what the operator opened in one
//! conversation says nothing about the same row of another. Nothing is dropped at any stage —
//! every word is one or two clicks away, and the head counts what it hides.

use super::{Kind, Row, trail};
use crate::settings::{Settings, Toggle};
use crate::ui::theme::{self, Bubble, Speaker, glyph, space};
use trail::is_call;

/// What the working's line does, each way, and what a step is. The line is
/// the one way back to what a settled turn hid, so its sentence says so.
pub const UNFOLD_HELP: &str = "opens what the turn did — every step it hid: the calls, their results, the thoughts and the prose between";
pub const FOLD_HELP: &str = "folds the working back to its one line";

/// **The working's one line** (bl-db54): what it hides, counted by kind in a
/// fixed order — `tool calls`, `thoughts`, `intermediate responses`, `notes`
/// — with a kind it holds none of left out, parted by the note glyph. A
/// result that met its call stands on the call's row, and one whose call is
/// not on this read is the end of a call still; the model's own prose
/// between is an intermediate response, and every other row said inside the
/// working — a peer's, a compaction — is a note.
pub fn counted(steps: &[Row]) -> String {
    ["tool call", "thought", "intermediate response", "note"]
        .into_iter()
        .enumerate()
        .map(|(kind, noun)| (steps.iter().filter(|row| sort(row) == kind).count(), noun))
        .filter(|(count, _)| *count > 0)
        .map(|(count, noun)| crate::render::counted(count as u64, noun))
        .collect::<Vec<_>>()
        .join(&format!(" {} ", glyph::NOTE))
}

/// Which of the line's four kinds a row is, by its place in the line.
fn sort(row: &Row) -> usize {
    if is_call(row) {
        0
    } else if row.kind == Kind::Thinking {
        1
    } else if is_response(row) {
        2
    } else {
        3
    }
}

/// The model's own prose between its calls.
fn is_response(row: &Row) -> bool {
    row.kind == Kind::Said && row.weight == Speaker::Model
}

/// **Whether a step stands whole**: a call with no result yet; and in the
/// turn that runs, the model's own prose wherever it stands (§4.45 item 5,
/// the 2026-10-31 ruling) and the reasoning while it is the last row — still
/// streaming. A call that returned is compact the moment it did, last or
/// not: *once the tool call is done, it should collapse*.
fn in_flight(row: &Row, last: bool) -> bool {
    row.kind == Kind::Call
        || (row.live && (is_response(row) || (last && row.kind == Kind::Thinking)))
}

/// **The working as one block**: its line as a door, then — while open —
/// each step on its sub-bubble, whole or compact by its stage. Answers
/// whether it stands open, which is what decides whether the `response`
/// elbow after it is painted at all.
///
/// **With the `fold` knob off nothing folds itself** (DESIGN §4.46 item 4):
/// the working and each finished step stand whole until clicked shut.
pub(super) fn working(ui: &mut egui::Ui, at: usize, steps: &[Row], settings: &Settings) -> bool {
    let id = ui.make_persistent_id(("working", at));
    let live = steps.iter().any(|row| row.live) || !settings.on(Toggle::Fold);
    let mut open = ui.data(|d| d.get_temp::<bool>(id)).unwrap_or(live);
    // **The line is a door the width of the bubble** (§4.45 item 6;
    // bl-db54): the row idiom, so the whole width is its seat, it lights
    // under the pointer across it and it holds the keyboard.
    let door = theme::paint::Row {
        mark: (
            if open { glyph::OPEN } else { glyph::SHUT }.to_owned(),
            theme::INK_WEAK,
        ),
        words: counted(steps),
        ink: theme::INK_WEAK,
        state: None,
        chosen: false,
        indent: 0.0,
        aside: None,
        beneath: Vec::new(),
        tail: None,
        door: None,
        help: if open { FOLD_HELP } else { UNFOLD_HELP }.to_owned(),
    };
    if door.paint(ui).clicked() {
        open = !open;
        ui.data_mut(|d| d.insert_temp(id, open));
    }
    if !open {
        return false;
    }
    let bubbles = settings.on(Toggle::Bubbles);
    for (i, row) in steps.iter().enumerate() {
        ui.add_space(space::XXS);
        let last = i + 1 == steps.len();
        theme::paint::nested(ui, bubbles.then_some(Bubble::Step), 0.0, |ui| {
            if in_flight(row, last) {
                trail::step(ui, at + i, row, false, bubbles);
            } else {
                finished(ui, at + i, row, settings);
            }
        });
    }
    true
}

/// **A finished step**: compact by default, whole once opened, and a door
/// either way — its line opens it compact and shuts it whole.
fn finished(ui: &mut egui::Ui, at: usize, row: &Row, settings: &Settings) {
    let id = ui.make_persistent_id(("step", at));
    let open = ui
        .data(|d| d.get_temp::<bool>(id))
        .unwrap_or(!settings.on(Toggle::Fold));
    let toggled = if open {
        trail::step(ui, at, row, true, settings.on(Toggle::Bubbles))
    } else {
        trail::line(row, false).paint(ui).clicked()
    };
    if toggled {
        ui.data_mut(|d| d.insert_temp(id, !open));
    }
}
