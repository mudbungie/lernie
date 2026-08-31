//! **The claim a start leaves on the selection**, the row it stands in for, and
//! the answer that spends it.
//!
//! Upstream's rule is *a start focuses what it started* (yog's `docs/DESIGN.md`
//! §3.4), and honouring it directly does not work: the minted name is a barrier
//! — every gesture after the receipt may address it — but only once the
//! detached driver has written the conversation's branch, and until then the
//! engine resolves it nowhere. A seat that simply selected it would publish the
//! standing set's third question against an address the engine refuses, and
//! paint the operator's own new conversation as unknown for the whole of a
//! healthy start.
//!
//! So the selection is taken and three things ride with it. None of them is a
//! second pending concept: the claim **is** [`Start`](super::Start) in its
//! [`Started`](super::Phase::Started) phase, read against what is selected, and
//! [`Model::pending`] is that reading — the one row the claim amounts to.
//!
//! - **Nothing is asked about it.** [`Model::asked`] is what
//!   `crate::state::Standing` publishes as its third question, and a claimed
//!   name is not it. An empty conversation pane is what the world honestly
//!   holds; a refusal painted there would be this seat asking a question it
//!   knew the answer to.
//! - **A row stands where the conversation will be.** [`Model::rows`] is the
//!   list every surface paints and walks, so the row is one row in one list
//!   rather than a case each of them carries. It is what nothing observed: no
//!   lock, no completed step, flagged uncertain — exactly what the engine's own
//!   classifier answers for a conversation it cannot probe. **It must not read
//!   `live`**, which would claim a driver this seat has never seen.
//! - **It is spent where it was made.** [`Model::resolve`] retires the claim
//!   the moment the engine answers a row under that name, and moves the
//!   selection **only while the selection is still the name the claim put
//!   there**. A start can take a minute to write its branch, and an operator
//!   who read something else in that minute must not be yanked back by their
//!   own conversation arriving.
//!
//! **A claim whose row never arrives is inert**, which is what a start whose
//! driver died honestly is: the name stays selected, the row stays faded, and
//! one arrow key leaves it.

use super::start::Phase;
use crate::reply::convs::{AgentState, ConvRow, Tone};
use crate::ui::Model;

impl Model {
    /// **The conversation this window started and the engine cannot yet
    /// resolve**, as the row that stands for it.
    ///
    /// The comparison against the selection is the whole predicate. Aiming
    /// elsewhere clears the selection and selecting another conversation
    /// replaces it, so a claim that is no longer where it was made answers
    /// `None` here with nothing having had to retire it.
    ///
    /// **`name` is absent on the row** for the reason the vocabulary gives that
    /// field: it is what a later act may *address*, and this is the one
    /// conversation the engine will not answer to yet. Withholding it is also
    /// what keeps [`resolve`](Self::resolve) from matching this row against the
    /// claim it was built from.
    pub fn pending(&self) -> Option<ConvRow> {
        let start = self.start.as_ref()?;
        let Phase::Started(name) = &start.phase else {
            return None;
        };
        if self.conversation.as_deref() != Some(name.as_str()) {
            return None;
        }
        Some(ConvRow {
            root_id: name.clone(),
            display: name.clone(),
            name: None,
            state: AgentState::Quiescent,
            uncertain: true,
            preview: start.goal.clone(),
            age_secs: 0,
            attention: 0,
            members: 1,
            depth: 0,
            tone: Tone::Weak,
        })
    }

    /// **The conversation the standing set may ask about.** A claimed name
    /// resolves nowhere, so there is no transcript to ask for and no tail to
    /// hold open against it.
    pub fn asked(&self) -> Option<String> {
        match self.pending() {
            Some(_) => None,
            None => self.conversation.clone(),
        }
    }

    /// **The conversation list as it is painted and walked** — what the engine
    /// answered, and the claim's row before the engine has anything to answer.
    ///
    /// One list, because the pointer and the keyboard both read it: a row a
    /// click could reach and a key could not is the second surface
    /// `crate::ui::keys` exists in order not to have.
    pub fn rows(&self) -> Vec<ConvRow> {
        self.pending()
            .into_iter()
            .chain(self.convs.iter().cloned())
            .collect()
    }

    /// **Spend the claim against what the engine answered.**
    ///
    /// The name is the join: a row hands back its *addressable* name exactly
    /// when a stored fact backs it, which is the same moment the conversation
    /// became addressable at all.
    pub(super) fn resolve(&mut self) {
        let Some(start) = self.start.as_ref() else {
            return;
        };
        let Phase::Started(name) = &start.phase else {
            return;
        };
        let Some(row) = self
            .convs
            .iter()
            .find(|row| row.name.as_deref() == Some(name.as_str()))
        else {
            return;
        };
        let root = row.root_id.clone();
        let held = self.conversation.as_deref() == Some(name.as_str());
        self.start = None;
        if held {
            self.select(&root);
        }
    }
}

#[cfg(test)]
mod tests;
