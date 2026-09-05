//! **The block a ball's text is authored in**, and the four acts that hang off
//! it (bl-f7ae; DESIGN §4.35).
//!
//! Split from [`super::wall`] at the design-time budget on the seam the pane
//! draws: that section is what the aimed wall HOLDS, and this is what an
//! operator would do to one of them. It is one block with two subjects — a
//! ball that does not exist yet and a ball this wall holds — because authoring
//! a ball's words is one act, which is the fold upstream's own
//! `actions/verbs/balls/edit` already made.
//!
//! # The way out comes first, and it says what it keeps
//!
//! §4.20's rule, and it is the rule here for the reason it is there: the
//! control an operator reaches for by reflex must be the one that changes
//! nothing. Escape reaches it too, on `Model::escape`'s ladder, where the
//! block sits inside the pane rather than instead of it.
//!
//! # `close` is armed and the other three are not
//!
//! §4.20 divides on one test — *undone by doing the other thing* — and three
//! of the four pass it. `create` is undone by a close or a release of the ball
//! it filed; `update` is undone by writing the old words back, and a `note` is
//! an append that claims nothing; `release` is undone by `assign`, which is on
//! the board one section up. `close` is the one with no verb that reverses it:
//! it folds `main` into the worktree, squashes the work onto the branch and
//! removes the worktree.
//!
//! So it takes §4.20's ENABLEMENT: the control is dark until the box holds the
//! ball's own id, and **the refusal is spelled beside it**, because a greyed
//! control says a thing is not live and nothing about what would make it live.
//! It does not take §4.20's other half — a covering pane of its own — and that
//! is an argument rather than an omission. That pane exists because
//! `delete-workspace`'s subject is a row in a routine surface an operator moves
//! through quickly, *"and a mis-aimed click there must not be able to land on
//! this"*. This block is already two deliberate gestures deep — a covering
//! pane, then a control on one row of it — and the arming is a third; a click
//! cannot arrive here by accident, and the id typed back is what says which
//! ball the operator meant.
//!
//! **The arming names its own subject**, which is why nothing records which
//! ball is armed: the box holds an id, and the control it enables is the one
//! on the ball with that id. And it is never spent on firing (§4.20): a
//! refusal is the common case — the engine declines while the gate fails — and
//! clearing the box would charge a retype for the engine's *no*.

use crate::ui::{Authoring, Model, keys, theme};

/// The control that opens the block on a ball that does not exist yet.
pub const FILE: &str = "file a ball…";
/// The control that opens it on a ball this wall holds.
pub const ACT: &str = "act on it…";
/// The control that claims a ready ball for the aimed wall — on a board row
/// rather than in the block, because the row is its subject.
pub const CLAIM: &str = "claim it for this wall";
/// The way out, which changes nothing. It is not the pane's own `done`,
/// because two controls reading the same word on one screen are one control to
/// everything that aims at a word — the operator, the accessibility tree and
/// the harness alike.
pub const DONE: &str = "done with this ball";
/// What the block says it is about, with no ball.
pub const NEW: &str = "a ball that does not exist yet";
/// The control that files it.
pub const FILE_IT: &str = "file it";
/// The control that amends it.
pub const AMEND: &str = "amend it";
/// The control that lets it go.
pub const RELEASE: &str = "let it go";
/// The control that delivers it.
pub const DELIVER: &str = "deliver it";
/// What the project box asks for.
pub const PROJECT_HINT: &str = "project";
/// What the title box asks for.
pub const TITLE_HINT: &str = "title";
/// What the body box asks for.
pub const BODY_HINT: &str = "body";
/// What the journal box asks for.
pub const NOTE_HINT: &str = "a note for its journal";
/// What the arming box asks for.
pub const ARM_HINT: &str = "the ball's id";
/// The refusal spelled beside a filing that cannot go.
pub const UNFILEABLE: &str = "a project and a title are what files a ball";
/// The refusal spelled beside an amendment that would change nothing.
pub const UNAMENDABLE: &str = "type a title, a body or a note to amend it";
/// **What an act with no missing parameter says**, which is nothing: `release`
/// is offered whenever there is a ball to release.
pub const NOTHING: &str = "";
/// The refusal spelled beside the one act that is armed.
pub const UNARMED: &str = "type the ball's id to deliver it — a close cannot be undone";

