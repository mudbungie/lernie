//! **The box, and the two readings it takes against the engine's answer**
//! (bl-4855; DESIGN §4.30).
//!
//! Split from [`super`] at the design-time budget on the seam the two already
//! have: [`super`] is which file the pane is pointed at and what it composes,
//! and this is the one value that has to hold a conversation with the engine's
//! answer — seeded from it, settled against it, and answering the two
//! questions the controls are enabled and worded by.
//!
//! The whole of it exists because upstream's hash guard does not cross the
//! wire. A `config` act *"carries no hash guard, and needs none"* (REMOTE
//! §9.18) because a gesture states its whole text in one atomic instruction —
//! which is true of the gesture and false of the operator typing it. This is
//! that guard restated as a reading rather than as a refusal.

/// **A config file being edited**: the bytes in the box, and the bytes the box
/// last agreed with the engine about.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Draft {
    /// **What the box and the engine last agreed on.** Not a copy of either —
    /// it is the anchor the two readings below are taken against, and it is
    /// the only way to tell *the operator has unsent edits* from *somebody
    /// else wrote this file*.
    pub seed: String,
    /// What is in the box.
    pub text: String,
}

impl Draft {
    /// A box opened on the file, agreeing with it.
    pub fn of(answered: &str) -> Self {
        Self {
            seed: answered.to_owned(),
            text: answered.to_owned(),
        }
    }

    /// **Re-anchor where the box and the file say the same thing**, which is
    /// the whole of what keeps the two readings below honest across a write.
    ///
    /// Called on every frame that paints the box. It moves nothing an operator
    /// typed — the branch is *the text already equals the answer* — and it is
    /// what makes a write the seat itself sent stop reading as a file that
    /// moved: the engine answers the bytes we asserted, the box already holds
    /// them, and the anchor catches up in the same frame.
    pub fn settle(&mut self, answered: &str) {
        if self.text == answered {
            answered.clone_into(&mut self.seed);
        }
    }

    /// **Whether there is anything to write** — the enablement on the control,
    /// and it is against the ENGINE's answer rather than the anchor: writing
    /// the bytes the file already holds is a round trip that changes nothing.
    pub fn unwritten(&self, answered: &str) -> bool {
        self.text != answered
    }

    /// **Whether the file went somewhere neither end of this box put it.**
    ///
    /// The answer differs from the box AND from what the two last agreed on,
    /// which leaves exactly one reading: another writer. Our own write in
    /// flight is excluded by the first clause (the answer is still the old
    /// bytes, which are the anchor) and our own write landed by the second
    /// (the answer is the box).
    pub fn moved(&self, answered: &str) -> bool {
        self.unwritten(answered) && answered != self.seed
    }
}

#[cfg(test)]
mod tests;
