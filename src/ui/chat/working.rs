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
//! rule kept). A step in flight — a call without its result, or the last row
//! of the running turn, whatever it is — stands whole. A finished step is
//! compact: a call is its call line and ONE line of its output elided with `…`
//! at the width it has, a preview and not the content; a finished thinking or
//! intermediate prose is one elided line. A settled turn's working is one
//! line, the counts of what it hides by kind. Each compact step and the one line are
//! doors that open what they hide in place.
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
use crate::ui::theme::{self, Speaker, glyph};

/// What the working's line does, each way, and what a step is. The line is
/// the one way back to what a settled turn hid, so its sentence says so.
pub const UNFOLD_HELP: &str = "opens what the turn did — every step it hid: the calls, their results, the thoughts and the prose between";
pub const FOLD_HELP: &str = "folds the working back to its one line";
pub const STEP_HELP: &str = "one step of the working; its result stands elided beneath";

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

fn is_call(row: &Row) -> bool {
    matches!(row.kind, Kind::Call | Kind::Answered | Kind::Result)
}

/// **Whether a step is still happening**: a call with no result yet, or the
/// last row of the turn that runs — the output arriving, the prose streaming.
fn in_flight(row: &Row, last: bool) -> bool {
    row.kind == Kind::Call || (row.live && last)
}

/// **The working as one block**: its line as a door, then — while open —
/// each step on its stem, whole or compact by its stage. Answers whether it
/// stands open, which is what decides whether the `response` elbow after it
/// is painted at all.
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
    let x = ui.cursor().min.x + theme::space::S;
    for (i, row) in steps.iter().enumerate() {
        let top = ui.cursor().min.y;
        let last = i + 1 == steps.len();
        ui.horizontal_top(|ui| {
            ui.add_space(theme::space::L);
            if in_flight(row, last) {
                trail::step(ui, at + i, row, false, settings.on(Toggle::Bubbles));
            } else {
                finished(ui, at + i, row, settings);
            }
        });
        let bottom = ui.min_rect().max.y;
        let middle = top + theme::space::S;
        theme::paint::elbow(ui, x, top, middle, x + theme::space::S);
        if !last {
            theme::paint::rail(ui, x, middle, bottom);
        }
    }
    true
}

/// **A finished step**: compact by default, whole once opened, and a door
/// either way — the compact row opens it, the name on the whole step shuts it.
fn finished(ui: &mut egui::Ui, at: usize, row: &Row, settings: &Settings) {
    let id = ui.make_persistent_id(("step", at));
    let open = ui
        .data(|d| d.get_temp::<bool>(id))
        .unwrap_or(!settings.on(Toggle::Fold));
    let toggled = if open {
        trail::step(ui, at, row, true, settings.on(Toggle::Bubbles))
    } else {
        compact(ui, row).clicked()
    };
    if toggled {
        ui.data_mut(|d| d.insert_temp(id, !open));
    }
}

/// **A finished step, compact** — the row idiom, which is one seat that
/// lights under the pointer and elides at the width it has. A call is its
/// line, marked with how it ended, and one line of its output beneath; a
/// thinking or an intermediate prose is its name and its first line in one.
fn compact(ui: &mut egui::Ui, row: &Row) -> egui::Response {
    let red = theme::accent(theme::State::Error);
    let first = row.said.lines().next().unwrap_or_default();
    let weak = theme::INK_WEAK;
    let (mark, words, beneath) = if is_call(row) {
        let mark = if row.failed {
            (glyph::FAILED, red)
        } else {
            (glyph::DONE, weak)
        };
        let beneath = if first.is_empty() {
            Vec::new()
        } else {
            vec![(first.to_owned(), weak, egui::TextStyle::Body)]
        };
        (mark, row.who.clone(), beneath)
    } else {
        let mark = if row.weight == Speaker::Model && row.kind == Kind::Said {
            glyph::MODEL
        } else {
            glyph::NOTE
        };
        let words = format!("{} {} {first}", row.who, glyph::NOTE);
        ((mark, weak), words, Vec::new())
    };
    theme::paint::Row {
        mark: (mark.0.to_owned(), mark.1),
        words,
        ink: if row.failed { red } else { weak },
        state: row.failed.then_some(red),
        chosen: false,
        indent: 0.0,
        aside: None,
        beneath,
        door: None,
        help: STEP_HELP.to_owned(),
    }
    .paint(ui)
}
