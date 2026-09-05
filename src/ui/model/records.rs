//! **The records pane between frames** (bl-2cf7): open or not, and the acts
//! that open and close it.
//!
//! # A flag, because the pane has one state
//!
//! The tuning pane is a two-state enum because an assignment can be mid-edit;
//! this pane's own state is one draft and no mode — every row on it is the
//! engine's answer, filed on [`Model`] beside the roles whether or not the
//! pane is open, and the fork's two words are a bare field (`super::spine`).
//! So `records` is a `bool`, and a struct here would be a state machine with
//! one state.
//!
//! # It follows the tuning pane's whole shape, one noun over
//!
//! The subject is the SELECTED CONVERSATION where tuning's is the aimed wall,
//! and every consequence carries across: the pane opens only where there is a
//! subject; the two reads stand while it is open and only then
//! (`crate::state::Standing::records`), so what it paints is always the
//! engine's fact and never this end's prediction; and it closes when the
//! subject moves ([`Model::select`], [`Model::aim_at`]), because a pane about
//! *the selected conversation* left open over a new selection would paint one
//! conversation's records under another's name for a beat.

use super::{Listing, Model};

impl Model {
    /// **Whether a pane covers the conversation** — the enrollment, the
    /// tuning pane, one of the three listings (`super::listing`), the window's
    /// own three, the login pane, or an unmaking. The question the shell and
    /// every pane-opening control share, asked once so ten panes cannot stand
    /// on one glass: a control that opened a second cover would replace what
    /// is standing without saying so.
    pub fn covered(&self) -> bool {
        self.enroll.is_some()
            || self.tuning.is_some()
            || self.listing.is_some()
            || self.lookup.is_some()
            || self.login.is_some()
            || self.unmaking.is_some()
    }

    /// **Open the records pane on the selected conversation**, or do nothing
    /// where none is selected. The selection is the gate exactly as the aim is
    /// tuning's: the two reads it stands up are addressed, and a selected
    /// conversation is what an address is.
    pub fn begin_records(&mut self) {
        if self.conversation.is_some() {
            self.stand(Listing::Records);
        }
    }

    /// **Close it.** The answers stay, for the same reason the roles do: the
    /// next open on the same conversation is about the same records, and the
    /// standing read replaces them anyway.
    pub fn close_records(&mut self) {
        self.put_down(Listing::Records);
    }

    /// **The records go with the conversation they answer** — called by the
    /// two acts that move the subject, so the pane and its rows never outlive
    /// what they are about.
    pub(super) fn retire_records(&mut self) {
        self.put_down(Listing::Records);
        self.steps = None;
        self.files = None;
        self.rail = None;
        self.governing = None;
        // **The draft goes with them** (`spine`), which the two reads above do
        // not need to say: a goal typed for one conversation is a sentence
        // about that one, and a box left standing over a new selection would
        // fire it at whatever is selected next.
        self.forking = crate::ui::Forking::default();
    }
}

#[cfg(test)]
mod tests;
