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
//! # The narrow shape does not add a binding; it changes what a place IS
//!
//! With one column on the glass at a time (`crate::ui::shell::policy`), left
//! and right name a **column** rather than one of two panes, and the arrows
//! belong to whichever column is showing — asked of [`Pane`] as always, set
//! from the column once per frame. Nothing else here knows: the walk, the
//! reveal and the mark on a heading read the one field they always read, and no
//! pane has to ask which shape it is in.
//!
//! **A box that is taking text takes every key**, which is the one gate: while
//! a text box holds the focus nothing here runs, so an arrow is a cursor move
//! inside the draft and Escape is egui's own *leave the box*. Press it again
//! with no box focused and it is [`Model::escape`] below — which closes the
//! enrollment where one covers the window, and puts the notice down otherwise.
//! One key, three contexts, and the contexts never overlap.
//!
//! The gate asks for **those boxes by name** rather than for egui's
//! `wants_keyboard_input`, which answers *is anything focused at all* — every
//! button included. Tabbing to Send would otherwise turn the arrows off, and a
//! click focuses a control too, so the honest question is the narrow one: every
//! box that takes text wears an id, [`BOXES`] is the whole list of them, and
//! the gate compares against it.

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

/// **The id the flag's reason box wears** (`crate::ui::composer::acts::WHY`).
pub const REASON_ID: &str = "the flag's reason box";

/// **The id the deletion's arming box wears**
/// (`crate::ui::composer::acts::ARM`).
pub const ARM_ID: &str = "the deletion's arming box";

/// **The id the config editor's box wears** (`crate::ui::config::edit`).
pub const CONFIG_ID: &str = "the config editor's box";

/// **The id the workflow name box wears** (`crate::ui::config`).
pub const WORKFLOW_ID: &str = "the workflow name box";

/// **Every box on the glass that takes text, so the gate can name them all**
/// (bl-dbc9).
///
/// It was one id, and one was enough while the only way into the other two was
/// Tab — a hazard, but one an operator walked into deliberately. A conversation
/// row's menu now LANDS the cursor in the reason box and in the arming box
/// (`crate::ui::model::fill`), and an arrow taken from inside either would have
/// walked the conversation list under a half-typed reason and flagged the row
/// it landed on. The gate is still a comparison rather than
/// `wants_keyboard_input` — which answers *is anything focused*, buttons
/// included — and this is the whole list of what it compares against. A fourth
/// box belongs here in the commit that paints it.
pub const BOXES: [&str; 5] = [BOX_ID, REASON_ID, ARM_ID, CONFIG_ID, WORKFLOW_ID];

/// Whether a box that takes text holds the keyboard right now.
fn typing(ctx: &egui::Context) -> bool {
    let focused = ctx.memory(egui::Memory::focused);
    BOXES
        .iter()
        .any(|name| focused == Some(egui::Id::new(*name)))
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
    // **A covering pane owns the arrows** (bl-7574; the tuning pane joined it
    // in bl-4a2c). While one covers the window the lists behind it are not the
    // subject of anything, and a walk under it would re-aim the roster beneath
    // the material — and would take the arrows out of the box the operator is
    // typing into. That second half is what makes this the gate for every pane
    // rather than only for the one holding a secret: the tuning pane's
    // assignment editor is a text box with no [`BOX_ID`] on it, and an arrow
    // reaching the roster from inside it would re-aim, which retires the pane
    // and the draft in it.
    //
    // **It asks `Model::covered`**, which is the one question the shell, the
    // roster's per-wall controls and this gate already share (bl-f0ef) — a
    // fifth pane listed here and not there is a pane the arrows walk under.
    if model.covered() {
        return;
    }
    // Left and right name a **place**, not a step in a cycle: the roster is
    // left of the conversation list on the glass, so the key that points at it
    // is the key that goes there, and an operator never has to know where the
    // focus was to know where it will be.
    //
    // **In the narrow shape the place is a COLUMN** (bl-dfda), because that is
    // what left and right mean when one column is on the glass at a time — and
    // there are three of them, so the key steps rather than names, saturating
    // at the ends the way the walk below does.
    let narrow = matches!(
        crate::ui::shell::shape(ctx.screen_rect().width()),
        crate::ui::Shape::Narrow
    );
    for (key, step) in [(egui::Key::ArrowLeft, -1), (egui::Key::ArrowRight, 1)] {
        if pressed(key) {
            sideways(model, narrow, step);
        }
    }
    // **In the narrow shape the arrows belong to the column on the glass**,
    // because it is the only list there is: a focus set at another width would
    // otherwise walk a selection nobody can see. It is spent here, once, so
    // everything below — the walk, the reveal, the mark on a heading — reads
    // one field and no pane has to ask which shape it is in.
    if narrow {
        model.focus = model.column.arrows();
    }
    for (key, step) in [(egui::Key::ArrowUp, -1), (egui::Key::ArrowDown, 1)] {
        if pressed(key) {
            walk(model, step);
        }
    }
}

/// **Sideways**: the place a left or right key names, in the shape the window
/// is in. Two panes hold the arrows in the broad shape and there is one of each
/// key, so the key IS the place; the narrow shape has three columns and one on
/// the glass, so the key is a step off wherever the operator is.
fn sideways(model: &mut Model, narrow: bool, step: isize) {
    if narrow {
        model.column = model.column.stepped(step);
    } else if step < 0 {
        model.focus = Pane::Roster;
    } else {
        model.focus = Pane::Conversations;
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
