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

/// **What the records pane has been answered** — its seven reads, held
/// together.
///
/// One struct rather than seven fields on [`Model`], for `super::listing`'s
/// reason one layer over: they are one pane's questions about one subject, and
/// they are retired together the moment that subject moves
/// ([`Model::retire_records`]). Seven options spread across the model would be
/// seven places to remember, and forgetting one paints a conversation's
/// records under another conversation's name.
///
/// Every one is `None` until an answer lands, which is the reading
/// [`Model::roles`]'s option carries: nobody has been answered is not the same
/// claim as a conversation whose loop did nothing.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Records {
    /// The steps its loop has taken (§4.18).
    pub steps: Option<crate::reply::steps::Steps>,
    /// What its worktree holds (§4.18).
    pub files: Option<crate::reply::files::Files>,
    /// Its spine — every operable commit, and the cards off them (§4.29).
    pub rail: Option<crate::reply::rail::Rail>,
    /// The config commit governing it (§4.29).
    pub governing: Option<crate::reply::governing::Governing>,
    /// Its own row, whole — the pane's header (§4.32).
    pub agent: Option<crate::reply::agent::Agent>,
    /// Its undelivered mail (§4.32).
    pub mail: Option<Vec<crate::reply::inbox::Row>>,
    /// **One step's records**, and the one of the seven that is POSTED rather
    /// than standing: it is asked about a row, by the control on that row.
    /// Which step it is about is the answer's own `seq`, so nothing here is a
    /// second name for it (`super::deep`).
    pub drilled: Option<crate::reply::step::Step>,
}

impl Model {
    /// **Whether a pane covers the conversation** — the enrollment, the
    /// tuning pane, one of the three listings (`super::listing`), the window's
    /// own FOUR, the login pane, the config pane, or an unmaking. The question
    /// the shell and every pane-opening control share, asked once so twelve
    /// panes cannot stand on one glass: a control that opened a second cover
    /// would replace what is standing without saying so.
    pub fn covered(&self) -> bool {
        self.enroll.is_some()
            || self.tuning.is_some()
            || self.listing.is_some()
            || self.lookup.is_some()
            || self.login.is_some()
            || self.configuring.is_some()
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
        // **One assignment, because the answers are one value** ([`Records`]):
        // a pane whose reads are retired field by field is a pane with a read
        // somebody will forget.
        self.records = Records::default();
        // **The draft goes with them** (`super::spine`), which the answers do
        // not need to say: a goal typed for one conversation is a sentence
        // about that one, and a box left standing over a new selection would
        // fire it at whatever is selected next.
        self.forking = crate::ui::Forking::default();
    }
}

#[cfg(test)]
mod tests;
