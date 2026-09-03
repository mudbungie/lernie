//! **The tuning pane between frames**, and the four acts its controls spend.
//!
//! Its type and its acts are one file rather than the two the enrollment uses
//! (`crate::ui::model::enroll` beside `crate::ui::model::acts`), because
//! `acts.rs` is at its design-time budget and because these four acts have no
//! reader but this type. The rule that governs both placements is the same one:
//! **a binding names a control that already exists**, so what a click means
//! lives once, wherever it lives.
//!
//! # The pane is a two-state thing, not a flag beside an option
//!
//! It is open showing rows, or open with one role's assignment being rewritten.
//! Spelling that as `open: bool` beside `editing: Option<Edit>` would admit a
//! fourth state that means nothing — editing while closed — and a state that
//! means nothing is a state some frame eventually paints. [`Tuning`] has the
//! two the pane has.
//!
//! # What is NOT held here is the answer
//!
//! The rows come off the wire into [`Model::roles`] and are the engine's, not
//! the pane's. **That field is one option where the conversation list needs a
//! pair**, and the difference is real: an empty `Vec` is a wall whose config
//! declares no role — a state a fresh workspace is really in — and `None` is a
//! wall nobody has been answered about. The list next door needs `answered`
//! beside it only because it also carries rows the engine never sent.
//!
//! The read is standing while the pane is open
//! (`crate::state::Standing`), so a tuning act that lands is reflected by the
//! next answer rather than by this end predicting one. A seat that wrote its
//! own row back would be painting a claim the engine had not made — and the
//! three writes go through `litany config`, which can refuse.
//!
//! The one thing held is the **draft** of an assignment, because two words
//! typed into two boxes are not a fact about anything until they are sent.

use super::{Aim, Model};
use crate::reply::roles::RoleRow;

/// The tuning pane, while it is open.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Tuning {
    /// The rows, each with its own controls.
    Rows,
    /// One role's assignment being rewritten, under the row it belongs to.
    Editing(Edit),
}

/// **An assignment being rewritten**: which role, and the two words for it.
///
/// It is seeded from the row it was opened on, so the boxes open holding what
/// is in force — the whole point of the `roles` read being the pane's first
/// question — and an operator changing one word does not have to retype the
/// other.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Edit {
    /// The role, which is the address the write takes and is never edited: a
    /// role the config does not declare is refused, so a box for it would be a
    /// box whose only other value is a refusal.
    pub role: String,
    pub provider: String,
    pub model: String,
}

impl Edit {
    /// Seed one from the row it was opened on.
    pub fn of(row: &RoleRow) -> Self {
        Self {
            role: row.role.clone(),
            provider: row.provider.clone(),
            model: row.model.clone(),
        }
    }

    /// **Whether it names both halves of an assignment.** A provider with no
    /// model and a model on no provider are each half a fact, and the write
    /// takes the pair.
    pub fn ready(&self) -> bool {
        !self.provider.trim().is_empty() && !self.model.trim().is_empty()
    }
}

impl Model {
    /// **Open the tuning pane on the wall the window is aimed at**, or do
    /// nothing where it is aimed at none.
    ///
    /// The aim is the gate rather than a separate check, exactly as it is for
    /// the enrollment: every gesture the pane composes carries a workspace, and
    /// a workspace is what an aim is.
    pub fn begin_tuning(&mut self) {
        if self.aim.is_some() {
            self.tuning = Some(Tuning::Rows);
        }
    }

    /// **Close it**, dropping any draft with it. Nothing else is touched — the
    /// rows stay, because the next open on the same wall is about the same
    /// rows and the standing read replaces them anyway.
    pub fn close_tuning(&mut self) {
        self.tuning = None;
    }

    /// **Begin rewriting one role's assignment**, seeded from what is in force.
    pub fn edit_assignment(&mut self, row: &RoleRow) {
        if self.tuning.is_some() {
            self.tuning = Some(Tuning::Editing(Edit::of(row)));
        }
    }

    /// **Put the draft down** and go back to the rows. It is not the same act
    /// as closing the pane, which is why it is not the same control.
    pub fn cancel_assignment(&mut self) {
        if self.tuning.is_some() {
            self.tuning = Some(Tuning::Rows);
        }
    }

    /// **The draft, if one is open.** The pane paints the editor under the row
    /// it belongs to, so it asks by role rather than for the whole state.
    pub fn editing(&self, role: &str) -> Option<Edit> {
        match &self.tuning {
            Some(Tuning::Editing(edit)) if edit.role == role => Some(edit.clone()),
            _ => None,
        }
    }

    /// **Take the draft for a box to type into**, where one is open. The pane
    /// needs a place to write and the model is the only one there is.
    pub fn draft_assignment(&mut self) -> Option<&mut Edit> {
        match &mut self.tuning {
            Some(Tuning::Editing(edit)) => Some(edit),
            _ => None,
        }
    }

    /// **Ask this role's model calls for `level` much reasoning**, or for none.
    pub fn post_effort(&mut self, role: &str, level: Option<String>) {
        self.tune(&|address| crate::verbs::effort(address, role.to_owned(), level.clone()));
    }

    /// **Turn this role's priority lane on or off.**
    pub fn post_priority(&mut self, role: &str, on: bool) {
        self.tune(&|address| crate::verbs::priority(address, role.to_owned(), on));
    }

    /// **Spend the draft assignment**, and go back to the rows.
    ///
    /// It composes and then closes the editor, unlike the enrollment's own
    /// post: an enrollment has to stand until its answer arrives because the
    /// answer *is* the product, and this act's product is a row the standing
    /// read will bring back on its own.
    pub fn post_assignment(&mut self) {
        let Some(edit) = self.draft_assignment().filter(|edit| edit.ready()).cloned() else {
            return;
        };
        self.tune(&|address| {
            crate::verbs::model(
                address,
                edit.role.clone(),
                edit.provider.trim().to_owned(),
                edit.model.trim().to_owned(),
            )
        });
        self.cancel_assignment();
    }

    /// **Compose one tuning gesture against the aimed wall**, or none at all.
    ///
    /// The one door the three writes share, because all three carry the same
    /// address and the aim is the same gate for each: a pane open with nothing
    /// aimed at is a state [`Model::begin_tuning`] refuses to create, and this
    /// is the reading that makes it unreachable rather than merely unlikely.
    ///
    /// **`&dyn Fn` rather than a generic**, which is the house preference for a
    /// door taking behaviour and is also the honest shape here: a bound would
    /// stamp out one copy of this body per call site, and the arm that costs
    /// nothing to run is then three arms a suite has to reach separately to say
    /// the same thing once.
    fn tune(&mut self, gesture: &dyn Fn(String) -> serde_json::Value) {
        let Some(aim) = self.aim.clone() else {
            return;
        };
        let Aim { address, .. } = aim;
        self.outbox.push(gesture(address));
    }
}

#[cfg(test)]
mod tests;
