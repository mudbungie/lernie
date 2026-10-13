//! **The switcher**: the roster as an overlay (DESIGN §4.42, item 7). Ctrl+X
//! or `/conversations` opens it; it lists every engine this seat reaches as
//! a heading row, each wall under it, and each conversation under its wall —
//! the accordion §4.39 built, with the `+` that begins a conversation on an
//! engine's row — and it closes when a conversation is chosen.
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
//! selection are the choosing ([`Model::chose`]); Esc leaves without one.

use crate::ui::{Model, roster};

/// The word typed after the slash that opens it.
pub const WORD: &str = "conversations";
/// Its name on the glass: the roster's own word.
pub const HEADING: &str = roster::HEADING;

impl Model {
    /// **Open the switcher**, or close it: the one toggle Ctrl+X is.
    pub fn switch(&mut self) {
        self.switching = !self.switching;
    }

    /// **A conversation was chosen**, so the switcher has done its work.
    /// Called by the click on a conversation's row, by the `+`, and by Enter
    /// on a selection — never by the walk.
    pub fn chose(&mut self) {
        self.switching = false;
    }
}

/// Paint the switcher in the overlay's frame, and say whether it stood.
pub fn render(ui: &mut egui::Ui, model: &mut Model) -> bool {
    if !model.switching {
        return false;
    }
    crate::ui::overlay::name(ui, HEADING);
    roster::render(ui, model);
    true
}

#[cfg(test)]
mod tests;
