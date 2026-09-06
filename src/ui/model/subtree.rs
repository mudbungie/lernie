//! **What hangs UNDER a conversation, and whether it is on the glass**
//! (bl-00f5).
//!
//! # The strip is the index of what is happening, and two thirds of it was
//! bookkeeping
//!
//! One session of ordinary work answered twelve conversations, of which five
//! were compactors the operator never started — each with a minted name as
//! memorable as the work's, each at the same weight, four of the five
//! belonging to ONE conversation. The rows were not wrong: `depth` is 1 and
//! the preview says what they are. But an index whose majority is machinery is
//! an index people learn to stop reading.
//!
//! # The rule is about DESCENT and not about compactors
//!
//! **The wire carries no kind.** A conversation row spells its address, its
//! badge, its preview and how far it hangs under its root; nothing on it says
//! *this one is a compactor*, and nothing should — a fork's candidate is a
//! depth-1 row the operator DID start, and a seat that guessed from a preview
//! would be reading prose to make a structural decision.
//!
//! What the engine does give is the descent, and the descent is the whole
//! answer: **the list is the conversations, and what hangs under one is under
//! it.** A root is always on the glass; its subtree is one gesture away and
//! says how much of it there is. That satisfies the ball's own first remedy —
//! *compactors fold under the conversation they compact* — without a rule
//! about compactors, and it folds a fork's candidates on the same terms, which
//! is right for the same reason: both are what this conversation is made of.
//!
//! # It is ONE list, so the fold is here rather than in the pane
//!
//! [`Model::rows`] is what the pointer paints and the keyboard walks
//! (`super::claim`), so a fold applied in the pane would be a row a click
//! could not reach and a key could — which is the second surface
//! `crate::ui::keys` exists in order not to have. Filtering the one list keeps
//! both honest for free.
//!
//! **What the attention rows say is not this seat's to fix.** Every compactor
//! in that session earned an `attention` row saying *came to rest — your
//! turn*, which is false of a finished compaction; that is a fact the engine
//! answers and yog's own ball owns it. This seat's half is where the rows are
//! painted.

use crate::reply::convs::ConvRow;
use crate::ui::Model;

impl Model {
    /// **`rows` with every unopened subtree left out.**
    ///
    /// A pure function of the ordered rows and what the operator has opened.
    /// The order is the engine's — a root followed by its descendants, deepest
    /// last — so one pass with a cut depth is the whole of it: below an
    /// unopened row, everything deeper than that row belongs to it.
    pub(super) fn unfolded(&self, rows: Vec<ConvRow>) -> Vec<ConvRow> {
        let mut out = Vec::new();
        let mut cut: Option<u64> = None;
        for row in rows {
            if cut.is_some_and(|at| row.depth > at) {
                continue;
            }
            cut = (!self.opened.contains(&row.root_id)).then_some(row.depth);
            out.push(row);
        }
        out
    }

    /// **Open or close what hangs under one conversation.**
    ///
    /// A set of what is OPEN rather than of what is folded, so the default is
    /// the answer with no state at all: a seat that has never been told
    /// otherwise shows the conversations and not their machinery.
    pub fn toggle_subtree(&mut self, root: &str) {
        if !self.opened.remove(root) {
            self.opened.insert(root.to_owned());
        }
    }

    /// **Whether this conversation's subtree is on the glass.**
    pub fn subtree_open(&self, root: &str) -> bool {
        self.opened.contains(root)
    }
}

#[cfg(test)]
mod tests;
