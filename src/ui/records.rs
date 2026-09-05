//! **The records pane**: what the selected conversation's loop did and what
//! its worktree holds (bl-2cf7; yog's `docs/REMOTE.md` §8.5).
//!
//! # The tuning pane's shape, one noun over
//!
//! bl-213c built the conversation's ACTS and left its records to this pane —
//! the reads an operator actually reaches for on a conversation that is doing
//! something: the steps taken and the files touched. It is the third covering
//! pane and the second that is a place rather than a moment, and it follows
//! `crate::ui::tuning` in every joint: it opens on its subject (the SELECTED
//! CONVERSATION where tuning's is the aimed wall), its two reads are standing
//! while it is open (`crate::state::Standing::records`), and everything on it
//! is the engine's answer — this pane holds no state at all, not even a draft.
//!
//! # Every empty state is a sentence, and the sentences differ
//!
//! A conversation nobody has been answered about is not one whose loop did
//! nothing ([`NOT_ANSWERED_STEPS`] vs [`NO_STEPS`]) — the same doctrine the
//! conversation list wrote down first. The worktree adds a third claim of its
//! own: `files` answers a torn-down worktree as an ABSENCE and an empty one as
//! a listing of nothing, and the two paint as two sentences because they are
//! two facts ([`NO_WORKTREE`] vs [`EMPTY_WORKTREE`]).
//!
//! # What is painted is words, computed beside the paint
//!
//! Every line below comes off a pure function of the row, so the suite reads
//! the sentence rather than the layout — the same reason `RoleRow::runs_on`
//! is a method. The class tokens ride verbatim (`crate::reply` rung 3): a
//! `framing` or a `wound` this build has no word for paints as itself.

/// One step's records, under the row that addresses them.
pub mod drill;
/// The conversation's own row, as the pane's header.
pub mod header;
/// The undelivered mail waiting in its inbox.
pub mod mail;
/// The spine half: what the history is anchored to, and the fork off it.
pub mod spine;

use crate::reply::files::{Files, Preview};
use crate::reply::steps::{StepRow, Steps};
use crate::ui::{Model, theme};

/// The word that opens the pane, on the selected conversation.
pub const OPEN: &str = "records…";
/// The word that closes it.
pub const CLOSE: &str = "done";
/// The pane's own heading.
pub const HEADING: &str = "records";
/// The steps half's own heading.
pub const STEPS_HEAD: &str = "steps";
/// The files half's.
pub const FILES_HEAD: &str = "files";
/// What the steps half says before the first answer.
pub const NOT_ANSWERED_STEPS: &str = "waiting to hear what this conversation's loop has done";
/// What it says for a loop that answered, and has taken no step. A fact about
/// the conversation, and the one empty state here that is not a wait.
pub const NO_STEPS: &str = "its loop has taken no step yet";
/// What the files half says before the first answer.
pub const NOT_ANSWERED_FILES: &str = "waiting to hear what its worktree holds";
/// The worktree's absence — a different claim from an empty listing, and the
/// wire keeps them two on purpose.
pub const NO_WORKTREE: &str = "no worktree stands for this conversation";
/// The listing of nothing, which is the other claim.
pub const EMPTY_WORKTREE: &str = "the worktree holds nothing";
/// Said under a listing the engine cut short of the worktree.
pub const TRUNCATED: &str = "…and more — the listing was cut short";

/// Paint the pane and take the clicks on it. Answers whether there was one to
/// paint, so the shell knows whether the conversation still stands.
pub fn render(ui: &mut egui::Ui, model: &mut Model) -> bool {
    if !model.showing(crate::ui::Listing::Records) {
        return false;
    }
    ui.heading(HEADING);
    if let Some(conversation) = model.conversation.clone() {
        ui.label(format!("on {conversation}"));
    }
    ui.horizontal_wrapped(|ui| {
        if ui.button(CLOSE).clicked() {
            model.close_records();
        }
    });
    ui.separator();
    // One scroll for both halves, and the heading above it fixed — the shape
    // every pane here keeps, for the reason the tuning pane states: a pane cut
    // off mid-row says nothing about having been cut.
    let (steps, files) = (model.records.steps.clone(), model.records.files.clone());
    egui::ScrollArea::vertical()
        .id_salt(HEADING)
        .auto_shrink(false)
        .show(ui, |ui| {
            header::render(ui, model);
            ui.separator();
            steps_half(ui, model, steps.as_ref());
            ui.separator();
            files_half(ui, files.as_ref());
            ui.separator();
            spine::render(ui, model);
            ui.separator();
            mail::render(ui, model);
        });
    true
}

/// The steps half: the orphan banner, then one row per step with the control
/// that drills into it (bl-3257).
fn steps_half(ui: &mut egui::Ui, model: &mut Model, steps: Option<&Steps>) {
    ui.label(egui::RichText::new(STEPS_HEAD).strong());
    let Some(listing) = steps else {
        ui.label(NOT_ANSWERED_STEPS);
        return;
    };
    if let Some(said) = orphaned(listing) {
        ui.colored_label(theme::NOTICE, said);
    }
    if listing.rows.is_empty() {
        ui.label(NO_STEPS);
        return;
    }
    for row in &listing.rows {
        step(ui, model, row);
    }
}

