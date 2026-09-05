//! **The clients pane**: which machines may execute for this wall, and what
//! each of them says it can do (yog's `docs/REMOTE.md` §5, §5.1; PROTOCOL 2).
//!
//! # Two lifetimes on one row, and the pane says both
//!
//! Presence is an **observation** — true at the instant the engine answered,
//! and recorded nowhere on either end — while the advertised set is a
//! **statement** the machine last made and which stands whether or not it is
//! connected. So a row reads *not connected* beside a full set as the ordinary
//! thing rather than as a contradiction: a tool host holds its connection only
//! while it is waiting for work, which means a busy machine and an absent one
//! look the same from here. The sentence says that outright, because the
//! alternative is an operator reading *absent* as *broken* once a day.
//!
//! # The consent is said on every tool, present or absent
//!
//! `subject_cwd` is the fact yog's worktree lane routes on (REMOTE §5.1): the
//! advertising box consenting to run this tool at a directory the invocation
//! names. It is stated for **both** answers rather than only for the consenting
//! ones, because a line that appears only when true makes its absence
//! ambiguous — an operator cannot tell a tool that refuses a subject from one
//! this build forgot to ask about — and the whole complaint that landed this
//! pane was being unable to tell those two machines apart.
//!
//! # There is no control on it, and that is the surface being honest
//!
//! Everything else on this family is a MACHINE's op (`crate::verbs::clients`
//! states which and why): `advertise` is a host's statement about itself, and
//! `invocations` *drains* the queue addressed to whichever certificate asked —
//! so a seat that offered either would be pretending to be a tool host with
//! somebody else's work in its hands. What an operator does with what they read
//! here happens on the conversation that is calling the tool, not on the
//! machine that would run it.

use crate::reply::clients::{ClientRow, ToolRow};
use crate::ui::{Model, theme};

/// The word that opens the pane, on the wall the window is aimed at.
pub const OPEN: &str = "machines…";
/// The word that closes it.
pub const CLOSE: &str = "done";
/// The pane's own heading.
pub const HEADING: &str = "machines";
/// What it says for a wall nobody has been answered about yet.
pub const NOT_ANSWERED: &str = "waiting to hear which machines this wall holds";
/// What it says for a wall that answered and holds no registration at all — a
/// fact about the workspace, and the one empty state here that is not a wait.
pub const NONE_REGISTERED: &str = "no machine is registered in this workspace: \
                                   registering one is an operator's act on the \
                                   server that holds it, never a gesture from \
                                   here";
/// What a row says of a machine that has presented no set.
pub const OFFERS_NOTHING: &str = "has advertised no tool";
/// What a tool says when its box consents to a caller-named directory.
pub const SUBJECT: &str = "runs where the caller says";
/// And when it does not — stated rather than left out, so an absence is never
/// mistaken for a fact nobody asked about.
pub const NO_SUBJECT: &str = "runs only where that box stands";

/// Paint the pane and take the clicks on it. Answers whether there was one to
/// paint, so the shell knows whether the conversation still stands.
pub fn render(ui: &mut egui::Ui, model: &mut Model) -> bool {
    if !model.showing(crate::ui::Listing::Clients) {
        return false;
    }
    ui.heading(HEADING);
    if let Some(aim) = model.aim.clone() {
        ui.label(format!("on {} — {}", aim.address, aim.channel));
    }
    ui.horizontal_wrapped(|ui| {
        if ui.button(CLOSE).clicked() {
            model.close_clients();
        }
    });
    ui.separator();
    match model.machines.clone() {
        None => {
            ui.label(NOT_ANSWERED);
        }
        Some(rows) if rows.is_empty() => {
            ui.label(NONE_REGISTERED);
        }
        // The list scrolls and the heading above it does not — the shape every
        // list in this window keeps (bl-e5d2).
        Some(rows) => {
            egui::ScrollArea::vertical()
                .id_salt(HEADING)
                .auto_shrink(false)
                .show(ui, |ui| {
                    for row in &rows {
                        client(ui, row);
                    }
                });
        }
    }
    true
}

/// One machine: what it is called, whether it was connected when the engine
/// answered, and what it offers.
fn client(ui: &mut egui::Ui, row: &ClientRow) {
    ui.separator();
    ui.label(row.line());
    let weak = theme::tone_ink(&crate::reply::convs::Tone::Weak);
    if row.tools.is_empty() {
        ui.colored_label(weak, OFFERS_NOTHING);
        return;
    }
    for offered in &row.tools {
        tool(ui, offered);
    }
}

/// One tool: the handle a call addresses it by, the host's own words, and
/// whether that box consents to run it somewhere the caller names.
fn tool(ui: &mut egui::Ui, row: &ToolRow) {
    let weak = theme::tone_ink(&crate::reply::convs::Tone::Weak);
    // **Every row wraps**, because this pane stands in the central panel and
    // the central panel is what the two side panels leave (bl-dc07).
    ui.horizontal_wrapped(|ui| {
        ui.label(row.name.clone());
        ui.colored_label(weak, if row.subject_cwd { SUBJECT } else { NO_SUBJECT });
    });
    ui.colored_label(weak, row.description.clone());
}

#[cfg(test)]
mod tests;
