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
//! **The wire carries no kind, and since yog bl-009a it carries no machinery
//! either** (bl-b538; ruling lernie bl-454d): an agent a workflow binding
//! forked — titler, compactor, verifier — is read off its dispatch commit's
//! `Procedure:` trailer on the server and leaves the conversations answer with
//! its descent, counted in no `members` and no `direct`. Nothing on a row says
//! *this one is a compactor* because no row is one. A seat that guessed from a
//! preview would be reading prose to make a structural decision, and now has
//! no reason to.
//!
//! What the server does give is the descent of what remains — the subagents a
//! model dispatched, a fork's candidates — and the descent is the whole
//! answer: **the list is the conversations, and what hangs under one is under
//! it.** A root is always on the glass; its subtree is one gesture away and
//! says how much of it there is. Those rows are what this conversation is made
//! of, and the operator may well open them, so the fold stays exactly as it
//! was: its mechanism unchanged, its subject shrunk to conversations.
//!
//! # It is ONE list, so the fold is here rather than in the pane
//!
//! [`Model::rows`] is what the pointer paints and the keyboard walks
//! (`super::claim`), so a fold applied in the pane would be a row a click
//! could not reach and a key could — which is the second surface
//! `crate::ui::keys` exists in order not to have. Filtering the one list keeps
//! both honest for free.
//!
//! **What the attention rows said was not this seat's to fix**, and the server
//! fixed it (yog bl-3592: a rest is the operator's only when nobody dispatched
//! it). This seat's half is where the rows are painted.

use crate::reply::convs::ConvRow;
use crate::ui::Model;

impl Model {
    /// **`rows` with every unopened subtree left out.**
    ///
    /// A pure function of the ordered rows and what the operator has opened.
    /// The order is the server's — a root followed by its descendants, deepest
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
