//! **Help**: what this window takes — its slash words and its keys — and then
//! what these engines answer, every op each one has a word for and what each
//! is for (bl-40ec; DESIGN §4.21; the window's half, bl-55ed).
//!
//! # The window's own words come first
//!
//! The operator who types `/help` asked THIS window. The engines' tables are
//! spelled in their own slash grammar (`/scan`, `/workflow <lineage>`), most
//! of which this box does not take, so a pane that led with them taught words
//! the menu answers *nothing here answers to that*. The window's half is read
//! off the menu's own table, so the two never disagree about a word.
//!
//! # The sixth covering pane, and the second about no focus
//!
//! `crate::ui::queue` is the first: `attention` names no workspace, so its
//! subject is every channel and the pane is the union. `help` names none
//! either, so this is that shape one noun over — its control hangs off the
//! **roster** for the same reason, and it is offered on a seat that has aimed
//! at nothing.
//!
//! # It is sectioned per channel and deliberately not merged
//!
//! Two engines on one box may be at two protocol versions, so their tables are
//! two answers and a union would say they are one thing. The section header is
//! the roster's own (`crate::ui::roster::header`), which carries the address
//! the entry dials — so two entries terminating at one listener are visible as
//! such here exactly as they are on the roster (bl-77df).
//!
//! # This is the table this seat is judged by
//!
//! `crate::snapshot::parity::roster` reads the `surface` field off the
//! vendored fixture of this same shape, because it is the one home for *which
//! ops owe every seat a discoverable interactable* (yog's `docs/PARITY.md`
//! §2). So the pane an operator reads and the ledger that reddens for a
//! missing control come off one answer, and the classification is painted
//! rather than hidden: an op marked `machine` is one nothing here owes a
//! control, and saying so is the difference between a short pane and an
//! incomplete one.
//!
//! # And it is not `lernie help`
//!
//! That answers what this BINARY takes, from a table compiled into it, with
//! nothing provisioned (`crate::verbs::help`). This answers what the ENGINE
//! offers. Two subjects; the wire op is the second, and `crate::verbs::window`
//! states why only one of them has an argv row.

use crate::ui::{Model, theme};

/// **The word typed after the slash** that opens it (`crate::ui::slash`).
pub const WORD: &str = "help";
/// The pane's own heading — what the operator asked for, which is what the
/// window takes (bl-55ed).
pub const HEADING: &str = "what this window takes";
/// The window's own words, said above the rows they head.
pub const WORDS: &str = "the words, typed after / in the box — the menu's own rows";
/// The window's keys, said above the table.
pub const KEYS_SAID: &str = "the keys";
/// **Every key this window binds, and what it does** — DESIGN §4.42's
/// *keyboard, whole*, as the operator reads it.
pub const KEYS: [(&str, &str); 10] = [
    ("Enter", "sends the box, or fires the chosen word"),
    ("Shift+Enter", "breaks a line"),
    ("/", "opens the words"),
    ("Tab", "completes the chosen word into the box"),
    ("Up, Down", "walk whatever is open"),
    (
        "Esc",
        "closes a dropup, then the overlay, then the menu, then the notice",
    ),
    (
        "Ctrl+X",
        "every engine, its workspaces and their conversations",
    ),
    ("Ctrl+O", "changes the model this workspace runs on"),
    ("Ctrl+T", "unfolds what is running"),
    ("1 to 6", "answers the question it is asking, by its number"),
];
/// **The engines' section heading**: their op tables are a second subject —
/// what each engine answers, in the engine's own spelling — and saying so is
/// what keeps an operator from typing `/scan` into a box that does not take it.
pub const ENGINES: &str = "what each engine answers to — its own commands, for the lernie command line; the words above are what this window takes";
/// What it says before any channel has answered.
pub const NOT_ANSWERED: &str = "waiting to hear what these engines answer to";
/// What a section says for an engine that answered and named no op at all — a
/// fact about that engine, and the one empty state here that is not a wait.
pub const NO_OPS: &str = "this engine lists no commands";

/// Paint the pane and take the clicks on it. Answers whether there was one to
/// paint, so the shell knows whether the conversation still stands.
pub fn render(ui: &mut egui::Ui, model: &mut Model) -> bool {
    if !model.commanding() {
        return false;
    }
    crate::ui::overlay::name(ui, HEADING);
    window(ui, model);
    ui.separator();
    ui.label(egui::RichText::new(ENGINES).color(theme::INK_WEAK));
    let pages = model.pages.clone();
    if pages.is_empty() {
        ui.label(NOT_ANSWERED);
        return true;
    }
    for section in &pages {
        ui.separator();
        ui.label(crate::ui::roster::header(&section.channel));
        if section.rows.is_empty() {
            ui.label(NO_OPS);
            continue;
        }
        for row in &section.rows {
            op(ui, row);
        }
    }
    true
}

/// **What this window takes** (bl-55ed): the menu's rows — the same words in
/// the same order, read off the one table (`crate::ui::slash::rows`) — and
/// the keys. The operator who types `/help` asked the window, so the window
/// answers first.
fn window(ui: &mut egui::Ui, model: &Model) {
    ui.label(egui::RichText::new(WORDS).color(theme::INK_WEAK));
    for row in crate::ui::slash::rows(model) {
        ui.add(egui::Label::new(row.line()).wrap());
    }
    ui.add_space(theme::space::XS);
    ui.label(egui::RichText::new(KEYS_SAID).color(theme::INK_WEAK));
    for (key, does) in KEYS {
        ui.add(egui::Label::new(format!("{key}  {does}")).wrap());
    }
}

/// One op: what to type and who it is for, then the sentence, then the page.
///
/// Every line comes off a pure function of the row (`crate::reply::help`), so
/// the suite reads the sentence rather than the layout — `crate::ui::records`'
/// own rule.
fn op(ui: &mut egui::Ui, row: &crate::reply::help::HelpRow) {
    ui.label(row.headline());
    ui.colored_label(
        theme::tone_ink(&crate::reply::convs::Tone::Weak),
        row.summary.clone(),
    );
    // **The page WRAPS.** A horizontal run lays a paragraph on one line however
    // long it is and the panel cuts what reaches the frame, with no ellipsis —
    // the defect bl-3d0f closed on the notice bar, and a detail paragraph is
    // the longest prose this window paints.
    ui.add(egui::Label::new(row.detail.clone()).wrap());
}

#[cfg(test)]
mod tests;
