//! **The acts that spend no words** — what an operator does to the selected
//! conversation as an *object*, rather than to the turn it is taking (bl-213c).
//!
//! It is a second row of the composer and not a strip of its own, and that is
//! this ball's one layout decision. The composer is already *the pane that acts
//! on the selected conversation*: it holds the one box, and every verb that
//! needs words spends that box. Putting these three anywhere else would be a
//! second place to look for the same subject — and the chat pane, the only
//! other candidate, is a **pure projection** of the transcript
//! (`crate::ui::chat`), so a control there would be the first thing in it that
//! is not.
//!
//! So the split between the two rows is by what the act does to the turn, which
//! is the distinction an operator is actually making:
//!
//! - **row one advances it** — `send`, `interrupt`, `nudge`. Each ends with a
//!   driver running, and the first two spend the box.
//! - **row two does not** — [`STOP`] kills the driver, [`RETARGET`] marks the
//!   conversation for another lineage, [`DELETE`] unmakes it.
//!
//! **Each of the three answers a captured run**, which is why they are here at
//! all rather than in the exemption ledger beside the conversation's records: a
//! reply this seat already paints is the difference between a control that
//! answers and one that earns *"this build cannot read that kind"*
//! (`crate::verbs::conversation`).

use crate::ui::{Aim, Model};

/// The word that kills the driver. `nudge`'s opposite, and the reason it sits
/// one row below rather than beside it: nudge leaves a driver running.
pub const STOP: &str = "stop";
/// The word that marks the conversation for its lineage's head.
pub const RETARGET: &str = "retarget";
/// The word that unmakes it.
pub const DELETE: &str = "delete";
/// **What the arming box asks for**, and it asks for exactly what the wire
/// means: an empty box deletes the one conversation, and its name typed back is
/// what admits the descendants (`crate::verbs::DELETE_AGENT`).
pub const ARM: &str = "its name, to take its children too";
/// **What the arming box is worth**, in points. Fixed rather than infinite: the
/// composer is a bottom panel laid out in what the two side panes left, and a
/// box that took the rest of the line would push `delete` off the row at every
/// width the window actually opens at (the wrap bl-dc07 landed is what saves
/// the button, and a box this wide is what keeps them on one line at all).
/// Wide enough that the hint above is READ rather than elided at the narrowest
/// width the layout still promises a shape for — a box whose label is cut off
/// mid-sentence is a box an operator has to guess the meaning of, and guessing
/// wrong here arms a cascade.
const ARM_WIDTH: f32 = 200.0;

/// Paint the row and take what it was given.
pub fn render(ui: &mut egui::Ui, model: &mut Model, aim: &Aim, agent: &str) {
    let mut fired = None;
    ui.horizontal_wrapped(|ui| {
        let halt = ui.button(STOP);
        crate::ui::act::tag(&halt, &[crate::verbs::STOP.word]);
        if halt.clicked() {
            fired = Some(crate::verbs::stop(aim.address.clone(), agent.to_owned()));
        }
        let settle = ui.button(RETARGET);
        crate::ui::act::tag(&settle, &[crate::verbs::RETARGET.word]);
        if settle.clicked() {
            fired = Some(crate::verbs::retarget(
                aim.address.clone(),
                agent.to_owned(),
            ));
        }
        ui.add(
            egui::TextEdit::singleline(&mut model.typed)
                .desired_width(ARM_WIDTH)
                .hint_text(ARM),
        );
        let unmake = ui.button(DELETE);
        crate::ui::act::tag(&unmake, &[crate::verbs::DELETE_AGENT.word]);
        if unmake.clicked() {
            fired = Some(crate::verbs::delete_agent(
                aim.address.clone(),
                agent.to_owned(),
                model.typed.clone(),
            ));
        }
    });
    // **The arming is CLONED, not taken**, which is the one place this row
    // parts from the composer's own rule that a fired deposit clears the box.
    // A deposit that fired is said; a delete is refused outright while the
    // conversation is live, and that refusal is the common case for a control
    // an operator reaches for on a conversation that is still working. Clearing
    // the name would charge them a retype for the engine's *no*, which is a
    // toll on the safe path — where clearing a sent message costs nothing,
    // because it was sent.
    model.outbox.extend(fired);
}

#[cfg(test)]
mod tests;
