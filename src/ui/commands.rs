//! **What these engines answer**: every op each one has a word for, and what
//! each is for (bl-40ec; DESIGN §4.21).
//!
//! # The sixth covering pane, and the second about no focus
//!
//! `crate::ui::queue` is the first: `attention` names no workspace, so its
//! subject is every channel and the pane is the union. `help` names none
//! either, so this is that shape one noun over — its control hangs off the
//! **roster** for the same reason, and it is offered on a seat that has aimed
//! at nothing.
//!
//! # It is sectioned per channel and deliberately not merged
//!
//! Two engines on one box may be at two protocol versions, so their tables are
//! two answers and a union would say they are one thing. The section header is
//! the roster's own (`crate::ui::roster::header`), which carries the address
//! the entry dials — so two entries terminating at one listener are visible as
//! such here exactly as they are on the roster (bl-77df).
//!
//! # This is the table this seat is judged by
//!
//! `crate::snapshot::parity::roster` reads the `surface` field off the
//! vendored fixture of this same shape, because it is the one home for *which
//! ops owe every seat a discoverable interactable* (yog's `docs/PARITY.md`
//! §2). So the pane an operator reads and the ledger that reddens for a
//! missing control come off one answer, and the classification is painted
//! rather than hidden: an op marked `machine` is one nothing here owes a
//! control, and saying so is the difference between a short pane and an
//! incomplete one.
//!
//! # And it is not `lernie help`
//!
//! That answers what this BINARY takes, from a table compiled into it, with
//! nothing provisioned (`crate::verbs::help`). This answers what the ENGINE
//! offers. Two subjects; the wire op is the second, and `crate::verbs::window`
//! states why only one of them has an argv row.

use crate::ui::{Model, theme};

/// The word that opens the pane. It hangs off the roster, above the channels.
pub const OPEN: &str = "what these engines answer…";
/// The word that closes it.
pub const CLOSE: &str = "done";
/// The pane's own heading.
pub const HEADING: &str = "what these engines answer";
/// What it says before any channel has answered.
pub const NOT_ANSWERED: &str = "waiting to hear what these engines answer to";
/// What a section says for an engine that answered and named no op at all — a
/// fact about that engine, and the one empty state here that is not a wait.
pub const NO_OPS: &str = "this engine names no op";

/// Paint the pane and take the clicks on it. Answers whether there was one to
/// paint, so the shell knows whether the conversation still stands.
pub fn render(ui: &mut egui::Ui, model: &mut Model) -> bool {
    if !model.commanding() {
        return false;
    }
    ui.heading(HEADING);
    ui.horizontal_wrapped(|ui| {
        if ui.button(CLOSE).clicked() {
            model.close_lookup();
        }
    });
    ui.separator();
    let pages = model.pages.clone();
    if pages.is_empty() {
        ui.label(NOT_ANSWERED);
        return true;
    }
    // One scroll for the whole pane and the heading above it fixed — the shape
    // every pane here keeps, for the reason the tuning pane states: a pane cut
    // off mid-row says nothing about having been cut.
    egui::ScrollArea::vertical()
        .id_salt(HEADING)
        .auto_shrink(false)
        .show(ui, |ui| {
            for section in &pages {
                ui.separator();
                ui.label(crate::ui::roster::header(&section.channel));
                if section.rows.is_empty() {
                    ui.label(NO_OPS);
                    continue;
                }
                for row in &section.rows {
                    op(ui, row);
                }
            }
        });
    true
}

/// One op: what to type and who it is for, then the sentence, then the page.
///
/// Every line comes off a pure function of the row (`crate::reply::help`), so
/// the suite reads the sentence rather than the layout — `crate::ui::records`'
/// own rule.
fn op(ui: &mut egui::Ui, row: &crate::reply::help::HelpRow) {
    ui.label(row.headline());
    ui.colored_label(
        theme::tone_ink(&crate::reply::convs::Tone::Weak),
        row.summary.clone(),
    );
    // **The page WRAPS.** A horizontal run lays a paragraph on one line however
    // long it is and the panel cuts what reaches the frame, with no ellipsis —
    // the defect bl-3d0f closed on the notice bar, and a detail paragraph is
    // the longest prose this window paints.
    ui.add(egui::Label::new(row.detail.clone()).wrap());
}

#[cfg(test)]
mod tests;
