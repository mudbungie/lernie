//! **Find text**, across the balls, workspaces and conversations every engine
//! this box holds can see (bl-40ec; DESIGN §4.21).
//!
//! # The seventh covering pane, and the third about no focus
//!
//! `search` names no workspace, so its subject is every channel and the pane
//! is the union — `crate::ui::queue`'s shape, and `crate::ui::commands`' — and
//! its control hangs off the roster for the same reason.
//!
//! # The needle is a required parameter, so the act is DISABLED and not absent
//!
//! DESIGN §4.20's enablement rule, which is the tuning pane's `set` rather
//! than the composer's second start: what is missing is the parameter, not the
//! subject, so the control stays on the glass saying what would fill it. And
//! the box is **not spent on firing** — refining a needle is the common act,
//! and clearing it would charge a retype for every second search.
//!
//! # A hit is READ and not actionable, and the pane says why
//!
//! yog bl-ef16: a search row addresses its workspace and its project by the
//! ENGINE'S OWN ABSOLUTE PATH while every gesture this box composes carries a
//! name, so the keys a row spells are the keys the acts take and the values
//! are not — feed one back and it earns `unknown workspace`. So there is no
//! *go to it* here, and the reason is on the glass rather than in a comment:
//! a control that guessed a name off a path would be the mis-aim
//! `crate::ui::model::window` refuses one layer down, and a control that
//! silently did nothing would be worse.
//!
//! That is `crate::ui::queue`'s decision about a parked invocation, one noun
//! over: saying what was found is worth more than saying nothing while the
//! address it names cannot be spent.

use crate::ui::{Model, theme};

/// The word that opens the pane. It hangs off the roster, above the channels.
pub const OPEN: &str = "find text…";
/// The word that closes it.
pub const CLOSE: &str = "done";
/// The pane's own heading.
pub const HEADING: &str = "find text";
/// The word on the control that spends the needle.
pub const FIND: &str = "find";
/// What stands beside the control while there is nothing to search for.
pub const NEEDS_WORDS: &str = "type what to look for and this becomes live";
/// What the pane says before anything has been asked.
pub const NOT_ASKED: &str = "nothing has been looked for yet";
/// What a section says for an engine that answered and found nothing.
pub const NOTHING: &str = "this engine found none of it";
/// **Why a hit cannot be acted on** — the standing sentence, said once above
/// the list rather than on every row (yog bl-ef16).
pub const NOT_ADDRESSABLE: &str = "a hit says where it is in the engine's own path spelling, which no gesture \
     from this box can carry — so these are read here and aimed at by hand";

/// Paint the pane and take the clicks on it. Answers whether there was one to
/// paint, so the shell knows whether the conversation still stands.
pub fn render(ui: &mut egui::Ui, model: &mut Model) -> bool {
    if !model.finding() {
        return false;
    }
    ui.heading(HEADING);
    ui.horizontal_wrapped(|ui| {
        if ui.button(CLOSE).clicked() {
            model.close_lookup();
        }
    });
    ui.separator();
    asking(ui, model);
    ui.separator();
    let found = model.found.clone();
    if found.is_empty() {
        ui.label(NOT_ASKED);
        return true;
    }
    ui.colored_label(theme::NOTICE, NOT_ADDRESSABLE);
    egui::ScrollArea::vertical()
        .id_salt(HEADING)
        .auto_shrink(false)
        .show(ui, |ui| {
            for section in &found {
                ui.separator();
                ui.label(crate::ui::roster::header(&section.channel));
                answered(ui, &section.found);
            }
        });
    true
}

/// The box and the act: what to look for, and the control that asks.
fn asking(ui: &mut egui::Ui, model: &mut Model) {
    ui.text_edit_singleline(&mut model.needle);
    let live = model.needled();
    let mut fired = false;
    ui.horizontal_wrapped(|ui| {
        let act = ui.add_enabled(live, egui::Button::new(FIND));
        crate::ui::act::tag(&act, &[crate::verbs::SEARCH.word]);
        fired = act.clicked();
    });
    // **A greyed control says a thing is not live and nothing about what would
    // make it live** (DESIGN §4.20), so the sentence stands beside it.
    if !live {
        ui.colored_label(theme::NOTICE, NEEDS_WORDS);
    }
    if fired {
        model.post_search();
    }
}

/// One channel's answer: what it read the needle as, what it could not read,
/// and the hits.
fn answered(ui: &mut egui::Ui, found: &crate::reply::search::Found) {
    // **The engine's own echo of the needle**, never the box's current
    // contents: these rows are an answer to what was asked, and an operator
    // mid-edit must not be told the old rows are about the new words.
    ui.label(looked_for(found));
    for why in &found.unreadable {
        ui.colored_label(theme::NOTICE, unread(why));
    }
    if found.rows.is_empty() {
        ui.label(NOTHING);
        return;
    }
    for row in &found.rows {
        hit(ui, row);
    }
}

/// One hit: what it is in, where in it, and the words around the needle.
fn hit(ui: &mut egui::Ui, row: &crate::reply::search::Hit) {
    ui.label(row.subject());
    ui.colored_label(
        theme::tone_ink(&crate::reply::convs::Tone::Weak),
        row.at_field(),
    );
    ui.add(egui::Label::new(row.excerpt.clone()).wrap());
}

/// **What this section is an answer to**, in the engine's own words.
pub fn looked_for(found: &crate::reply::search::Found) -> String {
    format!("looked for {:?}", found.needle)
}

/// **What the engine could not read**, which is a different claim from finding
/// nothing there — so it is said as its own sentence rather than folded into
/// an empty result.
pub fn unread(why: &str) -> String {
    format!("could not be read: {why}")
}

#[cfg(test)]
mod tests;
