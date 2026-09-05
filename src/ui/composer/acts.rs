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
//!   conversation for another lineage, [`FLAG`] asks the operator to look at
//!   it later, [`DELETE`] unmakes it.
//!
//! **The flag is on this row and not on the queue pane** (bl-f0ef), which is
//! the one placement decision that ball made here. A flag is *somebody asking
//! the operator to look at this conversation*, so it is raised while looking at
//! it — and `crate::ui::queue` covers the conversation, so a control there
//! would flag something the operator cannot see. The queue is where a flag is
//! READ; this is where one is raised.
//!
//! **Each of the three answers a captured run**, which is why they are here at
//! all rather than in the exemption ledger beside the conversation's records: a
//! reply this seat already paints is the difference between a control that
//! answers and one that earns *"this build cannot read that kind"*
//! (`crate::verbs::conversation`).

use crate::ui::{Aim, Fill, Model, keys};

/// The word that kills the driver. `nudge`'s opposite, and the reason it sits
/// one row below rather than beside it: nudge leaves a driver running.
pub const STOP: &str = "stop";
/// The word that marks the conversation for its lineage's head.
pub const RETARGET: &str = "retarget";
/// The word that raises an attention item on it.
pub const FLAG: &str = "flag";
/// **What the reason box asks for**, and it asks for what the wire requires:
/// `flag` takes a reason and refuses without one, so the control beside this
/// box is disabled until it holds something.
pub const WHY: &str = "why it wants a look";
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
    // **Taken once, before either box is painted** (bl-dbc9). A conversation
    // row's menu cannot hold either of these boxes, so its two parameterized
    // items send the operator here and ask for the cursor
    // (`crate::ui::model::fill`). One read rather than one per box: a frame
    // that paints this row spends the whole request, so half of one cannot be
    // left standing for the next.
    let wanted = model.filling();
    ui.horizontal_wrapped(|ui| {
        // **The reads this gesture reaches** (bl-2cf7, bl-b52c), exactly as the
        // wall's roster seat carries the conversation list's: opening the
        // records pane is what makes this seat read the selected
        // conversation's steps, its files, its spine and the config commit
        // governing it (`crate::state::Standing`), and the four reads have no
        // control of their own. It leads the row because it spends nothing — every
        // control after it acts on the conversation, this one only looks.
        let records = ui.button(crate::ui::records::OPEN);
        crate::ui::act::tag(
            &records,
            &[
                crate::verbs::STEPS.word,
                crate::verbs::FILES.word,
                crate::verbs::RAIL.word,
                crate::verbs::GOVERNING.word,
            ],
        );
        if records.clicked() {
            model.begin_records();
        }
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
        // **Each box wears an id**, which is what lets the keyboard's gate name
        // every box that takes text rather than only the draft
        // (`crate::ui::keys::BOXES`) — an arrow taken from inside a half-typed
        // reason would otherwise have walked the conversation list under it.
        let why = ui.add(
            egui::TextEdit::singleline(&mut model.reason)
                .id(egui::Id::new(keys::REASON_ID))
                .desired_width(ARM_WIDTH)
                .hint_text(WHY),
        );
        if wanted == Some(Fill::Reason) {
            why.request_focus();
        }
        // **Disabled and not absent**, which is the tuning pane's `set` rather
        // than the composer's second start: the parameter is missing, not the
        // subject, so the control stays on the glass saying what would fill it.
        let raise = ui.add_enabled(!model.reason.trim().is_empty(), egui::Button::new(FLAG));
        crate::ui::act::tag(&raise, &[crate::verbs::FLAG.word]);
        if raise.clicked() {
            // **The reason is TAKEN**, unlike the arming below: a flag that
            // fired is said, exactly as a deposit is, and the next flag on this
            // conversation is a different sentence about a different moment.
            fired = Some(crate::verbs::flag(
                aim.address.clone(),
                agent.to_owned(),
                std::mem::take(&mut model.reason),
            ));
        }
        let arming = ui.add(
            egui::TextEdit::singleline(&mut model.typed)
                .id(egui::Id::new(keys::ARM_ID))
                .desired_width(ARM_WIDTH)
                .hint_text(ARM),
        );
        if wanted == Some(Fill::Arming) {
            arming.request_focus();
        }
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
    model
        .outbox
        .extend(fired.into_iter().map(crate::ui::Posted::act));
}

#[cfg(test)]
mod tests;
