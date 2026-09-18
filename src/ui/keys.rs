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
//! Nudge, Start, the `+` that begins a conversation and the notice's dismiss
//! are keyboard-operable with nothing written — `tests` proves it rather than
//! assuming it. What Tab cannot make *usable* is a list: tabbing through thirty
//! rows to reach the composer is reachability without operability, and that is
//! the whole of what this module adds.
//!
//! # The cursor IS the selection, so there is nothing to keep in step
//!
//! A list cursor beside a selection is two highlights, two things to paint and
//! two ways to disagree. There is no cursor: **moving in a list selects**, so
//! the highlight the pointer already paints is where the keyboard is, and the
//! reads that follow a selection follow a keypress for free (the standing set
//! is derived — `crate::state::Standing`). The one exception is an engine's own
//! row, which the walk may stand on without opening it ([`walk`]).
//!
//! # There is ONE list, so there is nothing to ask which list this is
//!
//! The walk used to be two tracks behind a `Pane` field on the model, because
//! the roster and the conversations were two panes. DESIGN §4.39 folded the
//! conversations under their wall, so there is one list, one track and one
//! order — engine rows, the open engine's walls, and the aimed wall's
//! conversations, exactly as the glass paints them
//! (`crate::ui::roster::track`). The field, the enum and the mark on a heading
//! that said which of the two was live all go with the second track: an answer
//! with no alternative is not an answer worth holding.
//!
//! # The narrow shape does not add a binding; it changes what a place IS
//!
//! With one column on the glass at a time (`crate::ui::shell::policy`), left
//! and right name a **column**. In the broad shape they name nothing, because
//! both columns are already on the glass and only one of them is a list — so
//! they gain no meaning there rather than acquiring a second one.
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

use crate::ui::{Model, roster};

/// Every box on the glass that takes text, and the gate that reads them.
mod boxes;

pub use boxes::{
    ARM_ID, BODY_ID, BOX_ID, BOXES, CONFIG_ID, DELIVER_ID, FAN_GOAL_ID, NOTE_ID, PROJECT_ID,
    REASON_ID, SUMMARY_ID, TITLE_ID, WORKFLOW_ID,
};

/// **Take this frame's keys.** Called at the top of the frame, so what a key
/// changed is what the frame paints.
pub fn handle(ctx: &egui::Context, model: &mut Model) {
    if boxes::typing(ctx) {
        return;
    }
    let pressed = |key| ctx.input(|i| i.key_pressed(key));
    if pressed(egui::Key::Escape) {
        model.escape();
    }
    // **A covering pane owns the arrows** (bl-7574; the tuning pane joined it
    // in bl-4a2c). While one covers the window the list behind it is not the
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
    // **Left and right name a COLUMN, and only where there are two to choose
    // between** (bl-dfda, DESIGN §4.39). In the narrow shape one column is on
    // the glass at a time, so the key steps between them, saturating at the
    // ends the way the walk below does. In the broad shape both are already
    // painted and one of them is the only list there is, so the key has no
    // place to name and names none.
    if matches!(
        crate::ui::shell::shape(ctx.screen_rect().width()),
        crate::ui::Shape::Narrow
    ) {
        for (key, step) in [(egui::Key::ArrowLeft, -1), (egui::Key::ArrowRight, 1)] {
            if pressed(key) {
                model.column = model.column.stepped(step);
            }
        }
    }
    for (key, step) in [(egui::Key::ArrowUp, -1), (egui::Key::ArrowDown, 1)] {
        if pressed(key) {
            walk(model, step);
        }
    }
    opening(
        ctx,
        model,
        pressed(egui::Key::Enter) || pressed(egui::Key::Space),
    );
}

/// **Enter or Space opens the engine the walk is standing on** (DESIGN §4.39)
/// — the one binding this module has that is not a walk, and it exists because
/// the cursor on an engine row is deliberately NOT the selection: the walk has
/// to be able to pass a closed engine without opening it, so opening is a
/// second keypress.
///
/// It names a control rather than adding one: [`Model::open_engine`] is the
/// door `crate::ui::roster::engine`'s own click calls.
///
/// **It fires only while nothing holds the keyboard**, which is the whole of
/// why it is a binding at all. An arrow key walks the list without focusing
/// anything, so egui has no widget to fire and this stands in for the click;
/// the moment a Tab has put the keyboard on a control — the engine's row, the
/// `+` beside it, anything — egui fires THAT control itself, and a binding
/// running beside it would spend a second act the operator did not ask for.
fn opening(ctx: &egui::Context, model: &mut Model, pressed: bool) {
    if !pressed || ctx.memory(egui::Memory::focused).is_some() {
        return;
    }
    if let Some(name) = model.standing.clone() {
        model.open_engine(&name);
    }
}

/// Move the one list's cursor by one, and say the pane owes the new selection a
/// place on the glass.
///
/// **The walk is the surface that can leave the glass behind.** A list longer
/// than its pane scrolls ([`crate::ui::shell`]), and a key that moved the
/// selection past the fold without moving the fold would put the two surfaces
/// back into the disagreement `crate::ui::roster::track` exists to prevent —
/// the cursor IS the selection, so the selection has to be somewhere an
/// operator can see it.
///
/// **The track crosses three kinds of row** (DESIGN §4.39): landing on a wall
/// aims it and landing on a conversation selects it, exactly as a click does,
/// and landing on an engine's row only STANDS there — the row takes the
/// keyboard, and Enter or Space fires the same act a pointer fires, because a
/// walk that opened every engine it moved through would be a walk nobody could
/// use to reach the one below.
fn walk(model: &mut Model, step: isize) {
    let rows = roster::track(model);
    let at = rows.iter().position(|row| row.holding(model));
    let Some(row) = moved(rows.len(), at, step).and_then(|i| rows.get(i).cloned()) else {
        return;
    };
    match row {
        roster::Step::Engine(name) => model.standing = Some(name),
        roster::Step::Wall(aim) => model.aim_at(&aim.channel, &aim.address),
        // `Model::rows`, not `convs`: the list the glass paints carries a
        // started conversation's own row before the engine can answer one, and
        // a row the pointer can aim at is a row a key can aim at.
        roster::Step::Conversation(root_id) => model.select(&root_id),
    }
    model.reveal = true;
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
