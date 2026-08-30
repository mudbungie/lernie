//! **The composer**: what an operator types, and the gesture it becomes.
//!
//! It **composes and does not send**. The gesture lands in
//! [`Model::outbox`](crate::ui::Model::outbox) and whoever can reach a socket
//! drains it, because a frame that posted its own act is a frame that waits on
//! one — and a window that waits is the one failure a seat has no excuse for.
//!
//! The envelope is built by [`crate::verbs`], the same rows `lernie message`
//! spends, through a door whose arity is its signature. So a click and a typed
//! command build one object and there is no second spelling of a gesture to
//! drift.

use crate::ui::Model;

/// What the composer says with nothing to say it to. It names **both** halves
/// of the address, because either can be the one that is missing and a bare
/// "nothing selected" makes the operator guess which.
pub const NOWHERE: &str = "pick a workspace and a conversation to say anything";
/// The verb on the button, and the word the refusal above is about.
pub const SEND: &str = "send";
/// The other act a conversation affords from here: start a driver on one that
/// has gone quiet. It is beside the composer rather than in a menu because it
/// is the one thing an operator does to a conversation with **nothing to say**
/// — and a control for that case belongs where the case is looked at.
pub const NUDGE: &str = "nudge";

/// Paint the composer and take what it was given.
pub fn render(ui: &mut egui::Ui, model: &mut Model) {
    let (Some(aim), Some(agent)) = (model.aim.clone(), model.conversation.clone()) else {
        ui.label(NOWHERE);
        return;
    };
    let entry = ui.add(
        egui::TextEdit::singleline(&mut model.draft)
            .desired_width(f32::INFINITY)
            .hint_text(SEND),
    );
    // **Enter sends, and the button says so.** A composer that could only be
    // fired by pointing at it is a composer an operator has to leave the
    // keyboard for, once per message; a button that is the only way to discover
    // Enter exists is why both are here.
    let entered = entry.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));
    let mut sent = entered;
    let mut nudged = false;
    ui.horizontal(|ui| {
        sent |= ui.button(SEND).clicked();
        nudged = ui.button(NUDGE).clicked();
    });
    if sent {
        fire(model, &aim.address, &agent);
    }
    if nudged {
        model
            .outbox
            .push(crate::verbs::nudge(aim.address.clone(), agent.clone()));
    }
}

/// **Compose the deposit and clear the draft.**
///
/// An empty draft fires nothing: the content crosses verbatim and an empty
/// message is a turn nobody asked for. The draft is cleared only where
/// something was actually composed, so a mis-click never costs what was typed.
fn fire(model: &mut Model, workspace: &str, agent: &str) {
    if model.draft.trim().is_empty() {
        return;
    }
    let said = std::mem::take(&mut model.draft);
    model.outbox.push(crate::verbs::message(
        workspace.to_owned(),
        agent.to_owned(),
        said,
    ));
}

#[cfg(test)]
mod tests;
