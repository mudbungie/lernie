//! **The start half of the composer**: where a conversation is begun.
//!
//! It is a **mode of the composer and not a pane beside it**, and that is this
//! ball's one layout decision. A wall with no conversation selected already had
//! a box that refused — *pick a workspace and a conversation to say anything* —
//! and that refusal was the start's own case wearing a sentence: there is
//! nothing to say to, so the thing to do is begin one. A second box would be
//! the same box twice, each with its own Enter, on a face that is four panes
//! wide; upstream states the rule as *one box, one Enter* and it is worth more
//! here than there.
//!
//! So the composer is one control with two subjects, decided by what is
//! selected and by nothing else:
//!
//! - no wall aimed at → nothing to say and nothing to begin;
//! - a wall, no conversation → **this**, the start;
//! - a wall and a conversation → the deposit.
//!
//! **A start in flight paints its sentence instead of the box**, which is how a
//! second start is refused: not by a disarmed control, but by there being no
//! control. The receipt is the one state that paints the sentence *and* the box
//! — the minted name stays readable while the operator begins the next one.

use crate::ui::{Aim, Model};

/// The word on the control that begins a conversation.
pub const START: &str = "start";
/// What the box asks for. A start's goal is what the conversation is *for*,
/// which is a different question from what to say to one that exists.
pub const GOAL: &str = "what this conversation is for";

/// Paint the start composer and take what it was given.
pub fn render(ui: &mut egui::Ui, model: &mut Model, aim: &Aim) {
    if let Some(held) = model.start.clone() {
        ui.label(held.line());
        if held.outstanding() {
            return;
        }
    }
    let entry = ui.add(
        egui::TextEdit::singleline(&mut model.draft)
            .id(egui::Id::new(crate::ui::keys::BOX_ID))
            .desired_width(f32::INFINITY)
            .hint_text(GOAL),
    );
    // Enter begins it, and the button beside it is how an operator finds that
    // out — the same pairing the deposit's own Enter has, for the same reason.
    let mut fired = entry.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));
    fired |= ui.button(START).clicked();
    if fired {
        model.stage(&aim.address);
    }
}

#[cfg(test)]
mod tests;
