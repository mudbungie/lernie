//! **The spine between frames** (bl-b52c): the draft a fork is composed from,
//! and the one act it spends.
//!
//! # The records pane held no state, and now it holds exactly one draft
//!
//! DESIGN §4.18 recorded that this pane holds nothing at all — *"not even a
//! draft, which is one less than tuning holds"* — and that was true of a pane
//! whose whole content was two reads. The spine brings an ACT, and an act with
//! parameters needs somewhere to type them, so the pane arrives at §4.17's
//! shape after all: **one piece of state, the draft of the gesture**, because
//! two words typed into two boxes are not a fact about anything until they are
//! sent.
//!
//! It is a bare field rather than an option, exactly as the composer's own two
//! parameter boxes are (`Model::reason`, `Model::typed`): the boxes are on the
//! glass whenever the half is, so there is no *closed* state for an option to
//! represent.
//!
//! # The goal is spent and the role is kept, and the split is the acts' own
//!
//! The composer already draws this line twice. A flag's reason is **taken** on
//! firing, because what a flag says is said; an unmaking's arming is **cloned**,
//! because the refusal is the common answer and clearing it would charge a
//! retype for the engine's *no*. A fork's goal is the first of those — it was
//! said, to a child that now exists — and its role is the second: a role is a
//! name off a config lineage, the same name the next attempt off the same spine
//! wants, and re-typing it per notch would be a toll on the ordinary act.

use super::Model;

/// **The draft of a fork**: the two words the operator supplies, beside the
/// `from` the control itself carries off the notch it hangs on.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Forking {
    /// The role, which is the model: litany resolves the provider and model id
    /// from this name against the fork point's own governing config.
    pub role: String,
    /// What the child is being asked to do.
    pub goal: String,
}

impl Forking {
    /// **Whether both words are there.** The wire refuses a fork with an empty
    /// `from`, `role` or `goal`, so the control is disabled until the two this
    /// box holds are non-blank — the tuning pane's `set`, not the composer's
    /// second start: the parameters are missing, not the subject.
    pub fn ready(&self) -> bool {
        !self.role.trim().is_empty() && !self.goal.trim().is_empty()
    }
}

impl Model {
    /// **Fork the selected conversation from `commit`**, or do nothing where
    /// the draft is half-typed, nothing is aimed at, or nothing is selected.
    ///
    /// The three gates are one reading rather than three arms of a control:
    /// every state they exclude is one the pane cannot paint a fork control in,
    /// so this is what makes them unreachable rather than merely unlikely —
    /// `crate::ui::model::tuning::tune`'s own shape, one noun over.
    pub fn post_fork(&mut self, commit: String) {
        let (Some(aim), Some(parent)) = (self.aim.clone(), self.conversation.clone()) else {
            return;
        };
        if !self.forking.ready() {
            return;
        }
        let role = self.forking.role.trim().to_owned();
        let goal = std::mem::take(&mut self.forking.goal);
        self.outbox.push(super::Posted::act(crate::verbs::fork(
            aim.address,
            parent,
            commit,
            role,
            goal.trim().to_owned(),
        )));
    }
}

#[cfg(test)]
mod tests;
