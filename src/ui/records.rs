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
//! Every line a half paints comes off a pure function of the row, so the suite
//! reads the sentence rather than the layout — the same reason
//! `RoleRow::runs_on` is a method. The class tokens ride verbatim
//! (`crate::reply` rung 3): a `framing` or a `wound` this build has no word
//! for paints as itself. The two halves this file used to hold are modules of
//! their own (bl-d1ae), and their sentences travel with the paint that says
//! them; they are re-exported here, because a sentence's home on the crate's
//! surface is the pane it is read on.

//! # The vertical budget, measured (bl-d1ae)
//!
//! Six sections cost 42 points each — `L` of air, a hairline and the word's
//! own line — where the rules they replaced cost ten. At 900x700 this pane is
//! 420 points wide, because the two list columns keep 480, so nearly every
//! sentence here wraps to two lines and the answered world stands 789 points
//! tall in a 700-point window. What could be bought back inside the pane was:
//! the close rides the subject's line, the header's notices share one row and
//! its figures the next, the listing joins where the work lands, a deposit's
//! header joins what it says, the fork's boxes carry a width, and a step with
//! no wound paints no row for one. The nodes that still fall below the fold
//! there are LABELS — every CONTROL is inside the window at both sizes — and
//! `crate::snapshot::clipped` judges prose at all only because egui's
//! `interaction.selectable_labels` makes a label a click target. The room
//! that is left is not in this pane; it is the width next door.

/// The cascade: the stop that takes the subtree with it.
pub mod cascade;
/// One step's records, under the row that addresses them.
pub mod drill;
/// The files half: where the work lands and what the worktree holds.
mod files;
/// The conversation's own row, as the pane's header.
pub mod header;
/// The undelivered mail waiting in its inbox.
pub mod mail;
/// The spine half: what the history is anchored to, and the fork off it.
pub mod spine;
/// The steps half: what the loop did, and the sentences a step carries.
mod steps;

use crate::ui::{Model, theme};

pub use files::{entry, previewed};
pub use steps::{auth, headline, orphaned, provenance, wounded};

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
    // **The heading, then one line carrying the subject and the way out**
    // (`docs/STYLE.md` §5, *a covering pane*): the pane's name at heading size
    // in full ink, what it is open ON one step weaker beside it, and the close
    // at the end of that same line. Three lines became two, which is the air
    // the sections below are spent from.
    ui.label(egui::RichText::new(HEADING).heading().color(theme::INK));
    ui.horizontal(|ui| {
        if let Some(conversation) = model.conversation.clone() {
            ui.colored_label(theme::INK_WEAK, format!("on {conversation}"));
        }
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if ui.button(CLOSE).clicked() {
                model.close_records();
            }
        });
    });
    // **No air under the subject line**, because the first section brings `L`
    // of its own and this pane pays for every point it allocates.
    //
    // One scroll for every half, and the heading above it fixed — the shape
    // every pane here keeps, for the reason the tuning pane states: a pane cut
    // off mid-row says nothing about having been cut. **Each half opens with
    // its own section** (`theme::paint::section`) rather than being divided by
    // a rule painted here, so the word and the hairline that divide two halves
    // are one thing on the glass and one call in the code.
    let (steps, files) = (model.records.steps.clone(), model.records.files.clone());
    egui::ScrollArea::vertical()
        .id_salt(HEADING)
        .auto_shrink(false)
        .show(ui, |ui| {
            header::render(ui, model);
            steps::half(ui, model, steps.as_ref());
            files::half(ui, files.as_ref());
            spine::render(ui, model);
            mail::render(ui, model);
        });
    true
}

#[cfg(test)]
mod tests;
