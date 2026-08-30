//! **What a control does, whichever control did it.**
//!
//! Three acts the window affords beyond composing a gesture: aim at a wall,
//! select a conversation, put a notice down. Each used to live inside the
//! `if …clicked()` that made it, which is fine while a pointer is the only way
//! to make it — and stops being fine the moment a key can make it too. Two
//! spellings of *what aiming means* would drift, and the one that drifted would
//! be the one nobody points at.
//!
//! So a binding names a control that already exists (`crate::ui::keys`), by
//! calling the same door the click calls. **A binding that could fire something
//! a click cannot is a second surface**, which is why the keyboard's own cursor
//! walks only rows the pointer can also reach: an unaddressable roster row is
//! skipped by both, structurally, because both ask the same question.
//!
//! # Each act clears exactly what it invalidated
//!
//! Aiming at another wall retires the conversation list, the selection and
//! everything under it, because none of it is about the new wall. Selecting a
//! conversation retires the transcript and the live tail, because they are the
//! old one's. Nothing else is touched — and in particular the draft is not,
//! because what an operator typed is theirs until they send it.

use super::{Aim, Model};

impl Model {
    /// **Aim at a wall**, down the channel that resolved it.
    ///
    /// `address` is what a gesture must carry rather than the name the row
    /// wears; the two differ exactly where an entry renames, and the roster is
    /// what maps one to the other.
    pub fn aim_at(&mut self, channel: &str, address: &str) {
        self.aim = Some(Aim {
            channel: channel.to_owned(),
            address: address.to_owned(),
        });
        self.convs.clear();
        self.conversation = None;
        self.transcript = crate::reply::transcript::Transcript::default();
        self.live = None;
    }

    /// **Select a conversation**, by the id every gesture addresses it with.
    pub fn select(&mut self, root_id: &str) {
        self.conversation = Some(root_id.to_owned());
        self.transcript = crate::reply::transcript::Transcript::default();
        self.live = None;
    }

    /// **Put the notice down.** An operator who has read a refusal should not
    /// have to wait for the next answer to clear it.
    pub fn dismiss(&mut self) {
        self.notice = None;
    }
}

#[cfg(test)]
mod tests;
