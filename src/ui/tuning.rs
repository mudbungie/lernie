//! **The tuning pane**: what this wall's roles are set to, and the three
//! controls that set them (yog's `docs/PARITY.md` §7; PROTOCOL 6 and 7).
//!
//! # The settings surface this seat did not have
//!
//! `crate::snapshot::reach` was written against a ball asking for *the settings
//! panel*, and recorded that this seat had none — the window was a notice bar,
//! two lists, a conversation and a composer, with no preferences surface
//! anywhere. This is it, and it is deliberately not *preferences*: nothing here
//! is a fact about this box. Every row is a fact about the **wall**, held in
//! that workspace's config on the engine, and every control is one gesture
//! across the §8.5 boundary. A seat that kept a copy would be a second
//! authority for a file it does not own.
//!
//! # It opens on the read, so it opens showing what is in force
//!
//! The control that opens it fires `roles` — yog's own row for that read says
//! why: *"read back from the same place they write it, so a control can open
//! showing what is in force instead of blank."* The read is **standing** while
//! the pane is open (`crate::state::Standing`), so a write that lands is
//! reflected by the next answer rather than by this end predicting one. That
//! is what makes every control here a statement of the engine's fact: nothing
//! on this pane is ever this seat's own guess about a file it wrote to.
//!
//! Until the first answer the pane says so, in the [`crate::ui::convs`]
//! doctrine's own words: a wall nobody has been answered about is not a wall
//! with no roles. The two sentences are [`NOT_ANSWERED`] and [`NO_ROLES`], and
//! the second is a fact about the workspace — a fresh one really does declare
//! none.
//!
//! # Four controls per row, and the fourth is a draft
//!
//! Effort is four seats, one per level, and the fourth of those is the
//! **absence** rather than a fifth word: `off` removes the line, and the wire
//! spells that `null` (`crate::verbs::tuning`). Priority is one seat that
//! toggles, because upstream says it is *"a checkbox, not a choice of lanes"*.
//! The assignment is neither — it is two words the operator types — so it
//! opens an editor **under its own row**, seeded from what is in force, and
//! that editor is the one piece of state this pane holds
//! (`crate::ui::model::tuning`).

use crate::reply::roles::RoleRow;
use crate::ui::{Model, theme};

/// The word that opens the pane, on the wall the window is aimed at.
pub const OPEN: &str = "tune the roles…";
/// The word that closes it.
pub const CLOSE: &str = "done";
/// The pane's own heading.
pub const HEADING: &str = "role tuning";
/// What it says for a wall nobody has been answered about yet.
pub const NOT_ANSWERED: &str = "waiting to hear what this wall's roles are";
/// What it says for a wall that answered, and declares no role. A fact about
/// the workspace, and the one empty state here that is not a wait.
pub const NO_ROLES: &str = "this workspace's config declares no role";
/// The word on the seat that asks for the priority lane.
pub const PRIORITY: &str = "priority";
/// The word that opens the assignment editor on a row.
pub const ASSIGN: &str = "model…";
/// The word that spends a draft assignment.
pub const SET: &str = "set";
/// The word that puts one down without spending it.
pub const CANCEL: &str = "cancel";
/// What the two boxes ask for.
pub const PROVIDER_HINT: &str = "provider row";
/// The second of them.
pub const MODEL_HINT: &str = "model id";

/// Paint the pane and take the clicks on it. Answers whether there was one to
/// paint, so the shell knows whether the conversation still stands.
pub fn render(ui: &mut egui::Ui, model: &mut Model) -> bool {
    if model.tuning.is_none() {
        return false;
    }
    ui.heading(HEADING);
    if let Some(aim) = model.aim.clone() {
        ui.label(format!("on {} — {}", aim.address, aim.channel));
    }
    ui.horizontal_wrapped(|ui| {
        if ui.button(CLOSE).clicked() {
            model.close_tuning();
        }
    });
    ui.separator();
    match model.roles.clone() {
        None => {
            ui.label(NOT_ANSWERED);
        }
        Some(rows) if rows.is_empty() => {
            ui.label(NO_ROLES);
        }
        // The list scrolls, and the heading and the wall above it do not — the
        // same shape the two list panes keep, and for the same reason: a wall
        // with a dozen roles is longer than a pane at any width this seat opens
        // at, and a pane cut off mid-row says nothing about having been cut.
        Some(rows) => {
            egui::ScrollArea::vertical()
                .id_salt(HEADING)
                .auto_shrink(false)
                .show(ui, |ui| {
                    for row in &rows {
                        role(ui, model, row);
                    }
                });
        }
    }
    true
}

