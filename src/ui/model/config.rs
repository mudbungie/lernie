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
//! # The box is the write, and the seed is what makes the write ordinary
//!
//! A config write replaces a file's whole bytes (REMOTE §9.18: *"a typed edit
//! is a seat composing that text and applying it"*), so the act is the box —
//! and reaching it means having authored the file, which no mis-aimed click
//! does. That is why it takes DESIGN §4.20's ENABLEMENT and not its PLACE:
//! §4.20 is for an act whose subject ceases to exist, and this one's subject
//! is a file that still exists afterwards holding text the operator wrote.
//!
//! What is NOT ordinary about it is the thing the wire dropped. Upstream's
//! editors carry a hash guard because *"a long-lived RAM draft"* can be
//! written over a file that moved under it, and a `config` act *"carries no
//! hash guard, and needs none"* because a gesture states its whole text in one
//! atomic instruction — true of the gesture and false of the OPERATOR, who
//! types for minutes while a standing read replaces the answer underneath
//! them. [`Draft::seed`] is that guard restated as a READING rather than as a
//! refusal, which is this seat's whole division of labour: the pane says the
//! file went somewhere else, and the engine remains the only thing that
//! judges.
//!
//! # Both reads STAND, and the second only once a file is picked
//!
//! A config file changes under the operator — a `litany config` from any seat,
//! an agent's own edit, a lineage that advanced — so the read that says what it
//! holds is worth nothing unless it is asked again. The lineage listing stands
//! on the pane and the file read stands on the destination, which is
//! `crate::state::Open::Config`'s own shape.

/// The box's own two readings, split from the pane's state at the design-time
/// budget on the seam they already have: this file is *what the pane is
/// pointed at*, and that one is *what the box and the file have to say to each
/// other*.
mod draft;
pub use draft::Draft;

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
    /// **The bytes in the editor**, once the file has answered once — there is
    /// nothing to edit before that, and a box seeded from nothing would invite
    /// a write of nothing over a file nobody has seen.
    pub draft: Option<Draft>,
    /// **The name the workflow destination is addressed by.**
    ///
    /// It lives on the pane rather than inside a [`Where`] because it is what
    /// the operator is TYPING, not what they picked: `litany-workflow` is
    /// addressed by a name and no read this seat has says which names exist
    /// (DESIGN §4.30), so the box is the listing. Empty is not a destination,
    /// which is what keeps the control an enablement rather than a refusal.
    pub workflow: String,
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
        pane.draft = None;
        self.config = None;
    }

    /// **The box, seeded from the file the first time it answers and settled
    /// against every answer after.** Called by the frame that paints it, which
    /// is the one place that holds the engine's answer and the box at once.
    pub fn draft_config(&mut self, answered: &str) {
        let Some(pane) = self.configuring.as_mut() else {
            return;
        };
        match pane.draft.as_mut() {
            Some(draft) => draft.settle(answered),
            None => pane.draft = Some(Draft::of(answered)),
        }
    }

    /// **The box's own bytes, to paint into.** `pub(crate)` and borrowing,
    /// which the house rules permit for an internal accessor: a box is written
    /// by the widget and there is nothing to hand back by value.
    pub(crate) fn draft_box(&mut self) -> Option<&mut String> {
        Some(&mut self.configuring.as_mut()?.draft.as_mut()?.text)
    }

    /// **The workflow name box, to paint into** — [`Self::draft_box`]'s shape
    /// for the one destination addressed by a name rather than picked.
    pub(crate) fn workflow_box(&mut self) -> Option<&mut String> {
        Some(&mut self.configuring.as_mut()?.workflow)
    }

    /// **The workflow name as a destination would carry it**, trimmed —
    /// leading and trailing space is typing, never part of a file's name, and
    /// the empty string is the pane holding no fourth destination at all.
    pub fn workflow_named(&self) -> String {
        self.configuring
            .as_ref()
            .map(|pane| pane.workflow.trim().to_owned())
            .unwrap_or_default()
    }

    /// **The draft as a value**, for the frame's own readings.
    pub fn drafted(&self) -> Option<Draft> {
        self.configuring.as_ref()?.draft.clone()
    }

    /// **Put the engine's answer back in the box** — the way out of an edit,
    /// and the way to take another writer's file over your own draft. It is
    /// one gesture for both because it is one act: the box becomes the file.
    pub fn revert_config(&mut self, answered: &str) {
        let Some(pane) = self.configuring.as_mut() else {
            return;
        };
        pane.draft = Some(Draft::of(answered));
    }

    /// **Write the box to the file it was read from.**
    ///
    /// The address is the read half's, spelled once more rather than derived a
    /// second way (DESIGN §4.30): a destination naming a workspace is routed by
    /// it, and one naming the ENGINE is addressed down the channel the window
    /// is aimed at — never fanned, which is what would put the operator's text
    /// on every engine this box is a client of (bl-4855).
    pub fn write_config(&mut self, at: &Where, text: String) {
        let posted = super::Posted::act(crate::verbs::write(at, text));
        let posted = if at.addresses_a_workspace() {
            posted
        } else {
            let Some(down) = self.channel() else {
                return;
            };
            posted.down(down)
        };
        self.outbox.push(posted);
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
