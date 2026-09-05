//! **The deeper records between frames** (bl-3257): the one read this pane
//! posts rather than stands, and why it is the one.
//!
//! # Five of the pane's six reads stand, and the sixth is addressed
//!
//! The records pane's other reads are about *the selected conversation*, which
//! is a subject the window already holds, so they stand while the pane is open
//! and cost nothing when it is not (`crate::state::Standing`). The step
//! drill-in is about **one step of it**, and the window holds no such
//! selection — so a standing read would have to invent one and then hold it,
//! which is a second authority for a row an operator clicked.
//!
//! So the control on a steps row posts the read, exactly as the login pane's
//! `models` is posted off a provider row (DESIGN §4.24), and for the same
//! reason: it is a question about a row rather than about the pane.
//!
//! # The answer says which step it is about, so the model holds no second name
//!
//! `reply/step` echoes the `seq` it was asked by. That is the address, the
//! answer's own field, and the thing the paint keys on — so nothing here
//! remembers what was asked, and a reply that arrives after the operator
//! clicked another row cannot paint under the wrong one.

use super::Model;

impl Model {
    /// **Ask for one step's records**, or do nothing where there is no wall or
    /// no conversation to address the question to — the two states the pane
    /// cannot paint the control in.
    ///
    /// A read rather than an act ([`super::Posted::read`]): asking twice is
    /// asking once, so a lost reply needs no recovery beyond clicking again.
    pub fn ask_step(&mut self, seq: &str) {
        let (Some(aim), Some(conversation)) = (self.aim.clone(), self.conversation.clone()) else {
            return;
        };
        self.outbox.push(super::Posted::read(crate::verbs::step(
            aim.address,
            conversation,
            seq.to_owned(),
        )));
    }

    /// **The records of `seq`, if that is the step the last answer was
    /// about.** The paint asks per row, so the drill-in appears under the row
    /// it belongs to and under no other.
    pub fn drilled_into(&self, seq: &str) -> Option<crate::reply::step::Step> {
        self.records.drilled.clone().filter(|step| step.seq == seq)
    }
}

#[cfg(test)]
mod tests;
