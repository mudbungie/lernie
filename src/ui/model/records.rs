//! **The records pane between frames** (bl-2cf7): open or not, and the acts
//! that open and close it.
//!
//! # A flag, because the pane has one state
//!
//! The tuning pane is a two-state enum because an assignment can be mid-edit;
//! this pane holds nothing of its own — every row on it is the engine's answer
//! and every answer lives on [`Model`] beside the roles, filed whether or not
//! the pane is open. So `records` is a `bool`, and a struct here would be a
//! state machine with one state.
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

use super::Model;

impl Model {
    /// **Whether a pane covers the conversation** — the enrollment, the
    /// tuning pane, the decision queue, an unmaking, or this one. The question
    /// the shell and every pane-opening control share, asked once so five panes
    /// cannot stand on one glass: a control that opened a second cover would
    /// replace what is standing without saying so.
    pub fn covered(&self) -> bool {
        self.enroll.is_some()
            || self.tuning.is_some()
            || self.records
            || self.queue
            || self.lookup.is_some()
            || self.unmaking.is_some()
    }

    /// **Open the records pane on the selected conversation**, or do nothing
    /// where none is selected. The selection is the gate exactly as the aim is
    /// tuning's: the two reads it stands up are addressed, and a selected
    /// conversation is what an address is.
    pub fn begin_records(&mut self) {
        if self.conversation.is_some() {
            self.records = true;
        }
    }

    /// **Close it.** The answers stay, for the same reason the roles do: the
    /// next open on the same conversation is about the same records, and the
    /// standing read replaces them anyway.
    pub fn close_records(&mut self) {
        self.records = false;
    }

    /// **The records go with the conversation they answer** — called by the
    /// two acts that move the subject, so the pane and its rows never outlive
    /// what they are about.
    pub(super) fn retire_records(&mut self) {
        self.records = false;
        self.steps = None;
        self.files = None;
    }
}

#[cfg(test)]
mod tests;
