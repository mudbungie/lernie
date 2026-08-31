//! **The keyboard**: every act this window affords, reachable without a
//! pointer (yog's `docs/QUALITY.md` F1 — *everything keyboard-operable*).
//!
//! A face an operator has to leave the keyboard for, once per selection, is a
//! face they use through the command line instead. The obligation is inherited;
//! the implementation is not, because the shape that fits four panes is not the
//! shape that fits thirty.
//!
//! # Most of it is egui's, and that is the point
//!
//! Every control here is a button or a text box, and egui already moves focus
//! between them with Tab and fires a focused one with Space or Enter. So Send,
//! Nudge, Start and the notice's dismiss are keyboard-operable with nothing
//! written — `tests` proves it rather than assuming it. What Tab cannot make
//! *usable* is a list: tabbing through thirty walls to reach the composer is
//! reachability without operability, and that is the whole of what this module
//! adds.
//!
//! # The cursor IS the selection, so there is nothing to keep in step
//!
//! A list cursor beside a selection is two highlights, two things to paint and
//! two ways to disagree. There is no cursor: **moving in a list selects**, so
//! the highlight the pointer already paints is where the keyboard is, and the
//! reads that follow a selection follow a keypress for free (the standing set
//! is derived — `crate::state::Standing`). What is left to paint is only
//! *which list the arrows belong to*, which is [`Pane`], and it is marked on
//! that pane's own heading: a focus that cannot be seen is a focus nobody can
//! use.
//!
//! # It names controls; it never adds one
//!
//! Every binding below calls the same door a click calls
//! (`crate::ui::model::acts`), and the roster walk asks the same question the
//! roster's own paint asks — so a row no pointer can aim at is a row no key can
//! aim at either. A binding that could fire something a click cannot is a
//! second surface.
//!
//! **A box that is taking text takes every key**, which is the one gate: while
//! a text box holds the focus nothing here runs, so an arrow is a cursor move
//! inside the draft and Escape is egui's own *leave the box*. Press it again
//! with no box focused and it is [`Model::escape`] below — which closes the
//! enrollment where one covers the window, and puts the notice down otherwise.
//! One key, three contexts, and the contexts never overlap.
//!
//! The gate asks for **that box by name** rather than for egui's
//! `wants_keyboard_input`, which answers *is anything focused at all* — every
//! button included. Tabbing to Send would otherwise turn the arrows off, and a
//! click focuses a control too, so the honest question is the narrow one: the
//! composer's box wears [`BOX_ID`] and the gate compares against it.

use crate::ui::{Aim, Model};

/// Which list the arrows belong to. Two, because two of the four panes hold a
/// list; the chat pane scrolls and the composer takes text, and both are
/// reached the way every control is, with Tab.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum Pane {
    /// The roster, and the window opens on it: a seat with nothing aimed at has
    /// exactly one thing to do next.
    #[default]
    Roster,
    /// The aimed wall's conversations.
    Conversations,
}

/// **The id the composer's box wears**, and the whole of what the keyboard has
/// to know about it. The deposit's box and the start's are one control with two
/// subjects and are never painted together, so they wear one id — and the gate
/// below is a comparison rather than a guess about what "focused" means.
pub const BOX_ID: &str = "the composer's box";

/// Whether the composer's box holds the keyboard right now.
fn typing(ctx: &egui::Context) -> bool {
    ctx.memory(egui::Memory::focused) == Some(egui::Id::new(BOX_ID))
}

/// The mark a focused pane's heading wears — and the heading is where it goes
/// because a pane's heading is the one thing on it that is always painted, even
/// when it holds nothing at all.
pub const HERE: &str = "›";

/// The heading a pane paints, with the mark when the arrows are its.
pub fn heading(word: &str, focused: bool) -> String {
    if focused {
        format!("{HERE} {word}")
    } else {
        word.to_owned()
    }
}

/// **Take this frame's keys.** Called at the top of the frame, so what a key
/// changed is what the frame paints.
pub fn handle(ctx: &egui::Context, model: &mut Model) {
    if typing(ctx) {
        return;
    }
    let pressed = |key| ctx.input(|i| i.key_pressed(key));
    if pressed(egui::Key::Escape) {
        model.escape();
    }
    // **A modal owns the arrows** (bl-7574). While the enrollment covers the
    // window the lists behind it are not the subject of anything, and a walk
    // under it would re-aim the roster beneath the material — and would take
    // the arrows out of the name box the operator is typing into.
    if model.enroll.is_some() {
        return;
    }
    // Left and right name a **place**, not a step in a cycle: the roster is
    // left of the conversation list on the glass, so the key that points at it
    // is the key that goes there, and an operator never has to know where the
    // focus was to know where it will be.
    if pressed(egui::Key::ArrowLeft) {
        model.focus = Pane::Roster;
    }
    if pressed(egui::Key::ArrowRight) {
        model.focus = Pane::Conversations;
    }
    for (key, step) in [(egui::Key::ArrowUp, -1), (egui::Key::ArrowDown, 1)] {
        if pressed(key) {
            walk(model, step);
        }
    }
}

/// Move the focused list's selection by one, and say the pane owes the new
/// selection a place on the glass.
///
/// **The walk is the surface that can leave the glass behind.** A list longer
/// than its pane scrolls ([`crate::ui::shell`]), and a key that moved the
/// selection past the fold without moving the fold would put the two surfaces
/// back into the disagreement `crate::ui::roster::aimable` exists to prevent —
/// the cursor IS the selection, so the selection has to be somewhere an
/// operator can see it.
fn walk(model: &mut Model, step: isize) {
    match model.focus {
        Pane::Roster => {
            let rows: Vec<Aim> = crate::ui::roster::aimable(model);
            let at = rows.iter().position(|row| model.aim.as_ref() == Some(row));
            if let Some(row) = moved(rows.len(), at, step).and_then(|i| rows.get(i)) {
                model.aim_at(&row.channel.clone(), &row.address.clone());
                model.reveal = true;
            }
        }
        Pane::Conversations => {
            // `Model::rows`, not `convs`: the list the pane paints carries a
            // started conversation's own row before the engine can answer one,
            // and a row the pointer can aim at is a row a key can aim at.
            let rows: Vec<String> = model.rows().iter().map(|row| row.root_id.clone()).collect();
            let at = rows
                .iter()
                .position(|id| model.conversation.as_ref() == Some(id));
            if let Some(id) = moved(rows.len(), at, step).and_then(|i| rows.get(i)) {
                model.select(&id.clone());
                model.reveal = true;
            }
        }
    }
}

/// Where the cursor lands.
///
/// **Nothing selected goes to the first row, whichever way it was pressed**: a
/// list an operator has not entered has no direction to move in yet, and the
/// alternative is teaching them that Up means "the last one" before they have
/// selected anything. The ends **saturate** rather than wrap, because a wrap
/// makes the same keypress mean *next* thirty times and *back to the top* once,
/// with nothing on the glass to say which it will be.
fn moved(len: usize, at: Option<usize>, step: isize) -> Option<usize> {
    if len == 0 {
        return None;
    }
    let Some(at) = at else {
        return Some(0);
    };
    Some(at.saturating_add_signed(step).min(len - 1))
}

#[cfg(test)]
mod tests;
