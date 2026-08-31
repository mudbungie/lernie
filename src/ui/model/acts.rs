//! **What a control does, whichever control did it.**
//!
//! The acts the window affords beyond composing a gesture: aim at a wall,
//! select a conversation, put a notice down, and the enrollment's four — open
//! it, spend it, file what it answered, close it and forget. Each used to live
//! inside the
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
//!
//! The enrollment's own act is the sharpest case of that rule: closing it
//! clears the material and nothing else, and clearing the material IS the
//! product — after it there is no copy of that key on this box.

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

    /// **Open an enrollment aimed at the wall the window is pointed at**, or do
    /// nothing where it is pointed at none.
    ///
    /// The aim is the gate rather than a separate check: the act mints a
    /// registration, a registration is the pair `(client, workspace)`, and a
    /// workspace is exactly what an aim is. So the control is offered where
    /// there is one and nowhere else.
    pub fn begin_enrollment(&mut self) {
        if let Some(aim) = self.aim.clone() {
            self.enroll = Some(super::Enrolling::at(aim));
        }
    }

    /// **Spend the enrollment**: compose the gesture for whoever can send it.
    ///
    /// It composes and does not clear, because the pane has to stand until the
    /// answer arrives — there is nothing else on the screen that would tell an
    /// operator the act is in flight.
    pub fn post_enrollment(&mut self) {
        let Some(enrolling) = self.enroll.as_mut().filter(|held| held.ready()) else {
            return;
        };
        enrolling.posted = true;
        let gesture = enrolling.gesture();
        self.outbox.push(gesture);
    }

    /// **Close the enrollment, and drop the material with it.** The one control
    /// whose whole product is a forgetting: after it there is no copy of that
    /// key anywhere on this box, which is the property DESIGN §3 states and
    /// `crate::seat::enroll`'s tests assert over the tree.
    pub fn close_enrollment(&mut self) {
        self.enroll = None;
    }

    /// **File the material** — draw it, and hold the picture beside it.
    ///
    /// Two arms and the second is a real one. Where the pane is still open the
    /// symbol is drawn once, here, because a frame may not compute one. Where
    /// it is **not** — the operator closed it while the act was in flight — the
    /// material has arrived with nowhere to go, and it is dropped *and said so*:
    /// the enrollment did happen on the engine, and an operator who is not told
    /// would go looking for a box that has a registration and no material. A
    /// silent drop is the one thing this model's door does not do.
    pub(super) fn enrolled(&mut self, material: &crate::reply::enrolled::Enrolled) {
        let Some(enrolling) = self.enroll.as_mut() else {
            self.notice = Some(super::Notice::Unreadable(format!(
                "the engine enrolled {} while this pane was closed, so the \
                 material is gone — enroll again to mint another",
                material.caption()
            )));
            return;
        };
        match super::Shown::of(material) {
            Ok(shown) => {
                enrolling.shown = Some(shown);
                self.notice = None;
            }
            Err(why) => {
                self.enroll = None;
                self.notice = Some(super::Notice::Unreadable(why));
            }
        }
    }
}

#[cfg(test)]
mod tests;
