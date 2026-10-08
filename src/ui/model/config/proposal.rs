//! **The staged proposals between frames** (bl-a1d6; DESIGN §4.30) — which
//! row the config pane has named, what has been typed to arm a verdict on it,
//! and whether one has been asked for.
//!
//! # One standing read at two depths, and the depth is a derivation
//!
//! `proposals` is one op at two depths (yog REMOTE §9.22): bare it lists, and
//! naming an id answers the listing AND that proposal whole. So naming a row
//! does not post a second read — it deepens the one that already stands, and
//! [`Model::proposing`] is the depth. It is the named id **only while the
//! engine's own last listing still carries it**: a proposal that was accepted
//! or rejected leaves the listing, and asking for it whole after that would
//! earn a refusal on every beat for a row that is simply gone. Nothing records
//! the departure; the listing is the record.
//!
//! # The arming is the seat's, and the subject is the pane's
//!
//! The wire takes `proposal` with no `typed` at all, so by DESIGN §4.20's
//! amendment the arming is an ENABLEMENT this seat holds and never sends: the
//! proposal's own id typed back, surrounding whitespace forgiven and nothing
//! else. It earns one by SCOPE — an accept moves a lineage every conversation
//! on it resolves at its next step, which is beyond the row on screen, and a
//! reject throws a reviewer's work away.
//!
//! The subject is held rather than followed for §4.20's reason, and costs no
//! field: the id is held here, and the workspace is the aim's, which cannot
//! move under this pane — the act that moves the aim retires the whole config
//! pane (`Model::retire_configuring`). The arming is **not spent on firing**:
//! a stale accept refuses in litany's own words, and clearing the box would
//! charge a retype for the engine's *no*.

use crate::reply::proposals::Proposal;
use crate::ui::Model;

/// **One named proposal**, from the row control that named it to the way out.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Proposing {
    /// The reviewer id the row is addressed by — the read's depth and the
    /// settle's subject.
    pub id: String,
    /// The arming: that id, typed back.
    pub typed: String,
    /// **Whether a verdict has been asked for**, for the unmaking's reason: the
    /// outbox is drained within the beat, so nothing else on the glass answers
    /// *has this been asked*.
    pub posted: bool,
}

impl Proposing {
    /// **Whether the typed id arms a verdict.**
    pub fn armed(&self) -> bool {
        self.typed.trim() == self.id
    }
}

impl Model {
    /// **Name one row**, which deepens the standing read to it. Whatever the
    /// last named row answered whole goes with it: the answer carries no id,
    /// so a diff left standing under a new name would be unattributable — the
    /// rule [`Model::read_config`] keeps one read over.
    pub fn name_proposal(&mut self, id: &str) {
        let Some(pane) = self.configuring.as_mut() else {
            return;
        };
        pane.proposal = Some(Proposing {
            id: id.to_owned(),
            ..Proposing::default()
        });
        self.unwhole();
    }

    /// **Put it down** — the way out, and it settles nothing: the proposal
    /// stays staged and the read goes back to the bare listing.
    pub fn unname_proposal(&mut self) {
        if let Some(pane) = self.configuring.as_mut() {
            pane.proposal = None;
        }
        self.unwhole();
    }

    /// **The named row, as the pane holds it.**
    pub fn named_proposal(&self) -> Option<Proposing> {
        self.configuring.as_ref()?.proposal.clone()
    }

    /// **The arming box, to paint into** — [`Model::draft_box`]'s shape.
    pub(crate) fn verdict_box(&mut self) -> Option<&mut String> {
        Some(&mut self.configuring.as_mut()?.proposal.as_mut()?.typed)
    }

    /// **The row the engine's last listing holds under `id`**, or `None` where
    /// it holds none — never answered, or since settled.
    pub fn staged_proposal(&self, id: &str) -> Option<Proposal> {
        self.proposals
            .as_ref()?
            .rows
            .iter()
            .find(|row| row.id == id)
            .cloned()
    }

    /// **The depth the standing read is asked at**: the named id while the
    /// listing still carries it, and the bare listing otherwise (module doc).
    pub fn proposing(&self) -> Option<String> {
        let id = self.named_proposal()?.id;
        self.staged_proposal(&id).map(|row| row.id)
    }

    /// **Ask for one verdict on the named row**, where it is armed and still
    /// staged and never otherwise. `verdict` is one of
    /// [`crate::verbs::proposals::VERDICTS`], spelled by the control that fires
    /// it.
    pub fn settle_proposal(&mut self, verdict: &str) {
        let live = self.proposing().is_some();
        let Some(workspace) = self.aim.as_ref().map(|aim| aim.address.clone()) else {
            return;
        };
        let Some(held) = self
            .configuring
            .as_mut()
            .and_then(|pane| pane.proposal.as_mut())
            .filter(|held| live && held.armed())
        else {
            return;
        };
        held.posted = true;
        let gesture = crate::verbs::proposal(workspace, held.id.clone(), verdict.to_owned());
        self.outbox.push(crate::ui::Posted::act(gesture));
    }

    /// Drop a held answer's whole, keeping its rows.
    fn unwhole(&mut self) {
        if let Some(held) = self.proposals.as_mut() {
            held.whole = None;
        }
    }
}

#[cfg(test)]
mod tests;