/// Paint the block, if one is open, and take the clicks on it.
pub fn render(ui: &mut egui::Ui, model: &mut Model) {
    let Some(down) = model.channel() else {
        return;
    };
    let Some(block) = model.authoring.as_ref() else {
        return;
    };
    ui.separator();
    ui.label(subject(block));
    ui.horizontal_wrapped(|ui| {
        if ui.button(DONE).clicked() {
            model.close_authoring();
        }
    });
    boxes(ui, model);
    if let Some(fired) = acts(ui, model) {
        model.post_ball(&down, fired);
    }
}

/// **What the block is about**, said before it offers anything — §4.20's rule,
/// because no control has room to name its subject and an act on the wrong
/// ball is the thing a sentence here prevents.
pub fn subject(block: &Authoring) -> String {
    block.ball.as_ref().map_or_else(
        || format!("{NEW} — {}", block.at.address),
        |ball| format!("{} in {} as {}", ball.id, ball.project, block.name),
    )
}

/// The boxes, bound to the block's own words rather than to a copy compared
/// afterwards: a draft is what the block holds, so an edit is a write to it and
/// there is no second value for the two to disagree about.
fn boxes(ui: &mut egui::Ui, model: &mut Model) {
    let Some(block) = model.authoring.as_mut() else {
        return;
    };
    let filing = block.filing();
    if filing {
        typed(ui, &mut block.project, keys::PROJECT_ID, PROJECT_HINT);
    }
    typed(ui, &mut block.title, keys::TITLE_ID, TITLE_HINT);
    typed(ui, &mut block.body, keys::BODY_ID, BODY_HINT);
    if !filing {
        typed(ui, &mut block.note, keys::NOTE_ID, NOTE_HINT);
        typed(ui, &mut block.arm, keys::DELIVER_ID, ARM_HINT);
    }
}

/// One box that takes text, with the id the keyboard's own gate compares
/// against (`crate::ui::keys`).
fn typed(ui: &mut egui::Ui, text: &mut String, id: &str, hint: &str) {
    ui.add(
        egui::TextEdit::singleline(text)
            .id(egui::Id::new(id))
            .desired_width(f32::INFINITY)
            .hint_text(hint),
    );
}

/// The four acts, each dark until its own gesture composes — the enablement
/// and the envelope read off one fact rather than two
/// (`crate::ui::model::board::acts`). Answers the gesture a click composed, if
/// one did.
///
/// **The order is §4.20's**: what changes a thing first, what ends it under.
fn acts(ui: &mut egui::Ui, model: &Model) -> Option<serde_json::Value> {
    let block = model.authoring.as_ref()?;
    let offered = if block.filing() {
        vec![(FILE_IT, crate::verbs::CREATE, block.filed(), UNFILEABLE)]
    } else {
        vec![
            (AMEND, crate::verbs::UPDATE, block.amended(), UNAMENDABLE),
            (
                RELEASE,
                crate::verbs::RELEASE.word,
                block.released(),
                NOTHING,
            ),
            (
                DELIVER,
                crate::verbs::CLOSE.word,
                block.delivered(),
                UNARMED,
            ),
        ]
    };
    let mut fired = None;
    ui.horizontal_wrapped(|ui| {
        for (word, op, gesture, why) in offered {
            fired = spend(ui, word, op, gesture, why).or(fired.take());
        }
    });
    fired
}

/// **One act: its word, its parity token, the gesture it would send and the
/// sentence that says why it cannot.**
///
/// The gesture's absence IS the enablement, so a control can never be live and
/// unable to compose. An empty `why` is an act with no missing parameter —
/// `release` is offered whenever there is a ball — and prints nothing.
fn spend(
    ui: &mut egui::Ui,
    word: &str,
    op: &str,
    gesture: Option<serde_json::Value>,
    why: &str,
) -> Option<serde_json::Value> {
    let control = ui.add_enabled(gesture.is_some(), egui::Button::new(word));
    crate::ui::act::tag(&control, &[op]);
    if gesture.is_none() && !why.is_empty() {
        ui.colored_label(theme::tone_ink(&crate::reply::convs::Tone::Weak), why);
    }
    control.clicked().then_some(gesture).flatten()
}

#[cfg(test)]
mod tests;
