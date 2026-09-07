//! **The cascade: the stop that takes the subtree with it** (bl-3686;
//! bl-9fd1's window half).
//!
//! # It is here because this is the pane that says what is under there
//!
//! `crate::ui::composer::acts` fires the BARE stop and
//! `crate::ui::convs::menu` fires it off a row, and neither is the place for
//! this one. The composer's row and a list row's menu are routine surfaces —
//! DESIGN §4.20's *"a surface an operator moves through quickly; a mis-aimed
//! click there must not be able to land on this"* — and a cascade acts on
//! conversations that are **not** the one the control names. This pane is
//! where those conversations are stated: the header's descent line says what
//! it hangs under, the spine's cards say what was dispatched off it, and the
//! offers line beside this control is the engine's own sentence about whether
//! the cascade is available at all.
//!
//! # The arming is an ENABLEMENT, because the wire has no field for it
//!
//! §4.20 draws its arming rule off the wire's own grammar: where an act
//! carries a `typed` parameter the seat leaves it a parameter
//! (`delete-agent`), and where the engine refuses without a name the seat
//! makes the control an **enablement** (`delete-workspace`). `stop` carries
//! neither — the cascade is a boolean flag the engine takes on trust — so the
//! arming here is this seat's own, and §4.20 is amended to say what earns one:
//! the test is **scope**, not destruction. yog's §3.6 doctrine is already
//! written that way (*"typed-name confirm iff the verb … destroys objects
//! beyond the one named on screen"*), and a subtree stop is beyond the one
//! named on screen whatever it leaves standing.
//!
//! **It is never spent on firing**, for §4.20's reason: the refusal is the
//! common answer on this control's own surface, and clearing the box would
//! charge a retype for the engine's *no*.
//!
//! # The word is the ENGINE's word, read once
//!
//! The button's label is [`crate::ui::records::header::word`] of
//! [`Offer::Children`] — the same string the offers sentence one label to its
//! left is composed from. One home: a control whose label restated the offer
//! would be a second spelling of it, and the two would drift the first time
//! either was reworded.
//!
//! # Retiring the boundary field deletes ONE line
//!
//! litany bl-3114 made every engine stop take its children, so yog will retire
//! the `children` field at its next PROTOCOL bump (yog bl-6efc). The field has
//! exactly one site in this window that means *yes* —
//! `crate::ui::model::Model::post_cascade`'s `true` — and that literal is what
//! the bump deletes. What the bump does NOT do is settle where the arming
//! goes: once every stop is this act, the bare controls become it too, and the
//! arming has to move to them. That is a seat ball off the bump, and DESIGN
//! §4.32 records it rather than leaving it to be discovered by an operator.

use crate::reply::agent::{Agent, Offer};

/// **What the arming box asks for**, and it is the whole of the refusal too.
///
/// §4.20 wants the refusal spelled beside a disabled control, *"because a
/// greyed control says a thing is not live and nothing about what would make
/// it live"* — and the hint inside the box that fills it is that sentence, in
/// the one place an operator is already looking when they ask the question. A
/// second run saying it again is a second run this pane has no room for: it
/// covers the window, seven halves ride under one scroll, and a control laid
/// out past the frame is unreachable (`crate::snapshot::clipped`, DESIGN
/// §4.32). Measured, not assumed — the sentence beside the box put two nodes
/// off the bottom of the narrow shape.
pub const ARM: &str = "its name, to take the subtree";
/// **What the arming box is worth**, in points, on
/// `crate::ui::composer::acts::ARM_WIDTH`'s own reasoning: this pane covers
/// the window and a box that took the rest of the line would push the control
/// it arms off the row.
const ARM_WIDTH: f32 = 160.0;

/// **Whether the typed box arms the cascade** — the display name back, with
/// surrounding whitespace forgiven and nothing else, which is the engine's own
/// rule for the arming `delete-agent` takes (yog's `src/delete/agent.rs`).
pub fn armed(row: &Agent, typed: &str) -> bool {
    typed.trim() == row.display
}

/// Paint the cascade beside the offers line, where the engine offers it.
///
/// A row the engine offers no cascade on carries no control at all, which is
/// the same reading `crate::ui::records::spine`'s fork control takes off an
/// unanchored notch: the control is absent where its subject is, and disabled
/// only where a parameter is.
pub fn control(ui: &mut egui::Ui, model: &mut crate::ui::Model, row: &Agent) {
    if !row.offers.contains(&Offer::Children) {
        return;
    }
    ui.add(
        egui::TextEdit::singleline(&mut model.cascade)
            .desired_width(ARM_WIDTH)
            .hint_text(ARM),
    );
    let ready = armed(row, &model.cascade);
    let fire = ui.add_enabled(
        ready,
        egui::Button::new(super::header::word(Offer::Children)),
    );
    crate::ui::act::tag(&fire, &[crate::verbs::STOP.word]);
    if fire.clicked() {
        model.post_cascade();
    }
}

#[cfg(test)]
mod tests;
