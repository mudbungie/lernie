//! **The transcript**: what was said, read by its gutter (DESIGN §4.42, item
//! 4; `docs/STYLE.md` §5, *a transcript is read by its gutter*).
//!
//! An operator's turn is a row with the prompt's glyph in the gutter in the
//! brand and its words in body ink; the model's prose carries a bar in the
//! gutter in weak ink; a peer's, a tool's answer and the machinery's own
//! words carry a dot. The trail — the calls the model made, their results,
//! and its reasoning — is one group under a fold, `tool calls (3)`, each
//! step on a stem drawn as the roster draws its threading, a call with a
//! dot and its mark, a result folded to its head and unfolded in place. A
//! `response` line parts a trail from the prose that follows it, and a
//! hairline parts one turn from the next. No block wears a side rule and no
//! block is bubbled; who spoke is the gutter glyph, and weight is still the
//! speaker's (`theme::speaker`) on the words that name them.
//!
//! At the head of a transcript stands the conversation's panel, and where
//! nothing is selected the seat's ([`panel`]).
//!
//! It is a **pure projection** of what the model holds: nothing here fires,
//! and the one state it keeps — which folds are open — is the toolkit's
//! memory, a view state no wire gesture corresponds to.

pub mod fold;
/// The two panels at the head of the pane: the conversation's facts, and the
/// seat's where nothing is selected.
pub mod panel;
pub mod rows;

pub use fold::Fold;
pub use rows::{Kind, Row, rows};

use crate::ui::theme::{self, glyph};

/// The pane's word, kept for the seat panel's sentence.
pub const HEADING: &str = "conversation";

/// What the pane says with nothing selected, under the seat's panel.
pub const NO_CONVERSATION: &str = "Ctrl+X picks a conversation, or begin one in the box below";
pub const LIVE: &str = "«live»";
/// The fold's word over a trail, with the count of steps under it.
pub const TRAIL: &str = "tool calls";
/// What parts a trail from the prose that follows it.
pub const RESPONSE: &str = "response";

/// Paint the transcript.
pub fn render(ui: &mut egui::Ui, model: &crate::ui::Model) {
    let Some(conversation) = model.conversation.as_ref() else {
        panel::seat(ui, model);
        theme::paint::empty(ui, NO_CONVERSATION);
        return;
    };
    egui::ScrollArea::vertical()
        .id_salt(conversation)
        .stick_to_bottom(true)
        .auto_shrink([false, false])
        .show(ui, |ui| {
            panel::conversation(ui, model, conversation);
            let all = rows(&model.transcript, model.live.as_ref());
            let mut at = 0;
            let mut turns = 0;
            let mut after_trail = false;
            while let Some(row) = all.get(at) {
                let rest = all.get(at..).unwrap_or(&[]);
                let run = rest.iter().take_while(|row| row.kind != Kind::Said).count();
                if run > 0 {
                    ui.add_space(theme::space::S);
                    trail(ui, at, rest.get(..run).unwrap_or(&[]));
                    at += run;
                    after_trail = true;
                    continue;
                }
                if row.weight == theme::Speaker::Operator {
                    if turns > 0 {
                        parting(ui);
                    }
                    turns += 1;
                } else if after_trail {
                    response(ui);
                }
                ui.add_space(theme::space::S);
                said(ui, row);
                after_trail = false;
                at += 1;
            }
        });
}

/// **The hairline between turns.**
fn parting(ui: &mut egui::Ui) {
    ui.add_space(theme::space::M);
    let (rect, _) =
        ui.allocate_exact_size(egui::vec2(ui.available_width(), 1.0), egui::Sense::hover());
    // A fill rather than a line, so the probe that reads fills off the glass
    // can count it (`crate::paint_probe::fills_of`).
    ui.painter().rect_filled(rect, 0.0, theme::HAIRLINE);
}

/// **The line that parts a trail from the prose after it**: an elbow and
/// the word, in weak ink.
fn response(ui: &mut egui::Ui) {
    ui.add_space(theme::space::XS);
    let x = ui.cursor().min.x + theme::space::S;
    let top = ui.cursor().min.y;
    ui.horizontal(|ui| {
        ui.add_space(theme::space::L);
        ui.label(egui::RichText::new(RESPONSE).small().color(theme::INK_WEAK));
    });
    let middle = ui.min_rect().max.y - theme::space::S;
    theme::paint::elbow(ui, x, top, middle, x + theme::space::S);
}

