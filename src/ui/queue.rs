//! **The decision queue**: everything waiting on the operator, across every
//! workspace this box can reach (bl-f0ef; yog's `docs/REMOTE.md` §6, §9.11).
//!
//! # The fourth covering pane, and the first that is about no focus
//!
//! `crate::ui::tuning` is about the aimed wall and `crate::ui::records` about
//! the selected conversation, so both open on a subject and close when it
//! moves. This one has no subject to move: `attention` names no workspace, so
//! the read fans over every channel and the pane is the union. That is why its
//! control hangs off the **roster** — the one pane that is already the union
//! across channels — and why it is offered with nothing aimed at and nothing
//! selected, which is the seat most likely to be asking the question.
//!
//! # The flag is why this exists
//!
//! A flag is a second party asking the operator to look at one conversation, in
//! their own words (REMOTE §9.11). Until this pane the seat carried the field
//! and painted it nowhere, so the one place the ask could go was the one place
//! it went to die. It leads the row's detail for that reason, above the failure
//! and the parked invocation: it is the only line on a row that somebody
//! *wrote* rather than something the engine observed.
//!
//! # One control answers a row and one leaves the pane for it
//!
//! [`SEEN`] crosses the boundary and carries its `act:` token; so does each of
//! `crate::verbs::VERDICTS` on a row that is holding a call; [`GO`] aims and
//! selects, which is a view and out of the parity contract (PARITY §2). Both
//! stand down on a row this seat cannot address, and the row says why in the
//! roster's own words — `crate::ui::model::queue::Model::wall` is the one place
//! that resolution is asked, so this pane holds no opinion about how a wall is
//! addressed.
//!
//! # What is painted is words, computed beside the paint
//!
//! Every line comes off a pure function of the row, so the suite reads the
//! sentence rather than the layout — `crate::ui::records`' own rule. The state
//! and the signals ride verbatim (`crate::reply` rung 3): a word this build has
//! never seen paints as itself.

use crate::reply::queue::QueueRow;
use crate::ui::{Model, theme};

/// The word that opens the pane. It hangs off the roster, above the channels.
pub const OPEN: &str = "waiting on you…";
/// The word that closes it.
pub const CLOSE: &str = "done";
/// The pane's own heading.
pub const HEADING: &str = "waiting on you";
/// What it says before any channel has answered.
pub const NOT_ANSWERED: &str = "waiting to hear what is asking for you";
/// What it says once they have, and nothing is. A fact about the world, and
/// the one empty state here that is not a wait.
pub const NOTHING: &str = "nothing is waiting on you";
/// The word on the control that answers a row's place in the queue.
pub const SEEN: &str = "seen";
/// The word on the control that leaves the pane for the conversation.
pub const GO: &str = "go to it";

/// Paint the pane and take the clicks on it. Answers whether there was one to
/// paint, so the shell knows whether the conversation still stands.
pub fn render(ui: &mut egui::Ui, model: &mut Model) -> bool {
    if !model.showing(crate::ui::Listing::Queue) {
        return false;
    }
    ui.heading(HEADING);
    ui.horizontal_wrapped(|ui| {
        if ui.button(CLOSE).clicked() {
            model.close_queue();
        }
    });
    ui.separator();
    let waiting = model.waiting.clone();
    if waiting.is_empty() {
        ui.label(NOT_ANSWERED);
        return true;
    }
    if waiting.iter().all(|section| section.rows.is_empty()) {
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
            for section in &waiting {
                // **A section that answered nothing is silent, not a sentence.**
                // The union is the subject here; an engine holding nothing
                // waiting is a fact the roster already carries per wall, and a
                // header over a blank for every quiet channel would bury the
                // rows this pane exists for.
                if section.rows.is_empty() {
                    continue;
                }
                ui.separator();
                ui.label(crate::ui::roster::header(&section.channel));
                for row in &section.rows {
                    waiting_row(ui, model, row);
                }
            }
        });
    true
}

