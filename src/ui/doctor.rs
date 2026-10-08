//! **Whether this box is wired**: one row per check, sectioned per channel
//! (bl-9bbb; DESIGN §4.21; yog's `docs/REMOTE.md`, *"And one read asks all of
//! it at once"*).
//!
//! # A window-level read, on the verb table's standing
//!
//! Its control hangs on the roster's strip beside `verbs…`, because bare it
//! names no workspace and its subject is every channel this box holds — so it
//! is offered on a seat that has aimed at nothing, which is the seat most
//! likely to be asking whether anything is wired at all. Aimed, the same
//! control asks the deeper question about that wall (`crate::ui::model::
//! doctor`).
//!
//! # Every line is the engine's
//!
//! The check, the fact and the remedy are painted as sent: a diagnostic with
//! its own opinion is a second authority for a settled decision. The one thing
//! this pane composes is whether a row holds, and that is a word and a colour
//! off the row's own `ok`.

use crate::reply::doctor::Check;
use crate::ui::{Model, theme};

/// The word that opens the pane. It hangs off the roster, above the channels.
pub const OPEN: &str = "doctor…";
/// The word that closes it.
pub const CLOSE: &str = "done";
/// The pane's own heading.
pub const HEADING: &str = "is this box wired";
/// What it says before any channel has answered.
pub const NOT_ANSWERED: &str = "waiting to hear whether this box is wired";
/// What a section says for an engine that answered and ran no check — a fact
/// about that engine, and the one empty state here that is not a wait.
pub const NO_CHECKS: &str = "this engine ran no check";

/// Paint the pane and take the clicks on it. Answers whether there was one to
/// paint, so the shell knows whether the conversation still stands.
pub fn render(ui: &mut egui::Ui, model: &mut Model) -> bool {
    if !model.doctoring() {
        return false;
    }
    ui.heading(HEADING);
    ui.horizontal_wrapped(|ui| {
        if ui.button(CLOSE).clicked() {
            model.close_lookup();
        }
    });
    ui.separator();
    let diagnoses = model.diagnoses.clone();
    if diagnoses.is_empty() {
        ui.label(NOT_ANSWERED);
        return true;
    }
    egui::ScrollArea::vertical()
        .id_salt(HEADING)
        .auto_shrink(false)
        .show(ui, |ui| {
            for section in &diagnoses {
                ui.separator();
                ui.label(crate::ui::roster::header(&section.channel));
                if section.rows.is_empty() {
                    ui.label(NO_CHECKS);
                    continue;
                }
                for row in &section.rows {
                    check(ui, row);
                }
            }
        });
    true
}

/// One check: whether it holds, what was checked and what was found, in the
/// colour of its state — then the engine's remedy, verbatim and wrapped, where
/// there is one.
fn check(ui: &mut egui::Ui, row: &Check) {
    let state = if row.ok {
        theme::State::Rest
    } else {
        theme::State::Error
    };
    ui.add(
        egui::Label::new(egui::RichText::new(row.headline()).color(theme::accent(state))).wrap(),
    );
    if let Some(remedy) = &row.remedy {
        ui.add(egui::Label::new(remedy.clone()).wrap());
    }
}

#[cfg(test)]
mod tests;
