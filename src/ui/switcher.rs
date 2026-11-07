//! **The switcher**: the roster as an overlay (DESIGN §4.42, item 7). Ctrl+X
//! or `/conversations` opens it; it lists every server this seat reaches as
//! a heading row, each workspace under it, and each conversation under its workspace —
//! the accordion §4.39 built, with the `+` that begins a conversation on a
//! server's row — and it closes when a conversation is chosen.
//!
//! **The list is the roster's own paint, unchanged.** What moved is where it
//! stands: the one list used to be a column beside the conversation, and a
//! column is a thing the eye has to divide the window with whether or not
//! it is looking there. An overlay is asked for and goes away, which is the
//! baseline's shape and the whole of what this ball changes about the list.
//!
//! **Walking keeps it open; choosing closes it.** The arrows select as they
//! walk (`crate::ui::keys` — the cursor IS the selection), so a walk that
//! closed the overlay on the first conversation it crossed would be a walk
//! nobody could use. A click on a conversation, the `+`, and Enter on a
//! selection are the choosing ([`Model::chose`]), and the one act that opens
//! a tab here; Esc leaves without one ([`Model::close_switcher`]). The four
//! acts are the model's (`crate::ui::model::walk`); this file is the paint.
//! **A click on a workspace row is a step of the walk, not a choosing**
//! (bl-1ea2): it aims, the workspace's conversations unfold under it, and the
//! switcher stands — so the Ctrl+X after it closes it, one press, one toggle.

use crate::ui::{Model, roster};

/// The word typed after the slash that opens it.
pub const WORD: &str = "conversations";
/// Its name on the glass: the roster's own word.
pub const HEADING: &str = roster::HEADING;

/// Paint the switcher in the overlay's frame, and say whether it stood.
pub fn render(ui: &mut egui::Ui, model: &mut Model) -> bool {
    if !model.switching {
        return false;
    }
    crate::ui::overlay::name(ui, HEADING);
    // **The queue's door stands at its head** (§4.42, item 11): the count
    // stands at the status bar's right end too, and the switcher is the
    // other seat-wide list.
    if crate::ui::status::waiting::door(ui, model) {
        model.close_switcher();
        crate::ui::slash::fire(model, crate::ui::queue::WORD, "");
    }
    roster::render(ui, model);
    true
}

#[cfg(test)]
mod tests;