/// One row: the headline, why it is asking, and what can be done about it.
fn waiting_row(ui: &mut egui::Ui, model: &mut Model, row: &QueueRow) {
    ui.label(headline(row));
    // The flag first — it is the only line somebody wrote.
    for said in [flagged(row), row.failure.clone(), parked(row)]
        .into_iter()
        .flatten()
    {
        ui.colored_label(theme::NOTICE, said);
    }
    if let Some(said) = signalled(row) {
        ui.colored_label(theme::tone_ink(&crate::reply::convs::Tone::Weak), said);
    }
    if !row.preview.is_empty() {
        ui.colored_label(
            theme::tone_ink(&crate::reply::convs::Tone::Weak),
            row.preview.clone(),
        );
    }
    acts(ui, model, row);
}

/// The row's controls, or the sentence that stands where they would be.
fn acts(ui: &mut egui::Ui, model: &mut Model, row: &QueueRow) {
    if model.wall(&row.workspace).is_none() {
        ui.colored_label(theme::NOTICE, crate::ui::roster::NO_NAME_HERE);
        return;
    }
    let mut fired = None;
    let mut verdict = None;
    ui.horizontal_wrapped(|ui| {
        let answer = ui.button(SEEN);
        crate::ui::act::tag(&answer, &[crate::verbs::SEEN.word]);
        if answer.clicked() {
            fired = Some(row.clone());
        }
        // **The three verdicts, offered only on a row that is holding one**
        // (bl-bce2). `answer` is scoped to the exact call parked at the far
        // end, so a control on a row with nothing parked would fire a gesture
        // the engine refuses by name — and this pane is the one place that
        // already says what is parked (`parked`).
        if row.held.is_some() {
            for word in crate::verbs::VERDICTS {
                let seat = ui.button(word);
                crate::ui::act::tag(&seat, &[crate::verbs::ANSWER.word]);
                if seat.clicked() {
                    verdict = Some(word);
                }
            }
        }
        // **No `act:` token**: aiming and selecting cross no wire, so this is a
        // view (PARITY §2) and tagging it would put a widget in a ledger whose
        // unit is an op.
        if ui.button(GO).clicked() {
            model.go_to(row);
        }
    });
    if let Some(row) = fired {
        model.post_seen(&row);
    }
    if let Some(word) = verdict {
        model.post_answer(row, word);
    }
}

/// **The one line a row always gets**: what it is called, where it is, what it
/// is doing, and how long it has waited.
///
/// The workspace is on it because this pane is the one place in the window
/// where two rows may be on two different walls, so a label alone would not
/// say which conversation an operator is looking at.
pub fn headline(row: &QueueRow) -> String {
    let mut said = vec![format!(
        "{}  on {}  [{}{}]  {}",
        row.display,
        row.workspace,
        row.state.label(),
        if row.uncertain {
            crate::ui::convs::UNCERTAIN
        } else {
            ""
        },
        crate::ui::convs::age(row.age_secs)
    )];
    if row.pending > 0 {
        said.push(format!("{} under it", row.pending));
    }
    said.join("  ")
}

/// **The raised flag, in the raiser's own words** — the line this pane was
/// built for, or none where nobody raised one.
pub fn flagged(row: &QueueRow) -> Option<String> {
    let flag = row.flag.as_ref()?;
    Some(format!("flagged {} — {}", flag.at, flag.reason))
}

/// The invocation parked at the capability boundary, or none.
///
/// It is painted and not actionable: the control that releases or declines one
/// is `answer`, which belongs to the tool-host pane this seat does not have
/// (`parity.toml`, bl-e53c). Saying what is parked is worth more than saying
/// nothing while that pane is unbuilt.
pub fn parked(row: &QueueRow) -> Option<String> {
    let held = row.held.as_ref()?;
    Some(format!(
        "held at the boundary: {} ({}) — {}",
        held.tool, held.tool_use, held.reason
    ))
}

/// Why it is asking, as the engine's own words joined — or none where it named
/// nothing, which is a row asking for a reason no token spells.
pub fn signalled(row: &QueueRow) -> Option<String> {
    (!row.signals.is_empty()).then(|| row.signals.join("  "))
}

#[cfg(test)]
mod tests;
