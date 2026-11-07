//! **One step of the working** (DESIGN §4.48 item 2, amending §4.45 items 5
//! and 7; `docs/STYLE.md` §5, *a block*): its one line, and — whole — what
//! it said beneath, a call's output on the command card.
//!
//! Split from [`super`] at the cap on the seam that module's doc draws: the
//! transcript is what was said and this is what was DONE between the saying.
//! Which stage a step is in — whole, compact, or hidden under the working's
//! one line — is [`super::working`]'s decision, and so is the sub-bubble the
//! step stands in; this paints the step it is handed.
//!
//! **A finished step's line is the same door at both heights**: compact, it
//! is the whole step, one line tall, and opens it; whole, it heads the step
//! and shuts it. A call's line carries its **exit badge** at the right end —
//! the code the result stated (`Row::exit`), in the attention accent for
//! zero and the error accent otherwise — and its output stands beneath on
//! the ground in the monospace family: the baseline's command card. A step
//! in flight is not a door; its line is its mark and its name.

use super::{Fold, Kind, Row, code, fold};
use crate::ui::theme::{self, Bubble, Speaker, State, glyph, space};

/// What a finished step's line does, compact and whole.
pub const STEP_HELP: &str = "one step of the working; opens it whole in place";
pub const SHUT_HELP: &str = "folds this step back to its one line";

/// **A step's line, as a door**: its mark — how a call ended, or who spoke —
/// its name, a compact thought's or prose's first line after it, and a
/// call's exit badge at the right end. `open` says which way it goes.
pub(super) fn line(row: &Row, open: bool) -> theme::paint::Row {
    let red = theme::accent(State::Error);
    let weak = theme::INK_WEAK;
    let mark = if is_call(row) {
        if row.failed {
            (glyph::FAILED, red)
        } else {
            (glyph::DONE, weak)
        }
    } else if row.weight == Speaker::Model && row.kind == Kind::Said {
        (glyph::MODEL, weak)
    } else {
        (glyph::NOTE, weak)
    };
    let first = row.said.lines().next().unwrap_or_default();
    let words = if open || is_call(row) {
        row.who.clone()
    } else {
        format!("{} {} {first}", row.who, glyph::NOTE)
    };
    theme::paint::Row {
        mark: (mark.0.to_owned(), mark.1),
        words,
        ink: if row.failed { red } else { weak },
        state: row.failed.then_some(red),
        chosen: false,
        indent: 0.0,
        aside: row.exit.map(badge),
        beneath: Vec::new(),
        tail: None,
        door: None,
        help: if open { SHUT_HELP } else { STEP_HELP }.to_owned(),
    }
}

/// **The exit badge**: the code in words, and the state it says — zero
/// asks nothing of the operator, anything else failed.
fn badge(code: i32) -> (String, Option<State>) {
    let state = if code == 0 {
        State::Attention
    } else {
        State::Error
    };
    (format!("exit {code}"), Some(state))
}

/// A call, its result, or a result whose call is not on the read.
pub(super) fn is_call(row: &Row) -> bool {
    matches!(row.kind, Kind::Call | Kind::Answered | Kind::Result)
}

/// **One step, whole.** With `door`, its line is the seat that shuts it
/// again, and the answer is whether that seat was clicked; without, the line
/// is its mark and name and the answer is never.
pub(super) fn step(ui: &mut egui::Ui, at: usize, row: &Row, door: bool, bubbles: bool) -> bool {
    let shut = if door {
        line(row, true).paint(ui).clicked()
    } else {
        flying(ui, row);
        false
    };
    if row.said.is_empty() {
        return shut;
    }
    if matches!(row.kind, Kind::Answered | Kind::Result) {
        inset(ui, (space::XS, space::XS), |ui| {
            let role = bubbles.then_some(Bubble::Card);
            theme::paint::nested(ui, role, space::S, |ui| output(ui, at, row));
        });
    } else {
        inset(ui, (theme::RULE + space::M, space::S), |ui| {
            match row.kind {
                // Prose is what was said, with its fences; a step's words are
                // its machinery, in the weak ink.
                Kind::Said => code::paint(ui, at, &row.said, theme::INK),
                _ => {
                    theme::paint::prose(ui, &row.said, theme::INK_WEAK);
                }
            }
        });
    }
    shut
}

/// **The line of a step in flight**: its mark and its name, at a row's
/// height and a row's inset, so it stands where its door will.
fn flying(ui: &mut egui::Ui, row: &Row) {
    let (mark, ink) = match row.kind {
        Kind::Call => (glyph::CALL, theme::BRAND),
        Kind::Said if row.weight == Speaker::Model => (glyph::MODEL, theme::INK_WEAK),
        _ => (glyph::NOTE, theme::INK_WEAK),
    };
    let who = match row.kind {
        Kind::Said => theme::speaker(row.weight),
        _ => theme::INK_WEAK,
    };
    ui.horizontal(|ui| {
        ui.set_min_height(theme::ROW);
        ui.add_space(theme::RULE + space::M);
        ui.label(egui::RichText::new(mark).color(ink));
        ui.label(egui::RichText::new(&row.who).color(who));
    });
}

/// **What a step said, inset from its sub-bubble's edges** — `left` and
/// `right` — with `XS` under it.
fn inset(ui: &mut egui::Ui, (left, right): (f32, f32), body: impl FnOnce(&mut egui::Ui)) {
    ui.horizontal_top(|ui| {
        ui.add_space(left);
        ui.vertical(|ui| {
            ui.set_width(ui.available_width() - right);
            body(ui);
        });
    });
    ui.add_space(space::XS);
}

/// **A call's output, on the card**: the monospace family, folded to its head
/// where it is long (bl-90d0) under the one control that says what it hides.
fn output(ui: &mut egui::Ui, at: usize, row: &Row) {
    match &row.fold {
        Some(fold) => folded(ui, at, row, fold),
        None => mono(ui, &row.said, theme::INK_WEAK),
    }
}

/// Words in the monospace family, wrapped at the width they have.
fn mono(ui: &mut egui::Ui, words: &str, ink: egui::Color32) {
    ui.add(egui::Label::new(egui::RichText::new(words).monospace().color(ink)).wrap());
}

/// **A machine's answer, folded to its head** (bl-90d0): the one control
/// says what it hides and how much. It is egui's own button, which wears the
/// toolkit's hover (§4.45 item 6).
fn folded(ui: &mut egui::Ui, at: usize, row: &Row, fold: &Fold) {
    let id = ui.make_persistent_id(at);
    let mut open = ui.data(|d| d.get_temp::<bool>(id)).unwrap_or(false);
    let help = if open {
        fold::FOLD_HELP
    } else {
        fold::UNFOLD_HELP
    };
    if theme::paint::button(
        ui,
        if open { fold::UNFOLD } else { &fold.whole },
        true,
        help,
    )
    .clicked()
    {
        open = !open;
        ui.data_mut(|d| d.insert_temp(id, open));
    }
    if open {
        mono(ui, &row.said, theme::INK);
    } else {
        mono(ui, &fold.head, theme::INK_WEAK);
    }
}