/// One role: what it runs on, how it is tuned, and the controls that retune it.
fn role(ui: &mut egui::Ui, model: &mut Model, row: &RoleRow) {
    ui.separator();
    ui.label(row.role.clone());
    ui.colored_label(
        theme::tone_ink(&crate::reply::convs::Tone::Weak),
        row.runs_on(),
    );
    // **Every row here WRAPS**, because this pane stands in the central panel
    // and the central panel is what the two side panels leave (bl-dc07): at a
    // 400-point window that is about 120 points, and an unwrapped row lays its
    // seats on one line however long the line has to be. Four levels and a
    // toggle do not fit one.
    ui.horizontal_wrapped(|ui| {
        for level in crate::verbs::tuning::levels() {
            let chosen = row.effort == level;
            let seat = ui.selectable_label(chosen, crate::verbs::tuning::word(level.as_ref()));
            crate::ui::act::tag(&seat, &[crate::verbs::EFFORT]);
            if seat.clicked() {
                model.post_effort(&row.role, level.clone());
            }
        }
        // **A real checkbox and not a fifth seat beside the levels**, because
        // upstream says what it is: *"a checkbox, not a choice of lanes …
        // asking for the standard lane outright is a different intent that no
        // setting expresses."* Two seats reading `on` and `off` would express
        // exactly that intent. The bool is a **local mirroring the engine's
        // row**, re-seeded every frame and never read back for anything but
        // the value to send — so nothing here holds an opinion about a lane
        // between one answer and the next.
        let mut on = row.priority;
        let lane = ui.checkbox(&mut on, PRIORITY);
        crate::ui::act::tag(&lane, &[crate::verbs::PRIORITY]);
        if lane.clicked() {
            model.post_priority(&row.role, on);
        }
    });
    // **A level the four seats do not name is painted as itself** (`crate::reply`
    // rung 3). The gesture asserts one of a closed set; this reports what the
    // config file holds, and a file written by a hand can hold a word this seat
    // has no seat for. Painting it as a neighbour would be a lie; painting
    // nothing would leave four unselected seats saying the level is `off`.
    if let Some(said) = unseated(row) {
        ui.colored_label(theme::NOTICE, said);
    }
    assignment(ui, model, row);
}

/// The sentence for an effort level no seat on this pane names, or none.
pub fn unseated(row: &RoleRow) -> Option<String> {
    let known = crate::verbs::tuning::levels().contains(&row.effort);
    let level = row.effort.clone()?;
    (!known).then(|| format!("asking for {level:?}, which this seat has no seat for"))
}

/// The assignment: the control that opens the editor, or the editor itself.
fn assignment(ui: &mut egui::Ui, model: &mut Model, row: &RoleRow) {
    if model.editing(&row.role).is_none() {
        if ui.button(ASSIGN).clicked() {
            model.edit_assignment(row);
        }
        return;
    }
    let mut ready = false;
    if let Some(edit) = model.draft_assignment() {
        ready = edit.ready();
        ui.horizontal_wrapped(|ui| {
            ui.add(egui::TextEdit::singleline(&mut edit.provider).hint_text(PROVIDER_HINT));
            ui.add(egui::TextEdit::singleline(&mut edit.model).hint_text(MODEL_HINT));
        });
    }
    ui.horizontal_wrapped(|ui| {
        let set = ui.add_enabled(ready, egui::Button::new(SET));
        crate::ui::act::tag(&set, &[crate::verbs::MODEL.word]);
        if set.clicked() {
            model.post_assignment();
        }
        if ui.button(CANCEL).clicked() {
            model.cancel_assignment();
        }
    });
}

#[cfg(test)]
mod tests;
