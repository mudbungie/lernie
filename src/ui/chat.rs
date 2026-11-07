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
//! (§4.48 item 2; `theme::bubble`): the operator's on the brand's tint
//! inside a hairline, the working on the ground inside a hairline with each
//! step a sub-bubble on the surface inside it, the turn's final prose raised
//! with the turn strip's `copy` door under it — the gutter glyph
//! inside it, so who spoke is still the glyph, and weight is still the
//! speaker's (`theme::speaker`) on the words that name them. Which bubble a
//! row gets is the projection's call ([`rows`]), never this painter's.
//!
//! At the head of a transcript stands the conversation's panel ([`panel`]);
//! where nothing is selected the column is empty but for the seat's mark,
//! large and centred, and the one line naming the ways in (DESIGN §4.48
//! item 1; [`empty`]). At its end stands each message
//! this seat sent that the committed transcript does not yet carry, its words
//! in the pending ink until the server's receipt lands ([`rows::sent`];
//! DESIGN §4.45 item 4).
//!
//! It is a **pure projection** of what the model holds: nothing here fires,
//! and the one state it keeps — which folds are open — is the toolkit's
//! memory, a view state no wire gesture corresponds to.

/// Fenced code in what was said, painted as code.
pub mod code;
pub mod fold;
/// The panel at the head of a transcript: the conversation's facts.
pub mod panel;
/// A procedure run as a marker on the entry it fired at (§4.47).
pub mod procedures;
pub mod rows;
/// A scroll the keyboard asked for, spent inside the area (§4.46 item 3).
pub mod scroll;
/// One step of the working, whole.
mod trail;
/// The working in three stages: which each step and the turn are in.
pub mod working;

pub use fold::Fold;
pub use rows::sent::whole;
pub use rows::{Kind, Row, rows};

use crate::settings::Toggle;
use crate::ui::theme::{self, Bubble, glyph};

/// **The one line under the mark in the empty column** (DESIGN §4.48
/// item 1): the ways in, and nothing else.
pub const WAYS_IN: &str = "/help for commands · Ctrl+X for a conversation · Ctrl+N to begin";
pub const LIVE: &str = "«live»";
/// What parts an open working from the prose that follows it.
pub const RESPONSE: &str = "response";

/// **The empty column** — nothing selected, or a workspace with nothing
/// open (DESIGN §4.48 item 1): the seat's mark, a quarter of the column's
/// narrower side and never under two rows, centred, with [`WAYS_IN`] under
/// it in weak ink. The seat's facts are the status bar's right end and the
/// switcher's head (`crate::ui::status::seat`).
fn empty(ui: &mut egui::Ui) {
    let room = ui.available_size();
    let side = (room.x.min(room.y) / 4.0).max(2.0 * theme::ROW);
    ui.vertical_centered(|ui| {
        ui.add_space(((room.y - side) / 2.0 - theme::ROW).max(0.0));
        theme::paint::emblem(ui, side);
        ui.add_space(theme::space::M);
        ui.add(egui::Label::new(egui::RichText::new(WAYS_IN).color(theme::INK_WEAK)).wrap());
    });
}

/// Paint the transcript, or the empty column where nothing is selected.
pub fn render(ui: &mut egui::Ui, model: &crate::ui::Model) {
    let Some(conversation) = model.conversation.as_ref() else {
        empty(ui);
        return;
    };
    let shown = egui::ScrollArea::vertical()
        .id_salt(conversation)
        // **Unstuck on the frame a key scrolls it** (`scroll::asked`), and
        // **never eased**: an eased scroll moves on a later frame, after
        // the area has judged itself stuck to its end again, so a key's
        // page would be undone by the stick it was meant to loosen — and a
        // key's answer is state the glass shows the frame it is pressed.
        .stick_to_bottom(!scroll::asked(ui.ctx()))
        .animated(false)
        .auto_shrink([false, false])
        // **Every id under the transcript is the conversation's** (bl-b83d):
        // the scroll area's salt names only its own scroll state, and the ui
        // it hands its content is a child of the pane's, the same whichever
        // conversation is painted — so a fold keyed on the row's index alone
        // (`working`, `trail`, `code`) was one memory for row N of EVERY
        // conversation. One push here, rather than a salt at each site,
        // is what keeps a site that forgets it impossible.
        .show(ui, |ui| {
            ui.push_id(conversation, |ui| {
                scroll::apply(ui);
                panel::conversation(ui, model, conversation);
                let all = whole(model);
                let mut at = 0;
                let mut turns = 0;
                // Whether the working just painted stands open, where one does.
                let mut opened = None;
                while let Some(row) = all.get(at) {
                    // A procedure run stands above the first row of the entry
                    // its notch seats (`procedures`, §4.47).
                    if at == 0 || all.get(at - 1).is_some_and(|prev| prev.entry != row.entry) {
                        procedures::paint(
                            ui,
                            &procedures::markers(model.records.rail.as_ref(), &row.entry),
                        );
                    }
                    let rest = all.get(at..).unwrap_or(&[]);
                    let run = rest
                        .iter()
                        .take_while(|row| {
                            matches!(row.bubble, Bubble::Trail | Bubble::Intermediate)
                        })
                        .count();
                    if run > 0 {
                        ui.add_space(theme::space::S);
                        let mut open = false;
                        let settings = &model.settings;
                        theme::paint::nested(
                            ui,
                            settings.on(Toggle::Bubbles).then_some(Bubble::Trail),
                            theme::space::XXS,
                            |ui| {
                                open = working::working(
                                    ui,
                                    at,
                                    rest.get(..run).unwrap_or(&[]),
                                    settings,
                                );
                            },
                        );
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
                    let bubble = model.settings.on(Toggle::Bubbles).then_some(row.bubble);
                    theme::paint::bubbled(ui, bubble, |ui| said(ui, at, row));
                    if row.bubble == Bubble::Final && row.weight == theme::Speaker::Model {
                        strip(ui, &row.said);
                    }
                    opened = None;
                    at += 1;
                }
            })
        });
    scroll::note(ui.ctx(), shown.state.offset.y);
}

/// **The turn strip** (DESIGN §4.48 item 2): under a settled turn's final
/// prose, at the right end, the `copy` door that puts its words on the
/// clipboard — the one act of the baseline's strip the seat performs
/// without the wire. The baseline's time is not painted: no entry on the
/// wire carries one, and a seat that guessed it would be inventing a fact.
fn strip(ui: &mut egui::Ui, prose: &str) {
    ui.add_space(theme::space::XS);
    let width = ui.available_width();
    ui.allocate_ui_with_layout(
        egui::vec2(width, 0.0),
        egui::Layout::right_to_left(egui::Align::Min),
        |ui| {
            ui.add_space(theme::space::M);
            if theme::paint::link(ui, COPY, theme::INK_WEAK, COPY_HELP).clicked() {
                ui.ctx().copy_text(prose.to_owned());
            }
        },
    );
}

/// The strip's door, and what it does.
pub const COPY: &str = "copy";
pub const COPY_HELP: &str = "puts this answer's words on the clipboard";

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
            // **A message the server has not acknowledged wears the pending
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
