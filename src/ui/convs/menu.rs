//! **The conversation row's own acts**, hung off the row a secondary click
//! names (DESIGN §4.23; bl-dbc9).
//!
//! # One design, two platforms, because the toolkit synthesizes the trigger
//!
//! The operator's requirement was conversation management reachable from the
//! conversation itself — a right-click here, a long-press on the android
//! client. egui synthesizes a **secondary click from a touch long-press**, so
//! the two are one gesture and this is one design: `Response::context_menu` on
//! the row, and the platform-native trigger falls out with nothing written for
//! it.
//!
//! # It is the composer's second row, on the row
//!
//! The seam is already drawn and this file does not draw a second one:
//! `crate::ui::composer::acts` is *the acts that spend no words*, and those are
//! exactly the acts a list row can offer. `send` and `interrupt` stay off it,
//! together, because both advance the turn and both spend the composer's ONE
//! draft box — an item pointing at a box shared by two verbs names neither.
//!
//! So every item here is one of two things and the separator is where the
//! difference is:
//!
//! - **it fires**, because the gesture is the wall and the conversation and
//!   nothing else ([`straight`]) — [`crate::verbs::STOP`],
//!   [`crate::verbs::RETARGET`], and [`crate::verbs::SEEN`] on a row that is
//!   asking;
//! - **it leads somewhere and spends nothing here** — the records pane, and the
//!   two boxes on the composer that fill an act this menu cannot
//!   (`crate::ui::model::fill`).
//!
//! # Its subject is the ROW, and only the items that lead somewhere select it
//!
//! This is `crate::ui::queue`'s division read one pane over, not a new one: a
//! queue row's `seen` acts on that row without selecting it, and only *go to
//! it* moves the focus. A fired act here carries `(aim.address, row.root_id)`
//! outright, so it needs no selection and takes none — a secondary click that
//! stops a driver must not throw away the transcript the operator was reading.
//! The three that lead somewhere DO select, because each opens a surface whose
//! subject is the selected conversation, and a place opened about nothing would
//! be a place about the wrong thing.
//!
//! # Nothing here can destroy, which is §4.20 read on a control
//!
//! *"A routine pane is a surface an operator moves through quickly; a mis-aimed
//! click there must not be able to land on this."* A list row's menu is that
//! surface exactly — the next row is one pixel away and the menu opens under
//! the pointer. So `delete` does not fire here: the item OPENS the arming,
//! which is the box on the composer, and the act stays where an operator has to
//! reach for it deliberately.

use crate::reply::convs::ConvRow;
use crate::ui::composer::acts::{DELETE, FLAG, RETARGET, STOP};
use crate::ui::{Aim, Fill, Model};

/// **What a word wears when the item leads somewhere rather than acting.** The
/// ellipsis is the whole of the convention and it has one home: an item that
/// carries it spends nothing now.
pub fn leads_to(word: &str) -> String {
    format!("{word}…")
}

/// The door every fired item goes through: a wall and a conversation, and
/// nothing else. **That signature is the admission test** — an act that needs a
/// third parameter has a box, and a box is not something a menu can hold.
type Door = fn(String, String) -> serde_json::Value;

/// **The acts this menu fires outright**: the word on the item, the op it
/// carries, and the door that builds it.
///
/// [`crate::verbs::SEEN`] rides only on a row that is **asking** — it answers
/// what a conversation is currently asking about, so on a row with nothing
/// waiting it is an act with no subject. That is the one convenience this menu
/// adds over the composer's second row, and it is offered where the row itself
/// says it applies.
fn straight(row: &ConvRow) -> Vec<(&'static str, &'static str, Door)> {
    let mut out: Vec<(&'static str, &'static str, Door)> = vec![
        (STOP, crate::verbs::STOP.word, crate::verbs::stop),
        (
            RETARGET,
            crate::verbs::RETARGET.word,
            crate::verbs::retarget,
        ),
    ];
    if row.attention > 0 {
        out.push((
            crate::ui::queue::SEEN,
            crate::verbs::SEEN.word,
            crate::verbs::seen,
        ));
    }
    out
}

/// Hang the menu off a painted row and take what it was given.
///
/// `aim` is the wall the list is showing, which is what every gesture composed
/// here is addressed with — the row's own list is the aimed wall's, so there is
/// no second resolution to get wrong.
pub fn show(response: &egui::Response, model: &mut Model, aim: &Aim, row: &ConvRow) {
    response.context_menu(|ui| {
        let mut chose = false;
        for (word, op, door) in straight(row) {
            let item = ui.button(word);
            crate::ui::act::tag(&item, &[op]);
            if item.clicked() {
                model.outbox.push(crate::ui::Posted::act(door(
                    aim.address.clone(),
                    row.root_id.clone(),
                )));
                chose = true;
            }
        }
        ui.separator();
        // **The reads this gesture reaches** (§4.18), tagged exactly as the
        // composer's own control is: opening the pane is what makes this seat
        // read the conversation's steps and files, and the reads have no
        // control of their own.
        let records = ui.button(crate::ui::records::OPEN);
        crate::ui::act::tag(
            &records,
            &[crate::verbs::STEPS.word, crate::verbs::FILES.word],
        );
        if records.clicked() {
            model.select(&row.root_id);
            model.begin_records();
            chose = true;
        }
        // **No `act:` token on either**, because neither crosses a wire: each
        // selects the conversation and puts the cursor in the box that fills
        // its act, which is a view (§4.16) and out of the parity contract. The
        // acts themselves are tagged where they fire, on the composer.
        for (word, fill) in [(FLAG, Fill::Reason), (DELETE, Fill::Arming)] {
            if ui.button(leads_to(word)).clicked() {
                model.fill_in(&row.root_id, fill);
                chose = true;
            }
        }
        // **Closed once, at the end.** `Ui::close_menu` drops the menu state
        // off this `Ui`, so a second call inside the same pass propagates
        // nothing — and an item that fired while a later one still had to be
        // painted would have left the menu standing over a spent gesture.
        if chose {
            ui.close_menu();
        }
    });
}

#[cfg(test)]
mod tests;
