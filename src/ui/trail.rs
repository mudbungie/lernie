//! **The trail**: every action that crossed an engine's boundary, newest last
//! (bl-4c48; yog's `docs/REMOTE.md` §9.17).
//!
//! # The ninth covering pane, and the fourth about no focus
//!
//! `ops` names no workspace, so the read fans over every channel and the pane
//! is the union — the decision queue's shape (`crate::ui::queue`), which is
//! also why its control hangs off the roster rather than off a row. What it
//! adds to the window is the answer to *what has this box been doing*, which
//! until now was a question the seat carried the wire for and painted nowhere.
//!
//! # It reads the engine's words and classifies nothing
//!
//! REMOTE §9.17 put the classification on the wire precisely so a seat would
//! not re-derive it: the sentinel table, the `128 + n` signal reading, the
//! retirement key, the ack scan and the origin grouping are five derivations
//! with one home apiece, *"whose failure mode is a seat quietly disagreeing
//! rather than failing to build"*. So this pane paints `exit_label`, `failed`
//! and `standing` and never looks at the exit integer — which rides in the
//! detail because it is the next thing an operator asks for, not because
//! anything here reads it.
//!
//! # An alarm is a row that is standing, and it is said in the engine's word
//!
//! `clean` is silence (`crate::reply::ops::CLEAN`), because a badge on every
//! ordinary run would bury the rows this pane exists for. Every other standing
//! paints as its own word, in the notice ink where the row failed and weakly
//! where it did not — so *handed off, no exit to observe* does not read as an
//! alarm, and a word this build has never seen still paints as itself
//! (`crate::reply` rung 3).
//!
//! # What is painted is words, computed beside the paint
//!
//! Every line comes off a pure function of the row, so the suite reads the
//! sentence rather than the layout — `crate::ui::records`' own rule.

use crate::reply::ops::{CLEAN, OpRow};
use crate::ui::{Model, theme};

/// The word that opens the pane. It hangs off the roster, above the channels.
pub const OPEN: &str = "the trail…";
/// The word that closes it.
pub const CLOSE: &str = "done";
/// The word on the control that appends the operator's watermark.
pub const ACK: &str = "acknowledge every alarm";
/// The pane's own heading.
pub const HEADING: &str = "the trail";
/// What it says before any channel has answered.
pub const NOT_ANSWERED: &str = "waiting to hear what has crossed the boundary";
/// What it says once they have and nothing has. A fact about the engines, and
/// the one empty state here that is not a wait.
pub const NOTHING: &str = "nothing has crossed the boundary yet";

/// **Whether every channel that answered answered nothing** — the one reading
/// of these rows this pane makes, and it is about the trail rather than about
/// an alarm: what is standing is the engine's classification and stays there
/// (REMOTE §9.17).
fn crossed_any(model: &Model) -> bool {
    model.trails.iter().all(|section| section.rows.is_empty())
}

/// Paint the pane and take the clicks on it. Answers whether there was one to
/// paint, so the shell knows whether the conversation still stands.
pub fn render(ui: &mut egui::Ui, model: &mut Model) -> bool {
    if !model.trailing() {
        return false;
    }
    ui.heading(HEADING);
    ui.horizontal_wrapped(|ui| {
        if ui.button(CLOSE).clicked() {
            model.close_lookup();
        }
        // **The watermark is a control on the pane and not on a row**, because
        // that is what the op is: `ack` takes no address at all and appends
        // one line every failure-derived alarm reads past. Offering it per row
        // would be four controls for one gesture (bl-b8f7).
        //
        // It is offered only where there is a trail to acknowledge — the
        // enablement rule with the SUBJECT missing rather than a parameter —
        // and *a trail* is the one reading of these rows this seat makes: the
        // classification stays the engine's (REMOTE §9.17), so nothing here
        // asks whether an alarm is standing.
        let seen = ui.add_enabled(!crossed_any(model), egui::Button::new(ACK));
        crate::ui::act::tag(&seen, &[crate::verbs::ACK.word]);
        if seen.clicked() {
            model.post_ack();
        }
        // **And the cut opens a place rather than firing** (DESIGN §4.20), so
        // it carries no `act:` token: the op is tagged on the control inside
        // that pane which actually spends it — the division `enroll a box…`
        // keeps with `mint`.
        if ui.button(crate::ui::clear::OPEN).clicked() {
            model.begin_clearing();
        }
    });
    ui.separator();
    let trails = model.trails.clone();
    if trails.is_empty() {
        ui.label(NOT_ANSWERED);
        return true;
    }
    if trails.iter().all(|section| section.rows.is_empty()) {
        ui.label(NOTHING);
        return true;
    }
    // One scroll for every section, and the heading above it fixed — the shape
    // every pane here keeps, for the reason the tuning pane states: a pane cut
    // off mid-row says nothing about having been cut.
    egui::ScrollArea::vertical()
        .id_salt(HEADING)
        .auto_shrink(false)
        .show(ui, |ui| {
            for section in &trails {
                // **A section that answered nothing is silent, not a sentence**
                // — the queue's own rule: the union is the subject, and a
                // header over a blank for every quiet engine would bury the
                // rows this pane exists for.
                if section.rows.is_empty() {
                    continue;
                }
                ui.separator();
                ui.label(crate::ui::roster::header(&section.channel));
                for row in &section.rows {
                    crossed(ui, row);
                }
            }
        });
    true
}

/// One action: what ran, where its alarm stands, when and where it ran, and
/// whatever it said.
fn crossed(ui: &mut egui::Ui, row: &OpRow) {
    ui.label(headline(row));
    if let Some(said) = standing(row) {
        let ink = if row.failed {
            theme::NOTICE
        } else {
            theme::tone_ink(&crate::reply::convs::Tone::Weak)
        };
        ui.colored_label(ink, said);
    }
    ui.colored_label(
        theme::tone_ink(&crate::reply::convs::Tone::Weak),
        provenance(row),
    );
    if let Some(said) = output(row) {
        ui.label(egui::RichText::new(said).monospace());
    }
}

/// **The one line a row always gets**: the command that ran, and how it ended
/// in the engine's own words.
///
/// The label and never the integer — REMOTE §9.17's *"a client that renders
/// the trail should stop classifying `exit` and read the words"*.
pub fn headline(row: &OpRow) -> String {
    format!("{}  [{}]", row.argv, row.exit_label)
}

/// **Where the row's alarm stands**, or none for a clean run.
///
/// The word rides verbatim: this seat knows one of the five ([`CLEAN`]) and
/// knows it only in order to say nothing, so a standing that grew upstream
/// costs a badge and not a decode.
pub fn standing(row: &OpRow) -> Option<String> {
    (row.standing != CLEAN).then(|| row.standing.clone())
}

/// When and where it ran, and what subject it belongs to — the field a failure
/// banner groups by, carried here because it is what makes two rows with the
/// same argv two different facts.
pub fn provenance(row: &OpRow) -> String {
    format!(
        "{}  {}  in {}  (exit {})",
        row.ts, row.origin, row.cwd, row.exit
    )
}

/// **What it said**, if it said anything — the complaint first, because a row
/// that failed is read for why.
///
/// The two are one line rather than two rows: a trail is read by scanning, and
/// a child that printed on both streams is one event.
pub fn output(row: &OpRow) -> Option<String> {
    let said: Vec<String> = [&row.stderr, &row.stdout]
        .into_iter()
        .filter(|stream| !stream.is_empty())
        .cloned()
        .collect();
    (!said.is_empty()).then(|| said.join("\n"))
}

#[cfg(test)]
mod tests;
