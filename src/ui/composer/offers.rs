//! **The acts the conversation offers on its turn** (bl-f251), as a compact
//! row of glyph-and-word controls under the field, and the one control that
//! opens the strip of acts it does not offer every day.
//!
//! # Only what the conversation offers is on the row
//!
//! The engine's own row for a conversation carries `offers` — *what may be
//! done to this conversation* (REMOTE §9.4's four gates, read as one set by
//! `crate::reply::agent`) — and this row reads it. A driver at rest is
//! offered `nudge`; a driver running is offered `stop`, and with it
//! `interrupt`, which is the stop that says something in the same gesture and
//! degrades to a plain send where nothing was running (REMOTE §8.2) — so it
//! stands beside `stop` and not beside `send`. Until the engine has answered
//! about the conversation at all, every act is offered: an absent reading is
//! not a refusal, and a control withheld on a guess would be this seat
//! predicting one the engine has not made.
//!
//! That reverses the records header's older ruling that the composer's
//! controls do not read the gates (`crate::ui::records::header`). The
//! reasoning there was that a control greyed on a snapshot predicts a refusal;
//! what stands here is not a prediction but the engine's own answer, read off
//! the same row the header paints, and a row that offered `nudge` on a
//! streaming conversation was a control whose every press was a refusal.
//!
//! # The strip is behind one control
//!
//! `records…` leads the row because it spends nothing — it only looks. After
//! it, [`MORE`] opens [`super::acts`]: the floor pair, the retarget, the flag
//! with its reason and the deletion with its arming. They are the acts on the
//! conversation as an object rather than on its turn, and a row of nine
//! controls under every conversation was the form the pane read as
//! (`docs/STYLE.md` §2). The strip opens on the control and on a row menu's
//! *flag…* or *delete…*, which lands the cursor in the box it named
//! (`crate::ui::model::fill`) — a box the operator was sent to cannot be
//! behind a fold they have not opened.

use crate::reply::agent::Offer;
use crate::ui::{Aim, Model, theme};

use super::{INTERRUPT, NUDGE, acts};

/// The word that kills the driver. `nudge`'s opposite, offered where the
/// engine says the driver is running.
pub const STOP: &str = "stop";
/// The control that opens the strip. The ellipsis is
/// `crate::ui::convs::menu::leads_to`'s convention with no word before it:
/// what it leads to is every act that is not on this row.
pub const MORE: &str = "…";
/// Whether the strip is open — the toolkit's memory rather than the model's,
/// for the reason a folded tool result's is (`crate::ui::chat`): a view state
/// no wire gesture corresponds to and no other fact can be asked for.
const OPEN_ID: &str = "the composer's strip";

/// Whether the engine offers `act` on the selected conversation — and every
/// act where it has not answered about it yet.
fn offered(model: &Model, act: Offer) -> bool {
    model
        .records
        .agent
        .as_ref()
        .is_none_or(|row| row.offers.contains(&act))
}

/// One glyph-and-word control, tagged with the op it fires.
fn control(ui: &mut egui::Ui, glyph: &str, word: &str, op: &str) -> bool {
    let seat = ui.button(theme::worded(glyph, word));
    crate::ui::act::tag(&seat, &[op]);
    seat.clicked()
}

/// Paint the row and take what it was given.
pub fn render(ui: &mut egui::Ui, model: &mut Model, aim: &Aim, agent: &str) {
    let running = offered(model, Offer::Stop);
    let quiet = offered(model, Offer::Nudge);
    let wanted = model.filling();
    let id = egui::Id::new(OPEN_ID);
    let mut open = ui.data(|d| d.get_temp::<bool>(id)).unwrap_or(false) || wanted.is_some();
    let (mut cut, mut nudged, mut halted) = (false, false, false);
    ui.horizontal_wrapped(|ui| {
        if running {
            cut = control(
                ui,
                theme::glyph::INTERRUPT,
                INTERRUPT,
                crate::verbs::INTERRUPT.word,
            );
        }
        if quiet {
            nudged = control(ui, theme::glyph::NUDGE, NUDGE, crate::verbs::NUDGE.word);
        }
        if running {
            halted = control(ui, theme::glyph::STOP, STOP, crate::verbs::STOP.word);
        }
        // **The reads this gesture reaches** (bl-2cf7, bl-b52c, bl-3257):
        // opening the records pane is what makes this seat read the selected
        // conversation's steps, its files, its spine, the config commit
        // governing it, its own row and its undelivered mail
        // (`crate::state::Standing`), and those six reads have no control of
        // their own.
        let records = ui.button(crate::ui::records::OPEN);
        crate::ui::act::tag(
            &records,
            &[
                crate::verbs::STEPS.word,
                crate::verbs::FILES.word,
                crate::verbs::RAIL.word,
                crate::verbs::GOVERNING.word,
                crate::verbs::AGENT.word,
                crate::verbs::INBOX.word,
            ],
        );
        if records.clicked() {
            model.begin_records();
        }
        if ui.button(MORE).clicked() {
            open = !open;
        }
    });
    ui.data_mut(|d| d.insert_temp(id, open));
    if cut {
        super::fire(model, crate::verbs::interrupt, &aim.address, agent);
    }
    if nudged {
        model
            .outbox
            .push(crate::ui::Posted::act(crate::verbs::nudge(
                aim.address.clone(),
                agent.to_owned(),
            )));
    }
    if halted {
        // **The bare form, from this control** (bl-9fd1, bl-3686). The
        // cascade is a second act with an arming of its own and it lives
        // beside the records that say what is under there
        // (`crate::ui::records::cascade`); this row is a routine surface
        // and §4.20 keeps a subtree off one.
        model.outbox.push(crate::ui::Posted::act(crate::verbs::stop(
            aim.address.clone(),
            agent.to_owned(),
            false,
        )));
    }
    if open {
        acts::render(ui, model, aim, agent, wanted);
    }
}

#[cfg(test)]
mod tests;
