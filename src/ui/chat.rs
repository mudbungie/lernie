//! **The transcript**: what was said, read by its gutter (DESIGN §4.42, item
//! 4; `docs/STYLE.md` §5, *a transcript is read by its gutter*).
//!
//! An operator's turn is a row with the prompt's glyph in the gutter in the
//! brand and its words in body ink; the model's prose carries a bar in the
//! gutter in weak ink; a peer's, a tool's answer and the machinery's own
//! words carry a dot. The turn's working — every row between the operator's
//! turn and its final prose: the calls the model made, their results, its
//! reasoning and the prose between — is one block on the trail's bubble
//! that folds in three stages ([`working`], §4.45 item 5): whole while it
//! happens, compact once a step is done, one counted line once the answer
//! stands. A `response` line parts an open working from the prose that
//! follows it, and a hairline parts one turn from the next. No block wears a
//! side rule, and **every block stands on a bubble that says its role**
//! (§4.45 item 7; `theme::bubble`): the operator's on the brand's tint, the
//! working on the ground inside a hairline, intermediate prose on the
//! surface inside it, the turn's final prose raised — the gutter glyph
//! inside it, so who spoke is still the glyph, and weight is still the
//! speaker's (`theme::speaker`) on the words that name them. Which bubble a
//! row gets is the projection's call ([`rows`]), never this painter's.
//!
//! At the head of a transcript stands the conversation's panel, and where
//! nothing is selected the seat's ([`panel`]). At its end stands each message
//! this seat sent that the committed transcript does not yet carry, its words
//! in the pending ink until the engine's receipt lands ([`rows::sent`];
//! DESIGN §4.45 item 4).
//!
//! It is a **pure projection** of what the model holds: nothing here fires,
//! and the one state it keeps — which folds are open — is the toolkit's
//! memory, a view state no wire gesture corresponds to.

/// Fenced code in what was said, painted as code.
pub mod code;
pub mod fold;
/// The two panels at the head of the pane: the conversation's facts, and the
/// seat's where nothing is selected.
pub mod panel;
pub mod rows;
/// One step of the working, whole.
mod trail;
/// The working in three stages: which each step and the turn are in.
pub mod working;

pub use fold::Fold;
pub use rows::sent::whole;
pub use rows::{Kind, Row, rows};

use crate::ui::theme::{self, Bubble, glyph};

/// The pane's word, kept for the seat panel's sentence.
pub const HEADING: &str = "conversation";

/// What the pane says with nothing selected, under the seat's panel.
pub const NO_CONVERSATION: &str = "Ctrl+X picks a conversation, or begin one in the box below";
/// **And with nothing aimed**, where the box below begins nothing: a start
/// needs a wall (bl-2cb0), so the sentence names the key that picks one.
pub const NO_AIM: &str = "Ctrl+X picks a workspace to begin in, or a conversation";
pub const LIVE: &str = "«live»";
/// What parts an open working from the prose that follows it.
pub const RESPONSE: &str = "response";

/// Paint the transcript. Answers whether the seat panel's queue door was
/// clicked.
pub fn render(ui: &mut egui::Ui, model: &crate::ui::Model) -> bool {
    let Some(conversation) = model.conversation.as_ref() else {
        let asked = panel::seat(ui, model);
        let said = if model.aim.is_some() {
            NO_CONVERSATION
        } else {
            NO_AIM
        };
        theme::paint::empty(ui, said);
        return asked;
    };
    egui::ScrollArea::vertical()
        .id_salt(conversation)
        .stick_to_bottom(true)
        .auto_shrink([false, false])
        .show(ui, |ui| {
            panel::conversation(ui, model, conversation);
            let all = whole(model);
            let mut at = 0;
            let mut turns = 0;
            // Whether the working just painted stands open, where one does.
            let mut opened = None;
            while let Some(row) = all.get(at) {
                let rest = all.get(at..).unwrap_or(&[]);
                let run = rest
                    .iter()
                    .take_while(|row| matches!(row.bubble, Bubble::Trail | Bubble::Intermediate))
                    .count();
                if run > 0 {
                    ui.add_space(theme::space::S);
                    let mut open = false;
                    theme::paint::bubbled(ui, Bubble::Trail, |ui| {
                        open = working::working(ui, at, rest.get(..run).unwrap_or(&[]));
                    });
                    at += run;
                    opened = Some(open);
                    continue;
                }
                if row.weight == theme::Speaker::Operator {
                    if turns > 0 {
                        parting(ui);
                    }
                    turns += 1;
                } else if opened == Some(true) {
                    response(ui);
                }
                ui.add_space(theme::space::S);
                theme::paint::bubbled(ui, row.bubble, |ui| said(ui, at, row));
                opened = None;
                at += 1;
            }
        });
    false
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

/// **The line that parts an open working from the prose after it**: an
/// elbow and the word, in weak ink. Folded, the working's one line is the
/// parting (§4.45 item 5).
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
fn said(ui: &mut egui::Ui, at: usize, row: &Row) {
    let failed = theme::accent(theme::State::Error);
    let (mark, ink) = match row.weight {
        _ if row.failed => (glyph::FAILED, failed),
        theme::Speaker::Operator => (glyph::PROMPT, theme::BRAND),
        theme::Speaker::Model => (glyph::MODEL, theme::INK_WEAK),
        _ => (glyph::NOTE, theme::INK_WEAK),
    };
    ui.horizontal_top(|ui| {
        ui.label(egui::RichText::new(mark).color(ink));
        ui.vertical(|ui| {
            // A failed step is a step of the trail; the one failure that is
            // said is the wound — the conversation's own state, which ends
            // the transcript in the error accent rather than a speaker's
            // weight (`rows::WOUND`).
            let who = if row.failed {
                failed
            } else {
                theme::speaker(row.weight)
            };
            ui.label(egui::RichText::new(&row.who).color(who).strong());
            // **A message the engine has not acknowledged wears the pending
            // ink** (§4.45 item 4): the ink of what is not yet settled.
            let words = if row.pending {
                theme::INK_WEAK
            } else {
                theme::INK
            };
            code::paint(ui, at, &row.said, words);
        });
    });
}

#[cfg(test)]
mod tests;