/// One step: the headline, the provenance under it, what went wrong, and the
/// drill-in its `seq` addresses.
fn step(ui: &mut egui::Ui, model: &mut Model, row: &StepRow) {
    // **The headline, the provenance and the drill-in control share one
    // wrapped row.** This pane covers the window and its content has to fit
    // it: a control laid out past the frame is unreachable, and there are
    // seven halves under this one scroll (`crate::snapshot::clipped`).
    ui.horizontal_wrapped(|ui| {
        ui.label(headline(row));
        if let Some(weak) = provenance(row) {
            ui.colored_label(theme::tone_ink(&crate::reply::convs::Tone::Weak), weak);
        }
        drill::control(ui, model, &row.seq);
    });
    ui.horizontal_wrapped(|ui| {
        for said in [wounded(row), auth(row)].into_iter().flatten() {
            ui.colored_label(theme::NOTICE, said);
        }
    });
    drill::records(ui, model, &row.seq);
}

/// The files half: where the work lands, the listing, and the preview.
fn files_half(ui: &mut egui::Ui, files: Option<&Files>) {
    let Some(answer) = files else {
        ui.label(egui::RichText::new(FILES_HEAD).strong());
        ui.label(NOT_ANSWERED_FILES);
        return;
    };
    // **The heading shares its line with where the work lands**, and the
    // walked entries share one wrapped line with each other: seven halves ride
    // under this pane's one scroll and a control laid out past the frame is
    // unreachable (`crate::snapshot::clipped`).
    ui.horizontal_wrapped(|ui| {
        ui.label(egui::RichText::new(FILES_HEAD).strong());
        if let Some(dir) = &answer.working_dir {
            ui.colored_label(
                theme::tone_ink(&crate::reply::convs::Tone::Weak),
                format!("working in {dir}"),
            );
        }
    });
    match &answer.listing {
        None => {
            ui.label(NO_WORKTREE);
        }
        Some(listing) if listing.rows.is_empty() => {
            ui.label(EMPTY_WORKTREE);
        }
        Some(listing) => {
            ui.horizontal_wrapped(|ui| {
                for row in &listing.rows {
                    ui.label(entry(row));
                }
                if listing.truncated {
                    ui.colored_label(theme::NOTICE, TRUNCATED);
                }
            });
        }
    }
    if let Some(preview) = &answer.preview {
        ui.label(egui::RichText::new(previewed(preview)).monospace());
    }
}

/// One walked entry as a line: a directory wears its slash, a file its size.
pub fn entry(row: &crate::reply::files::FileRow) -> String {
    if row.dir {
        format!("{}/", row.path)
    } else {
        format!("{}  {} B", row.path, row.size)
    }
}

/// A bounded preview as text — the engine's three classes, and the rung-3
/// word painted as itself. Shared with the drill-in's two capture logs
/// ([`drill`]), which are the same bounded reading.
pub fn previewed(preview: &Preview) -> String {
    match preview {
        Preview::Text(text) => text.clone(),
        Preview::Truncated { text, size } => format!("{text}\n… {size} bytes in all"),
        Preview::Binary { size } => format!("binary — {size} bytes"),
        Preview::Unknown(word) => format!("a {word:?} preview, which this seat cannot show"),
    }
}

/// The orphan banner, or none: [`crate::reply::steps::NONE`] is the engine's
/// own *nothing is orphaned* and paints as silence rather than as a badge.
pub fn orphaned(listing: &Steps) -> Option<String> {
    if listing.orphan == crate::reply::steps::NONE {
        return None;
    }
    Some(match &listing.orphan_reason {
        Some(reason) => format!("an orphaned {} tail — {reason}", listing.orphan),
        None => format!("an orphaned {} tail", listing.orphan),
    })
}

/// **The one line a step always gets**: its address, how it ended, what it
/// cost — and the retries, where there were any.
pub fn headline(row: &StepRow) -> String {
    let said = format!("{}  {} — {} tokens", row.seq, row.framing, row.tokens.total);
    if row.attempts > 1 {
        return format!("{said}, {} attempts", row.attempts);
    }
    said
}

/// The weak line under it — when it ran and what commit read it — or none
/// where the step's record carried neither.
pub fn provenance(row: &StepRow) -> Option<String> {
    let mut parts = Vec::new();
    // A word and not an arrow: the toolkit's default font has no glyph for
    // `→` and paints a box in its place — photographed, not guessed.
    if let (Some(from), Some(to)) = (&row.started_at, &row.ended_at) {
        parts.push(format!("{from} to {to}"));
    }
    if let Some(commit) = &row.commit {
        parts.push(format!("at {commit}"));
    }
    (!parts.is_empty()).then(|| parts.join("  "))
}

/// The wound, said once: the class verbatim, and the adapter's own words
/// where it left any.
pub fn wounded(row: &StepRow) -> Option<String> {
    if row.wound == crate::reply::steps::NONE {
        return None;
    }
    Some(match &row.wound_reason {
        Some(reason) => format!("wound: {} — {reason}", row.wound),
        None => format!("wound: {}", row.wound),
    })
}

/// The sign-in affordance: offered at all, and the provider row it points at
/// when one was derivable.
///
/// **Offered on the wound's `refused` arm, not on a flag beside it.** The flag
/// (`auth_failed`) was deleted from the row at PROTOCOL 9 because it said what
/// the wound class already said, and two spellings of one fact are two things
/// that can disagree — so the affordance and [`wounded`] above now read the
/// same word, and a step cannot offer a sign-in while its badge denies there
/// was a refusal.
pub fn auth(row: &StepRow) -> Option<String> {
    if row.wound != crate::reply::steps::REFUSED {
        return None;
    }
    Some(match &row.auth_row {
        Some(provider) => format!("a sign-in is wanted on {provider}"),
        None => "a sign-in is wanted".to_owned(),
    })
}

#[cfg(test)]
mod tests;
