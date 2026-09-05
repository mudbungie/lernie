//! **The config pane**: the files a wall's policy is written in, and the
//! settings the engine reads out of them (yog's `docs/REMOTE.md` §9, §9.18;
//! PROTOCOL 13).
//!
//! # Two views of one read, never two reads
//!
//! The answer carries the file's bytes and the schema applied to *those very
//! bytes* (REMOTE §9.18), so the settings above the text and the text below it
//! are one moment. A pane that asked twice would be painting two.
//!
//! # The judgement is the engine's and this pane never composes one
//!
//! A setting's `fault` is the engine's own words about that value — the same
//! call its pick gate makes — so what is painted red here is what the far end
//! says is wrong, never what this end guessed. A file with no schema answers no
//! settings at all, which is not an error: it is the raw-text destination doing
//! exactly what upstream says it does.
//!
//! # It reads and does not write, and the reason is a hazard rather than time
//!
//! `config` is one op that is a read or a write depending on whether it carries
//! `text` at all, so the editor is one field away — and the field is not the
//! problem. Three of the five destinations name no workspace, and a gesture
//! naming no workspace is **fanned** by this seat's poster: an act composed for
//! one engine's `cadence.yaml` would be written to every channel this box
//! holds. That is bl-4855's, with the frame and the hazard in it.
//!
//! # The workflow destination is not offered, because nothing lists one
//!
//! `litany-workflow` is addressed by a name, and no read this seat has answers
//! what workflow names exist — so a control for it would be a text box asking
//! the operator to remember. It lands with the editor (bl-4855), which needs a
//! box anyway.

use crate::reply::config::{Config, Setting};
use crate::reply::lineages::Lineage;
use crate::ui::{Model, theme};
use crate::verbs::Where;

/// The word that opens the pane, on the wall the window is aimed at.
pub const OPEN: &str = "config…";
/// The word that closes it.
pub const CLOSE: &str = "done";
/// The pane's own heading.
pub const HEADING: &str = "config";
/// What it says before a file has been picked.
pub const NOTHING_PICKED: &str = "pick a file to read";
/// What it says for a wall nobody has been answered about yet.
pub const NOT_ANSWERED: &str = "waiting to hear which lineages this wall holds";
/// What it says for a wall that answered and holds no lineage — a fact about
/// the workspace, and the one empty state here that is not a wait.
pub const NO_LINEAGES: &str = "this wall holds no config lineage";
/// What it says under a file that was asked for and has not answered.
pub const NOT_READ: &str = "waiting to hear what this file holds";
/// What it says for a destination that answered and is empty — a file that
/// does not exist yet reads as no bytes rather than as a refusal.
pub const NO_BYTES: &str = "this file has no bytes yet";
/// What it says for a file whose destination has no schema at all.
pub const NO_SETTINGS: &str = "this file has no typed settings — its bytes are the whole of it";

/// Paint the pane and take the clicks on it. Answers whether there was one to
/// paint, so the shell knows whether the conversation still stands.
pub fn render(ui: &mut egui::Ui, model: &mut Model) -> bool {
    if model.configuring.is_none() {
        return false;
    }
    ui.heading(HEADING);
    if let Some(aim) = model.aim.clone() {
        ui.label(format!("on {} — {}", aim.address, aim.channel));
    }
    ui.horizontal_wrapped(|ui| {
        if ui.button(CLOSE).clicked() {
            model.close_configuring();
        }
    });
    ui.separator();
    egui::ScrollArea::vertical()
        .id_salt(HEADING)
        .auto_shrink(false)
        .show(ui, |ui| {
            destinations(ui, model);
            ui.separator();
            file(ui, model);
        });
    true
}

/// **What can be read**, in the order an operator meets them: the wall's own
/// brazen file, the engine's two globals, then every path on every lineage.
fn destinations(ui: &mut egui::Ui, model: &mut Model) {
    let Some(aim) = model.aim.clone() else {
        return;
    };
    ui.horizontal_wrapped(|ui| {
        for at in [
            Where::Brazen {
                workspace: aim.address.clone(),
            },
            Where::LitanyModels,
            Where::Cadence,
        ] {
            pick(ui, model, &at);
        }
    });
    match model.lineages.clone() {
        None => {
            ui.label(NOT_ANSWERED);
        }
        Some(rows) if rows.is_empty() => {
            ui.label(NO_LINEAGES);
        }
        Some(rows) => {
            for row in &rows {
                lineage(ui, model, row, &aim.address);
            }
        }
    }
}

/// One lineage: what it is and where its tip stands, then a control per path
/// the tip holds.
fn lineage(ui: &mut egui::Ui, model: &mut Model, row: &Lineage, workspace: &str) {
    ui.colored_label(
        theme::tone_ink(&crate::reply::convs::Tone::Weak),
        row.line(),
    );
    ui.horizontal_wrapped(|ui| {
        for path in &row.files {
            pick(
                ui,
                model,
                &Where::Branch {
                    workspace: workspace.to_owned(),
                    lineage: row.name.clone(),
                    path: path.clone(),
                },
            );
        }
    });
}

/// One control that points the pane at a file. **The one this seat is already
/// reading shows as chosen** rather than being dropped from the row: the set
/// of destinations is a fact about the wall, and hiding the one in force would
/// make the picker change shape under a click.
fn pick(ui: &mut egui::Ui, model: &mut Model, at: &Where) {
    let chosen = model.configured().as_ref() == Some(at);
    let control = ui.selectable_label(chosen, at.label());
    crate::ui::act::tag(&control, &[crate::verbs::CONFIG]);
    if control.clicked() {
        model.read_config(at);
    }
}

/// The file the pane is pointed at: its settings, then its bytes.
fn file(ui: &mut egui::Ui, model: &Model) {
    let Some(at) = model.configured() else {
        ui.label(NOTHING_PICKED);
        return;
    };
    ui.label(at.label());
    let Some(held) = model.config.clone() else {
        ui.label(NOT_READ);
        return;
    };
    settings(ui, &held);
    ui.separator();
    if held.text.is_empty() {
        ui.label(NO_BYTES);
        return;
    }
    ui.label(held.text);
}

/// Every setting the schema found, each under the declaration it belongs to.
fn settings(ui: &mut egui::Ui, held: &Config) {
    if held.settings.is_empty() {
        ui.colored_label(
            theme::tone_ink(&crate::reply::convs::Tone::Weak),
            NO_SETTINGS,
        );
        return;
    }
    for row in &held.settings {
        setting(ui, row);
    }
}

/// One setting: where it lives, what it says, what it takes, and the engine's
/// judgement of it where there is one.
fn setting(ui: &mut egui::Ui, row: &Setting) {
    let weak = theme::tone_ink(&crate::reply::convs::Tone::Weak);
    // **Every row wraps**, because this pane stands in the central panel and
    // the central panel is what the two side panels leave (bl-dc07).
    ui.horizontal_wrapped(|ui| {
        ui.label(format!("{}.{}", row.entry, row.name));
        ui.label(row.value.clone());
        ui.colored_label(weak, row.control.says());
    });
    ui.colored_label(weak, row.help.clone());
    if let Some(fault) = &row.fault {
        ui.colored_label(theme::NOTICE, fault.clone());
    }
}

#[cfg(test)]
mod tests;
