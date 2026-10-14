//! **A selection the engine no longer knows** (bl-6dfd).
//!
//! The selected conversation's own row is asked every beat
//! (`crate::offframe::asker::wall::selected`). When the engine refuses that
//! read — the conversation was deleted from another seat, or a row was aimed
//! at the wrong engine — it is the engine saying the subject is gone, and a
//! selection held past that is one whose refusal re-posts every beat faster
//! than Esc can put it down. So the refusal drops the selection back to the
//! wall, where the box begins a conversation, and is posted once: with
//! nothing selected nothing asks again.

use super::{Channel, Model};
use crate::reply::Read;

impl Model {
    /// **Take one read about the selected conversation**, knowing which op
    /// asked it. A refused `agent` read retires the selection before the
    /// refusal is filed as the bar's sentence; everything else is
    /// [`Model::absorb`]'s.
    pub fn about(&mut self, channel: &Channel, op: &str, read: Read) {
        if op == crate::verbs::AGENT.word && matches!(read, Read::Refusal(_)) {
            // A selection the engine put down is a tab put down (§4.44).
            if let Some(id) = self.conversation.clone() {
                self.shut(&id);
            }
            self.deselect();
        }
        self.absorb(channel, read);
    }

    /// **Nothing selected**, the wall still aimed: the transcript, its tail
    /// and the records pane go with the conversation they were about.
    pub(super) fn deselect(&mut self) {
        self.conversation = None;
        self.transcript = crate::reply::transcript::Transcript::default();
        self.live = None;
        self.retire_records();
    }
}

#[cfg(test)]
mod tests;
