//! **The board**: every live ball in its column, what each has cost, and the
//! loops running them (bl-d2af; yog's `docs/REMOTE.md` §9.7, VISION §5 V4).
//!
//! # The twelfth covering pane, and the one that says what the fleet is FOR
//!
//! The window painted workspaces, conversations and a transcript, and nothing
//! about the task store those conversations are working — so an operator could
//! watch work happen and could not see the ball it was happening on. Four ops
//! answer that and this pane is all four: `board` folds the store into
//! columns, `balls` is the whole box's binding table under it, and
//! `workspace-balls` and `marks` are the aimed wall's own half
//! (`crate::ui::board::wall`).
//!
//! # Two widths, one pane, and the wire is what decided that
//!
//! `balls` and `board` name no workspace, so they fan and their sections are
//! the union across channels — the decision queue's shape
//! (`crate::ui::queue`). `workspace-balls` and `marks` name one, so they are
//! the tuning pane's. Splitting them into two panes would put *the board* and
//! *this wall's balls* on two screens, which is one question; keeping them
//! here costs one section that says what it is about.
//!
//! # The loop's own facts are here because there is nowhere else
//!
//! There is no `fleet` read on this wire and no reply kind for one. What an
//! armed loop is doing — how full, how often it looks, when it last acted —
//! rides on the `board` answer, so the readable half of the fleet is this
//! pane's whether or not the fleet's CONTROLS ever land here (bl-a43a).
//!
//! # What is painted is words, computed beside the paint
//!
//! Every line comes off a pure function of the row, so the suite reads the
//! sentence rather than the layout — `crate::ui::records`' own rule. Every
//! token rides verbatim (`crate::reply` rung 3): a column, a state or a badge
//! this build has never seen paints as itself, and the money is upstream's own
//! rendering rather than a sum computed here.

use crate::reply::board::BoardRow;
use crate::ui::{Model, theme};

pub use rows::{binding, cost, gated, headline, held, placed, running, worked};

/// The words a row wears, each a pure function of it.
mod rows;
/// The aimed wall's half of the pane.
pub mod wall;

/// The word that opens the pane. It hangs off the roster, above the channels.
pub const OPEN: &str = "the board…";
/// The word that closes it.
pub const CLOSE: &str = "done";
/// The pane's own heading.
pub const HEADING: &str = "the board";
/// What it says before any channel has answered.
pub const NOT_ANSWERED: &str = "waiting to hear what is on the board";
/// What it says once they have and nothing is on it. A fact about the box, and
/// the one empty state here that is not a wait.
pub const NOTHING: &str = "no ball is on any board here";
/// The heading over the whole box's binding table.
pub const BINDINGS: &str = "every ball⇄workspace binding";

/// Paint the pane and take the clicks on it. Answers whether there was one to
/// paint, so the shell knows whether the conversation still stands.
pub fn render(ui: &mut egui::Ui, model: &mut Model) -> bool {
    if !model.boarding() {
        return false;
    }
    ui.heading(HEADING);
    ui.horizontal_wrapped(|ui| {
        if ui.button(CLOSE).clicked() {
            model.close_lookup();
        }
    });
    ui.separator();
    let columns = model.columns.clone();
    let bindings = model.bindings.clone();
    // One scroll for every section, and the heading above it fixed — the shape
    // every pane here keeps, for the reason the tuning pane states: a pane cut
    // off mid-row says nothing about having been cut.
    egui::ScrollArea::vertical()
        .id_salt(HEADING)
        .auto_shrink(false)
        .show(ui, |ui| {
            // **Each emptiness is its own sentence and none of them speaks for
            // another** — the union's two here, and the aimed wall's four in
            // its own section. A pane that answered one sentence for both
            // widths would say *nothing is on any board* about a box whose
            // channels have not answered and a wall it never asked.
            if columns.is_empty() && bindings.is_empty() {
                ui.label(NOT_ANSWERED);
            } else if empty(&columns, &bindings) {
                ui.label(NOTHING);
            }
            for section in &columns {
                // **A section that answered nothing is silent, not a
                // sentence** — the queue's own rule, on a pane that is the
                // union across channels.
                if section.board.rows.is_empty() && section.board.fleet.is_empty() {
                    continue;
                }
                ui.separator();
                ui.label(crate::ui::roster::header(&section.channel));
                for loop_ in &section.board.fleet {
                    ui.colored_label(theme::NOTICE, running(loop_));
                }
                for row in &section.board.rows {
                    live(ui, row);
                }
            }
            wall::render(ui, model);
            for section in &bindings {
                if section.rows.is_empty() {
                    continue;
                }
                ui.separator();
                ui.label(format!(
                    "{}  —  {BINDINGS}",
                    crate::ui::roster::header(&section.channel)
                ));
                for row in &section.rows {
                    ui.label(binding(row));
                }
            }
        });
    true
}

/// **Whether every channel that answered answered nothing** — the one state
/// that is a fact about the box rather than a wait. The aimed wall is not in
/// it: that section keeps its own four sentences, and folding them together
/// would let one wall's emptiness speak for every channel.
fn empty(columns: &[crate::ui::Columns], bindings: &[crate::ui::Bindings]) -> bool {
    columns
        .iter()
        .all(|section| section.board.rows.is_empty() && section.board.fleet.is_empty())
        && bindings.iter().all(|section| section.rows.is_empty())
}

/// One live ball: what it is, where the board put it, who is on it and what it
/// has cost.
fn live(ui: &mut egui::Ui, row: &BoardRow) {
    ui.label(headline(row));
    ui.colored_label(
        theme::tone_ink(&crate::reply::convs::Tone::Weak),
        placed(row),
    );
    for said in [held(row), gated(row), worked(row)].into_iter().flatten() {
        ui.colored_label(theme::tone_ink(&crate::reply::convs::Tone::Weak), said);
    }
    for (figure, over) in [
        (row.spend.as_ref(), "spent"),
        (row.rollup.as_ref(), "under it"),
    ] {
        if let Some(figure) = figure {
            ui.colored_label(
                theme::tone_ink(&crate::reply::convs::Tone::Weak),
                format!("{over}: {}", cost(figure)),
            );
        }
    }
}

#[cfg(test)]
mod tests;
