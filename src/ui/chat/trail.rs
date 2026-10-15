//! **The trail painted**: the fold's head, then each step on its stem
//! (DESIGN §4.42 item 4, §4.45 item 5; `docs/STYLE.md` §5, *a trail*).
//!
//! Split from [`super`] at the cap on the seam that module's doc draws: the
//! transcript is what was said and this is what was DONE between the saying —
//! the calls the model made, their results, and its reasoning — one group
//! under a fold, each step on a stem drawn as the roster draws its threading.
//!
//! **The fold's default is the turn's state** (§4.45 item 5): open while any
//! of its steps is live or the turn it belongs to still runs, shut once the
//! turn settled — the calls are read while they run and are bookkeeping once
//! they have. The one state kept here is the operator's own toggle, the
//! toolkit's memory of a view state; it is remembered as before, and the
//! default decides only where nobody toggled. Nothing is dropped: the head
//! still counts the calls, and a click opens it.

use super::{Fold, Kind, Row, TRAIL, fold};
use crate::ui::theme::{self, glyph};

/// **A trail**: the fold's head, then each step on its stem.
pub(super) fn trail(ui: &mut egui::Ui, at: usize, steps: &[Row]) {
    let id = ui.make_persistent_id(("trail", at));
    let live = steps.iter().any(|row| row.live);
    let mut open = ui.data(|d| d.get_temp::<bool>(id)).unwrap_or(live);
    // **The fold counts calls** (bl-eb4e), not the steps under it: a result
    // stands on its call's row, and reasoning is not a call.
    let calls = steps
        .iter()
        .filter(|row| matches!(row.kind, Kind::Call | Kind::Answered))
        .count();
    let head = format!(
        "{} {TRAIL} ({calls})",
        if open { glyph::OPEN } else { glyph::SHUT },
    );
    // **The head is a door** (§4.45 item 6): a link at the body size, so it
    // lights under the pointer as every other clickable seat does.
    if theme::paint::tab(ui, &head, theme::INK_WEAK).clicked() {
        open = !open;
        ui.data_mut(|d| d.insert_temp(id, open));
    }
    if !open {
        return;
    }
    let x = ui.cursor().min.x + theme::space::S;
    for (i, row) in steps.iter().enumerate() {
        let top = ui.cursor().min.y;
        ui.horizontal_top(|ui| {
            ui.add_space(theme::space::L);
            step(ui, at + i, row);
        });
        let bottom = ui.min_rect().max.y;
        let middle = top + theme::space::S;
        theme::paint::elbow(ui, x, top, middle, x + theme::space::S);
        if i + 1 < steps.len() {
            theme::paint::rail(ui, x, middle, bottom);
        }
    }
}

/// **One step**: its mark, its name — a call's with how it ended beside it
/// — and what it said, folded to its head where there is more.
fn step(ui: &mut egui::Ui, at: usize, row: &Row) {
    let red = theme::accent(theme::State::Error);
    let ended = |ui: &mut egui::Ui| {
        let (mark, ink) = if row.failed {
            (glyph::FAILED, red)
        } else {
            (glyph::DONE, theme::INK_WEAK)
        };
        ui.label(egui::RichText::new(mark).color(ink));
    };
    let (mark, ink) = match row.kind {
        Kind::Call | Kind::Answered => (glyph::CALL, theme::BRAND),
        _ => (glyph::NOTE, theme::INK_WEAK),
    };
    if row.kind == Kind::Result {
        ended(ui);
    } else {
        ui.label(egui::RichText::new(mark).color(ink));
    }
    ui.vertical(|ui| {
        let who = if row.failed { red } else { theme::INK_WEAK };
        ui.horizontal_wrapped(|ui| {
            ui.label(egui::RichText::new(&row.who).color(who));
            if row.kind == Kind::Answered {
                ended(ui);
            }
        });
        match &row.fold {
            Some(fold) => folded(ui, at, row, fold),
            None if row.said.is_empty() => {}
            None => {
                theme::paint::prose(ui, &row.said, theme::INK_WEAK);
            }
        }
    });
}

/// **A machine's answer, folded to its head** (bl-90d0): the one control
/// says what it hides and how much. It is egui's own button, which wears the
/// toolkit's hover (§4.45 item 6).
fn folded(ui: &mut egui::Ui, at: usize, row: &Row, fold: &Fold) {
    let id = ui.make_persistent_id(at);
    let mut open = ui.data(|d| d.get_temp::<bool>(id)).unwrap_or(false);
    if ui
        .button(if open { fold::UNFOLD } else { &fold.whole })
        .clicked()
    {
        open = !open;
        ui.data_mut(|d| d.insert_temp(id, open));
    }
    if open {
        theme::paint::prose(ui, &row.said, theme::INK);
    } else {
        theme::paint::prose(ui, &fold.head, theme::INK_WEAK);
    }
}
