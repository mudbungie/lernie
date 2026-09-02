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
//!
//! **One box with two subjects, decided by what is selected.** A wall with a
//! conversation on it is spoken to; a wall with none is where one is *begun*
//! ([`start`]) — which used to be a refusal, and was the start's own case
//! wearing a sentence. A second box beside this one would be the same box
//! twice, each with its own Enter.
//!
//! **The one case with no box at all** is a conversation this window started
//! and the engine cannot resolve yet: there is a selection, and nothing this
//! seat composed against it would be answered
//! (`crate::ui::model::claim`).

use crate::ui::Model;

/// The half that begins a conversation rather than continuing one.
pub mod start;

/// What the composer says with no wall aimed at — the one case that is neither
/// a deposit nor a start, because there is nowhere for either to go.
pub const NOWHERE: &str = "pick a workspace to say anything or begin anything";
/// The verb on the button, and the word the refusal above is about.
pub const SEND: &str = "send";
/// The other act a conversation affords from here: start a driver on one that
/// has gone quiet. It is beside the composer rather than in a menu because it
/// is the one thing an operator does to a conversation with **nothing to say**
/// — and a control for that case belongs where the case is looked at.
pub const NUDGE: &str = "nudge";

/// Paint the composer and take what it was given.
pub fn render(ui: &mut egui::Ui, model: &mut Model) {
    let Some(aim) = model.aim.clone() else {
        ui.label(NOWHERE);
        return;
    };
    let Some(agent) = model.conversation.clone() else {
        start::render(ui, model, &aim);
        return;
    };
    // **A conversation this window has started is not addressable yet**, and
    // this seat knows it: the minted name resolves nowhere until its driver
    // writes the branch (`crate::ui::model::claim`). So the box and both its
    // buttons stand down and the start's own sentence stands in their place —
    // a gesture composed here would be one this end knew the engine would
    // refuse. It is not a wedge: the claim retires on the engine's next
    // listing, and one arrow key leaves it before then.
    if let Some(held) = model.start.clone().filter(|_| model.pending().is_some()) {
        ui.label(held.line());
        return;
    }
    let entry = ui.add(
        egui::TextEdit::singleline(&mut model.draft)
            .id(egui::Id::new(crate::ui::keys::BOX_ID))
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
    // **WRAPPED, because this pane is not as wide as the window** (bl-dc07).
    // The composer is a bottom panel added AFTER both side panels, so what it
    // gets is what they left: at a 400-point window the two lists are on their
    // floor (`crate::ui::shell::SIDE_FLOOR`) and this row is laid out in about
    // 120 points. An unwrapped row lays its buttons on one line however long
    // they are, so `nudge` went off the right edge of the window entirely —
    // painted, correct, and unreachable by any pointer. Wrapping costs a
    // second line in a panel already sized to its content.
    ui.horizontal_wrapped(|ui| {
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
