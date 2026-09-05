//! **The spine half of the records pane** (bl-b52c): what the conversation's
//! history is anchored to, and the one act composed off it.
//!
//! Split from [`super`] at the design-time budget on the seam the pane's own
//! subject draws: that file is what the loop DID and what it TOUCHED, and this
//! is what it is anchored TO — the operable commits, the config commit
//! governing them, and the fork whose `from` is one of the first.
//!
//! # The fork is here because its argument is only discoverable here
//!
//! `fork`'s `from` is a **ref**, and upstream refuses an empty one: *"a fork
//! with no ref is a different gesture"*. The refs a conversation offers are
//! exactly this pane's notches, so a fork control anywhere else in this window
//! would be a control demanding a string nothing on the glass can supply —
//! which is what `parity.toml`'s line for it said, in those words, until this
//! commit deleted it. So there is one control per **operable** notch
//! ([`crate::reply::rail::Notch::operable`]), it carries that notch's own
//! commit, and a notch that recorded none carries no control at all.
//!
//! # The role is typed, and the pane says why rather than implying a set
//!
//! A role names an entry litany resolves against the fork point's governing
//! config, so the honest control would offer the names that config declares —
//! and the read that lists them is the config-file pane's, which this seat
//! does not have (bl-5c53). A picker over a list this seat cannot obtain would
//! be capability theatre; a bare box with no sentence would look like a free
//! string. So it is a box with [`ROLE_SAID`] under it.

use crate::reply::rail::{Card, Notch, Rail};
use crate::reply::{convs::Tone, governing::Governing};
use crate::ui::{Model, theme};

/// The spine half's own heading.
pub const SPINE_HEAD: &str = "spine";
/// The governing half's.
pub const GOVERNING_HEAD: &str = "governing config";
/// What the spine says before the first answer.
pub const NOT_ANSWERED_SPINE: &str =
    "waiting to hear what this conversation's history is anchored to";
/// What it says for a conversation that answered and offers no commit. A fact
/// about the conversation: nothing here can be forked from yet.
pub const NO_NOTCHES: &str = "no step of its history is anchored to a commit yet";
/// What the cards say when nothing was dispatched — the honest empty case
/// upstream names, and not an error.
pub const NO_CARDS: &str = "nothing has been dispatched from this conversation";
/// What the governing half says before the first answer.
pub const NOT_ANSWERED_GOVERNING: &str = "waiting to hear which config governs it";
/// What the governing half says of a commit whose tree it listed as empty.
pub const NO_FILES: &str = "its tree holds no file";
/// What the role box asks for.
pub const ROLE_HINT: &str = "role";
/// What the goal box asks for.
pub const GOAL_HINT: &str = "what the fork is for";
/// **Why the role is typed and not chosen.** A role resolves against the fork
/// point's own config, and the lineages have no pane in this seat.
pub const ROLE_SAID: &str =
    "a role is a name the fork point's config declares, and this seat cannot list them yet";

/// Paint the spine half: the governing commit, the draft, the notches and the
/// cards hanging off them.
pub fn render(ui: &mut egui::Ui, model: &mut Model) {
    governing_half(ui, model.records.governing.clone().as_ref());
    ui.separator();
    let Some(spine) = model.records.rail.clone() else {
        ui.label(egui::RichText::new(SPINE_HEAD).strong());
        ui.label(NOT_ANSWERED_SPINE);
        return;
    };
    draft(ui, model);
    if spine.notches.is_empty() {
        ui.label(NO_NOTCHES);
    }
    for row in &spine.notches {
        notch(ui, model, row);
    }
    cards(ui, &spine);
}

/// The governing commit: the engine's own sentence, then the paths its tree
/// holds.
fn governing_half(ui: &mut egui::Ui, answer: Option<&Governing>) {
    let Some(config) = answer else {
        ui.label(egui::RichText::new(GOVERNING_HEAD).strong());
        ui.label(NOT_ANSWERED_GOVERNING);
        return;
    };
    ui.horizontal_wrapped(|ui| {
        ui.label(egui::RichText::new(GOVERNING_HEAD).strong());
        ui.label(format!("{} — {}", config.label(), config.oid));
    });
    ui.horizontal_wrapped(|ui| {
        if config.files.is_empty() {
            ui.label(NO_FILES);
            return;
        }
        for path in &config.files {
            ui.label(path.clone());
        }
    });
}

/// The two boxes a fork is composed from, and the sentence under the first.
fn draft(ui: &mut egui::Ui, model: &mut Model) {
    ui.horizontal_wrapped(|ui| {
        ui.label(egui::RichText::new(SPINE_HEAD).strong());
        ui.add(egui::TextEdit::singleline(&mut model.forking.role).hint_text(ROLE_HINT));
        ui.add(egui::TextEdit::singleline(&mut model.forking.goal).hint_text(GOAL_HINT));
        ui.colored_label(theme::tone_ink(&Tone::Weak), ROLE_SAID);
    });
}

/// One notch: its line, where the chat seats it, and the fork it can carry.
fn notch(ui: &mut egui::Ui, model: &mut Model, row: &Notch) {
    ui.horizontal_wrapped(|ui| {
        ui.label(headline(row));
        if let Some(said) = seated(row) {
            ui.colored_label(theme::tone_ink(&Tone::Weak), said);
        }
        let Some(commit) = row.commit.clone() else {
            return;
        };
        // **Disabled and not absent**, for the reason the composer's `flag`
        // is: the parameters are missing, not the subject, so the control
        // stays on the glass saying what would fill it.
        let fire = ui.add_enabled(model.forking.ready(), egui::Button::new(forking(row)));
        crate::ui::act::tag(&fire, &[crate::verbs::spine::FORK]);
        if fire.clicked() {
            model.post_fork(commit);
        }
    });
}

/// The cards hanging off the spine, or the sentence for a conversation nobody
/// forked from.
fn cards(ui: &mut egui::Ui, spine: &Rail) {
    if spine.cards.is_empty() {
        ui.label(NO_CARDS);
        return;
    }
    for row in &spine.cards {
        ui.horizontal_wrapped(|ui| {
            ui.label(card(row));
            if let Some(said) = &row.tail {
                ui.colored_label(theme::tone_ink(&Tone::Weak), said.clone());
            }
        });
    }
}

/// **The one line a notch always gets**: its step, the commit it read against
/// — or the engine's own word for having none — and the spend as of it.
pub fn headline(row: &Notch) -> String {
    format!("{}  {} — {} tokens", row.seq, row.short(), row.budget)
}

/// Where the chat seats this notch, or none where it has no seat: a call that
/// sealed nothing and was superseded is a point on the spine the chat never
/// drew a rule for.
pub fn seated(row: &Notch) -> Option<String> {
    let seat = row.seat.as_ref()?;
    Some(format!(
        "above {} — it had read {} entries",
        seat.row, seat.cut
    ))
}

/// The word on the control that forks from this notch. It names the commit it
/// carries, so two notches never offer one label and an operator reads what
/// they are about to fork off before clicking it.
pub fn forking(row: &Notch) -> String {
    format!("fork from {}", row.short())
}

/// One child card as a line: who it is, where it forked from, what it is doing
/// and what it has spent.
pub fn card(row: &Card) -> String {
    format!(
        "{} — {} — {} — {} tokens",
        row.name,
        row.fork,
        row.state.label(),
        row.tokens
    )
}

#[cfg(test)]
mod tests;
