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
//! line, the counts of what it holds. Each compact step and the one line are
//! doors that open what they hide in place.
//!
//! **A toggle is remembered and the stages decide only where nobody toggled**:
//! the working's default is open while any of its rows is live, and each
//! step's default is its stage. Both are the toolkit's memory of a view state,
//! which no wire gesture corresponds to. Nothing is dropped at any stage —
//! every word is one or two clicks away, and the head counts what it hides.

use super::{Kind, Row, trail};
use crate::ui::theme::{self, Speaker, glyph};

/// The word on a settled working's one line, before its counts.
pub const WORKED: &str = "worked";

/// **The working's one line**: `worked · N calls · M notes` — the calls it
/// holds, and the notes, which are every row that is not a call: the reasoning
/// and the prose between. A result that met its call stands on the call's
/// row, and one whose call is not on this read is the end of a call still.
pub fn counted(steps: &[Row]) -> String {
    let calls = steps.iter().filter(|row| is_call(row)).count();
    let notes = steps.len() - calls;
    let sep = glyph::NOTE;
    format!(
        "{WORKED} {sep} {} {sep} {}",
        crate::render::counted(calls as u64, "call"),
        crate::render::counted(notes as u64, "note"),
    )
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
pub(super) fn working(ui: &mut egui::Ui, at: usize, steps: &[Row]) -> bool {
    let id = ui.make_persistent_id(("working", at));
    let live = steps.iter().any(|row| row.live);
    let mut open = ui.data(|d| d.get_temp::<bool>(id)).unwrap_or(live);
    let head = format!(
        "{} {}",
        if open { glyph::OPEN } else { glyph::SHUT },
        counted(steps)
    );
    // **The line is a door** (§4.45 item 6): a link at the body size, so it
    // lights under the pointer as every other clickable seat does.
    if theme::paint::tab(ui, &head, theme::INK_WEAK).clicked() {
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
                trail::step(ui, at + i, row, false);
            } else {
                finished(ui, at + i, row);
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
fn finished(ui: &mut egui::Ui, at: usize, row: &Row) {
    let id = ui.make_persistent_id(("step", at));
    let open = ui.data(|d| d.get_temp::<bool>(id)).unwrap_or(false);
    let toggled = if open {
        trail::step(ui, at, row, true)
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
    }
    .paint(ui)
}
