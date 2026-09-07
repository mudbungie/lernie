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

use crate::ui::{Model, theme};

/// The acts that spend no words and are not offered every day: the strip
/// behind the more control.
pub mod acts;
/// The acts the conversation offers on its turn, and the control that opens
/// the strip.
pub mod offers;
/// The half that begins a conversation rather than continuing one.
pub mod start;

pub use offers::{MORE, STOP};

/// What the composer says with no wall aimed at — the one case that is neither
/// a deposit nor a start, because there is nowhere for either to go.
pub const NOWHERE: &str = "pick a workspace to say anything or begin anything";
/// The verb on the control, and the word the refusal above is about.
pub const SEND: &str = "send";
/// **What the field says while it is empty** — the two keys, because a field
/// three lines tall is one an operator expects Enter to break a line in, and
/// a composer whose one binding has to be guessed is one they leave the
/// keyboard for (bl-f251).
pub const HINT: &str = "say it — Enter sends, Shift+Enter breaks a line";
/// **And what it says while the conversation is asking** (`docs/STYLE.md`
/// §2): the field is where the answer goes, so it is where the asking is said.
pub const ASKING: &str = "it is waiting on you — Enter sends";
/// The deposit that cuts first: the driver is killed and the content deposited
/// as one gesture. It spends the same box `send` does, because *what to say
/// instead* is the same question as *what to say* — a second box for it would
/// be the same box twice, which is the rule [`start`] states one level down.
pub const INTERRUPT: &str = "interrupt";
/// The other act a conversation affords from here: start a driver on one that
/// has gone quiet. It is beside the composer rather than in a menu because it
/// is the one thing an operator does to a conversation with **nothing to say**
/// — and a control for that case belongs where the case is looked at.
pub const NUDGE: &str = "nudge";

/// Paint the composer and take what it was given.
pub fn render(ui: &mut egui::Ui, model: &mut Model) {
    let Some(aim) = model.aim.clone() else {
        theme::paint::empty(ui, NOWHERE);
        return;
    };
    let Some(agent) = model.conversation.clone() else {
        start::render(ui, model, &aim);
        return;
    };
    // **A conversation this window has started is not addressable yet**, and
    // this seat knows it: the minted name resolves nowhere until its driver
    // writes the branch (`crate::ui::model::claim`). So the box and its
    // controls stand down and the start's own sentence stands in their place —
    // a gesture composed here would be one this end knew the engine would
    // refuse. It is not a wedge: the claim retires on the engine's next
    // listing, and one arrow key leaves it before then.
    if let Some(held) = model.start.clone().filter(|_| model.pending().is_some()) {
        ui.label(held.line());
        return;
    }
    // **The field is the pane's focal element** (bl-f251; `docs/STYLE.md`
    // §2): three lines tall, the send inside it, and the glow under both
    // while the conversation is asking — the attention tint and the hint
    // that says so, because the box is where the answer goes.
    let asking = asking(model, &agent);
    let (entry, deposit) = theme::paint::composer(
        ui,
        egui::Id::new(crate::ui::keys::BOX_ID),
        &mut model.draft,
        if asking { ASKING } else { HINT },
        asking.then(|| theme::tint(theme::State::Attention)),
        SEND,
    );
    // **The send wears the brand**: it is the operator's own act.
    crate::ui::act::tag(&deposit, &[crate::verbs::MESSAGE.word]);
    // **Enter sends and Shift+Enter breaks a line**, and the hint says so.
    // The field consumes only the shifted key (`theme::paint::composer`), so
    // a bare Enter reaches this read while the box holds the caret — and
    // only then, because the same key on the send control is egui's own
    // click and must not fire twice.
    //
    // **Enter is the deposit's and no other verb's.** A key that could fire
    // the cut would be a key an operator reaches by muscle memory to say
    // something and kills a running driver with; the two verbs differ by
    // exactly that, and the one with the destructive half is the one that has
    // to be pointed at (`crate::ui::model::acts` on why a binding never fires
    // what a click cannot — this is the other direction, which is allowed).
    let entered = entry.has_focus()
        && ui.input(|i| {
            i.events.iter().any(|event| {
                matches!(
                    event,
                    egui::Event::Key {
                        key: egui::Key::Enter,
                        pressed: true,
                        modifiers,
                        ..
                    } if !modifiers.shift
                )
            })
        });
    if entered || deposit.clicked() {
        fire(model, crate::verbs::message, &aim.address, &agent);
    }
    offers::render(ui, model, &aim, &agent);
}

/// **Whether the selected conversation is asking for the operator** — read
/// off the row the list already paints, so the composer and the list agree
/// about the one fact they both show.
fn asking(model: &Model, agent: &str) -> bool {
    model
        .rows()
        .iter()
        .any(|row| row.root_id == agent && row.attention > 0)
}

/// **Compose one of the two deposits and clear the draft.**
///
/// An empty draft fires nothing: the content crosses verbatim and an empty
/// message is a turn nobody asked for — and an empty *cut* is a driver killed
/// with nothing said, which is `stop`, one row down. The draft is cleared only
/// where something was actually composed, so a mis-click never costs what was
/// typed.
///
/// **The door is the parameter**, so the two verbs share this body rather than
/// branching inside it: `message` and `interrupt` take the same three named
/// strings and differ only in the word, which is exactly what a function
/// pointer carries.
pub(super) fn fire(
    model: &mut Model,
    door: fn(String, String, String) -> serde_json::Value,
    workspace: &str,
    agent: &str,
) {
    if model.draft.trim().is_empty() {
        return;
    }
    let said = std::mem::take(&mut model.draft);
    model.outbox.push(crate::ui::Posted::act(door(
        workspace.to_owned(),
        agent.to_owned(),
        said,
    )));
}

#[cfg(test)]
mod tests;
