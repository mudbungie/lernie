//! **One step of the working, whole** (DESIGN §4.42 item 4, §4.45 item 5;
//! `docs/STYLE.md` §5, *a block*): its mark, its name — a call's with how it
//! ended beside it — and what it said, a machine's answer folded to its head.
//!
//! Split from [`super`] at the cap on the seam that module's doc draws: the
//! transcript is what was said and this is what was DONE between the saying.
//! Which stage a step is in — whole, compact, or hidden under the working's
//! one line — is [`super::working`]'s decision; this paints the whole step it
//! is handed, which is the step in flight and the finished step an operator
//! opened. An intermediate prose stands whole on its own fill (§4.45 item 7),
//! inside the working's: who spoke is still the fill, one rung up from the
//! trail it stands on.

use super::{Fold, Kind, Row, code, fold};
use crate::ui::theme::{self, Speaker, glyph};

/// **One step, whole.** With `door`, its name is the seat that shuts it
/// again, and the answer is whether that seat was clicked; without, the name
/// is a label and the answer is never.
pub(super) fn step(ui: &mut egui::Ui, at: usize, row: &Row, door: bool) -> bool {
    if row.kind == Kind::Said {
        let mut shut = false;
        theme::paint::bubbled(ui, theme::Bubble::Intermediate, |ui| {
            shut = bare(ui, at, row, door);
        });
        return shut;
    }
    bare(ui, at, row, door)
}

/// The step without its fill.
fn bare(ui: &mut egui::Ui, at: usize, row: &Row, door: bool) -> bool {
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
        Kind::Said if row.weight == Speaker::Model => (glyph::MODEL, theme::INK_WEAK),
        _ => (glyph::NOTE, theme::INK_WEAK),
    };
    if row.kind == Kind::Result {
        ended(ui);
    } else {
        ui.label(egui::RichText::new(mark).color(ink));
    }
    let mut shut = false;
    ui.vertical(|ui| {
        let who = match row.kind {
            _ if row.failed => red,
            Kind::Said => theme::speaker(row.weight),
            _ => theme::INK_WEAK,
        };
        ui.horizontal_wrapped(|ui| {
            if door {
                shut = theme::paint::tab(ui, &row.who, who).clicked();
            } else {
                ui.label(egui::RichText::new(&row.who).color(who));
            }
            if row.kind == Kind::Answered {
                ended(ui);
            }
        });
        match &row.fold {
            Some(fold) => folded(ui, at, row, fold),
            None if row.said.is_empty() => {}
            // Prose is what was said, with its fences; a step's words are
            // its machinery, in the weak ink.
            None if row.kind == Kind::Said => code::paint(ui, at, &row.said, theme::INK),
            None => {
                theme::paint::prose(ui, &row.said, theme::INK_WEAK);
            }
        }
    });
    shut
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
