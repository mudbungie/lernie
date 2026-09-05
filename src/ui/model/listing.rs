//! **The panes that are pure listings** — the three that hold no question of
//! their own, and the one field that says which of them is standing.
//!
//! # One field, because three flags make two of them open at once representable
//!
//! `super::window`'s [`Lookup`](super::Lookup) drew this a pane earlier and its
//! reasoning is the whole of this module's: *"one field rather than two flags,
//! because no two panes ever stand together and a pair of bools would make
//! **both** representable"*. Three of them made it worse — the model carried a
//! `bool` per pane and only the derivation order in `Model::covered` and
//! `crate::state::Open::of` resolved a state the window cannot reach — and
//! clippy's `struct_excessive_bools` names exactly this reframe when the count
//! reaches four. The clients pane (bl-e53c) was the fourth.
//!
//! # What makes one a LISTING rather than a pane with state
//!
//! Every pane in this window covers the conversation; what these three have in
//! common is that they hold nothing of their own. `super::tuning` holds a
//! draft, `super::login` holds two questions, `super::unmake` holds an arming
//! and a subject, `super::window`'s find pane holds a needle. The records pane,
//! the decision queue and the clients pane hold **only what the engine
//! answered** — so *which one is up* is the entire state, and the answer to
//! that is one word.

use super::Model;

/// Which listing pane is standing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Listing {
    /// **The records pane** — the selected conversation's steps and files
    /// (`super::records`; DESIGN §4.18).
    Records,
    /// **The decision queue** — everything asking for the operator, anywhere
    /// (`super::queue`; DESIGN §4.19).
    Queue,
    /// **The clients pane** — the machines registered in the aimed wall's
    /// workspace (`super::clients`; DESIGN §4.28).
    Clients,
}

impl Model {
    /// **Whether `listing` is the pane standing right now.** The reading every
    /// pane's own `render` opens with, so a pane paints when it is up and at no
    /// other time.
    pub fn showing(&self, listing: Listing) -> bool {
        self.listing == Some(listing)
    }

    /// Stand one up, replacing whatever was there — which is unreachable from
    /// the window, because every control that opens a pane stands down under
    /// [`Model::covered`].
    pub(super) fn stand(&mut self, listing: Listing) {
        self.listing = Some(listing);
    }

    /// **Put one down, and only that one.** A close control names its own
    /// pane, so a stale one cannot take down the pane that replaced it.
    pub(super) fn put_down(&mut self, listing: Listing) {
        if self.showing(listing) {
            self.listing = None;
        }
    }
}

#[cfg(test)]
mod tests;