/// **One thing said**: the gutter glyph, then who said it in their weight,
/// then the words as prose.
fn said(ui: &mut egui::Ui, row: &Row) {
    let (mark, ink) = match row.weight {
        theme::Speaker::Operator => (glyph::PROMPT, theme::BRAND),
        theme::Speaker::Model => (glyph::MODEL, theme::INK_WEAK),
        _ => (glyph::NOTE, theme::INK_WEAK),
    };
    ui.horizontal_top(|ui| {
        ui.label(egui::RichText::new(mark).color(ink));
        ui.vertical(|ui| {
            // A failure is a step of the trail, never something said, so
            // the weight here is always the speaker's.
            let who = theme::speaker(row.weight);
            ui.label(egui::RichText::new(&row.who).color(who).strong());
            theme::paint::prose(ui, &row.said, theme::INK);
        });
    });
}

/// **A trail**: the fold's head, then each step on its stem.
fn trail(ui: &mut egui::Ui, at: usize, steps: &[Row]) {
    let id = ui.make_persistent_id(("trail", at));
    let mut open = ui.data(|d| d.get_temp::<bool>(id)).unwrap_or(true);
    // **The fold counts calls** (bl-eb4e), not the steps under it: a result
    // stands on its call's row, and reasoning is not a call.
    let calls = steps
        .iter()
        .filter(|row| matches!(row.kind, Kind::Call | Kind::Answered))
        .count();
    let head = format!(
        "{} {TRAIL} ({calls})",
        if open { glyph::OPEN } else { glyph::SHUT },
    );
    if ui
        .add(
            egui::Label::new(egui::RichText::new(head).color(theme::INK_WEAK))
                .sense(egui::Sense::click()),
        )
        .clicked()
    {
        open = !open;
        ui.data_mut(|d| d.insert_temp(id, open));
    }
    if !open {
        return;
    }
    let x = ui.cursor().min.x + theme::space::S;
    for (i, row) in steps.iter().enumerate() {
        let top = ui.cursor().min.y;
        ui.horizontal_top(|ui| {
            ui.add_space(theme::space::L);
            step(ui, at + i, row);
        });
        let bottom = ui.min_rect().max.y;
        let middle = top + theme::space::S;
        theme::paint::elbow(ui, x, top, middle, x + theme::space::S);
        if i + 1 < steps.len() {
            theme::paint::rail(ui, x, middle, bottom);
        }
    }
}

/// **One step**: its mark, its name — a call's with how it ended beside it
/// — and what it said, folded to its head where there is more.
fn step(ui: &mut egui::Ui, at: usize, row: &Row) {
    let red = theme::accent(theme::State::Error);
    let ended = |ui: &mut egui::Ui| {
        let (mark, ink) = if row.failed {
            (glyph::FAILED, red)
        } else {
            (glyph::DONE, theme::INK_WEAK)
        };
        ui.label(egui::RichText::new(mark).color(ink));
    };
    let (mark, ink) = match row.kind {
        Kind::Call | Kind::Answered => (glyph::CALL, theme::BRAND),
        _ => (glyph::NOTE, theme::INK_WEAK),
    };
    if row.kind == Kind::Result {
        ended(ui);
    } else {
        ui.label(egui::RichText::new(mark).color(ink));
    }
    ui.vertical(|ui| {
        let who = if row.failed { red } else { theme::INK_WEAK };
        ui.horizontal_wrapped(|ui| {
            ui.label(egui::RichText::new(&row.who).color(who));
            if row.kind == Kind::Answered {
                ended(ui);
            }
        });
        match &row.fold {
            Some(fold) => folded(ui, at, row, fold),
            None if row.said.is_empty() => {}
            None => {
                theme::paint::prose(ui, &row.said, theme::INK_WEAK);
            }
        }
    });
}

fn folded(ui: &mut egui::Ui, at: usize, row: &Row, fold: &Fold) {
    let id = ui.make_persistent_id(at);
    let mut open = ui.data(|d| d.get_temp::<bool>(id)).unwrap_or(false);
    if ui
        .button(if open { fold::UNFOLD } else { &fold.whole })
        .clicked()
    {
        open = !open;
        ui.data_mut(|d| d.insert_temp(id, open));
    }
    if open {
        theme::paint::prose(ui, &row.said, theme::INK);
    } else {
        theme::paint::prose(ui, &fold.head, theme::INK_WEAK);
    }
}

#[cfg(test)]
mod tests;
