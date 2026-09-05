//! **The config pane between frames** (bl-5c53; DESIGN §4.30) — which
//! destination it is pointed at, and the two reads that stand on it.
//!
//! # A struct of one question, and the question is which file
//!
//! `super::listing`'s three panes hold nothing; this one holds exactly one
//! thing — the destination — and it is an option inside the pane's own option
//! for `super::login`'s reason: *a subject with no pane* is unrepresentable
//! because the subject lives inside the pane. The pane opens pointed at
//! nothing, which is a real state an operator meets: the lineage listing is up
//! and no file has been asked for yet.
//!
//! # What is NOT held here is either answer
//!
//! [`Model::lineages`] and [`Model::config`] are the engine's, filed on the
//! model beside the roles and the provider table for their reason verbatim: a
//! pane that wrote its own bytes back would be painting a file the engine had
//! not written. That matters more here than anywhere else on this surface,
//! because the answer carries the engine's own JUDGEMENT of the values
//! (`crate::reply::config::Setting::fault`) and a seat that re-derived one
//! would be a second authority across a boundary.
//!
//! # Both reads STAND, and the second only once a file is picked
//!
//! A config file changes under the operator — a `litany config` from any seat,
//! an agent's own edit, a lineage that advanced — so the read that says what it
//! holds is worth nothing unless it is asked again. The lineage listing stands
//! on the pane and the file read stands on the destination, which is
//! `crate::state::Open::Config`'s own shape.

use super::Model;
use crate::verbs::Where;

/// The config pane, while it is open.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Configuring {
    /// **Which file is being read**, or `None` where the pane is open on the
    /// listing alone. It is the file read's whole subject
    /// (`crate::state::Standing::at`), so closing the pane ends that read and
    /// picking another replaces it.
    pub at: Option<Where>,
}

impl Model {
    /// **Open the config pane on the wall the window is aimed at**, or do
    /// nothing where it is aimed at none — the aim being the gate for
    /// [`Model::begin_tuning`]'s reason: the lineage read carries a workspace,
    /// and a workspace is what an aim is.
    pub fn begin_configuring(&mut self) {
        if self.aim.is_some() {
            self.configuring = Some(Configuring::default());
        }
    }

    /// **Close it.** The answers stay, for the reason the roles do: the next
    /// open on the same wall is about the same files, and the standing reads
    /// replace them anyway.
    pub fn close_configuring(&mut self) {
        self.configuring = None;
    }

    /// **The destination the file read is standing on**, if any — the pane's
    /// own question, asked once so the pane and the standing set cannot
    /// disagree about which file is up.
    pub fn configured(&self) -> Option<Where> {
        self.configuring.as_ref()?.at.clone()
    }

    /// **Point the pane at one file**, dropping whatever the last one
    /// answered.
    ///
    /// The drop is the act's own half of the read: the answer carries no
    /// destination — upstream answers the bytes and the schema, not what was
    /// asked — so a listing left standing under a new question would be
    /// unattributable, exactly as the login pane's offering is.
    pub fn read_config(&mut self, at: &Where) {
        let Some(pane) = self.configuring.as_mut() else {
            return;
        };
        pane.at = Some(at.clone());
        self.config = None;
    }

    /// **The pane and both answers go with the wall they are about** — called
    /// by the act that moves the aim, so nothing on it outlives its subject.
    /// It is `super::login::Model::retire_login`'s rule one noun over, and
    /// sharper: a destination carries the aim's own workspace inside it.
    pub(super) fn retire_configuring(&mut self) {
        self.configuring = None;
        self.config = None;
        self.lineages = None;
    }
}

#[cfg(test)]
mod tests;
