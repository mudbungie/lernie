//! **The undelivered mail** waiting in the conversation's inbox (bl-3257).
//!
//! The last of the pane's halves, and the one about what has not happened yet:
//! the steps are what the loop did, the files are what it touched, the spine
//! is what it is anchored to, and this is what it has not read.
//!
//! # A forgiving parse means a row can state almost nothing, and that is a
//! reading rather than a failure
//!
//! The engine renders a half-written or hand-edited deposit with whatever
//! fields it actually stated. So a row whose sender and stamp are absent is
//! not a malformed answer — and painting `?` where a fact was not stated is
//! upstream's own rendering rule, kept here so a hand-edited deposit stays
//! legible instead of losing its header.
//!
//! # The bytes are painted, not the body, when the two differ
//!
//! A deposit that parsed has a body; one that did not has the whole file as
//! its body and the frontmatter fields absent. Either way the reading is what
//! is shown, and the file's own bytes ride under it — which is the only door
//! this seat has to a deposit whose envelope the parse dropped.

use crate::reply::convs::Tone;
use crate::reply::inbox::Row;
use crate::ui::{Model, theme};

/// The half's own heading.
pub const HEAD: &str = "inbox";
/// What it says before the first answer.
pub const NOT_ANSWERED: &str = "waiting to hear what is in its inbox";
/// What it says for a conversation whose inbox is empty — a fact about the
/// conversation, and the one empty state here that is not a wait.
pub const NO_MAIL: &str = "nothing is waiting in its inbox";
/// What an unstated frontmatter fact reads as, in the engine's own rendering.
pub const UNSTATED: &str = "?";
/// Said above a deposit whose body is empty.
pub const SAID_NOTHING: &str = "it says nothing";

/// Paint the mail half.
pub fn render(ui: &mut egui::Ui, model: &Model) {
    ui.label(egui::RichText::new(HEAD).strong());
    let Some(rows) = model.records.mail.as_ref() else {
        ui.label(NOT_ANSWERED);
        return;
    };
    if rows.is_empty() {
        ui.label(NO_MAIL);
        return;
    }
    for row in rows {
        ui.label(headline(row));
        if row.deposit.body.trim().is_empty() {
            ui.colored_label(theme::tone_ink(&Tone::Weak), SAID_NOTHING);
        } else {
            ui.label(row.deposit.body.clone());
        }
    }
}

/// **The one line a deposit always gets**: its file, who sent it and when —
/// each unstated fact said as itself rather than dropped, so a hand-edited
/// deposit keeps a header.
pub fn headline(row: &Row) -> String {
    let said = format!(
        "{} — from {} at {}",
        row.name,
        row.deposit.from.clone().unwrap_or(UNSTATED.to_owned()),
        row.deposit
            .deposited_at
            .clone()
            .unwrap_or(UNSTATED.to_owned())
    );
    match ending(row) {
        Some(ended) => format!("{said} — {ended}"),
        None => said,
    }
}

/// **How the sending agent ended**, on a result message — the epitaph and the
/// commit it ended at. `None` on an ordinary deposit, which states neither.
pub fn ending(row: &Row) -> Option<String> {
    let epitaph = row.deposit.epitaph.clone()?;
    match &row.deposit.terminal_ref {
        Some(at) => Some(format!("ended {epitaph}, at {at}")),
        None => Some(format!("ended {epitaph}")),
    }
}

#[cfg(test)]
mod tests;
