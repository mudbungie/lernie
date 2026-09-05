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
//! Aiming at another wall retires the conversation list, what that list was an
//! answer to, the selection and everything under it, because none of it is
//! about the new wall. Retiring the *answer* beside the rows is what lets the
//! pane say it has not heard yet rather than reporting an empty wall. Selecting a
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
        self.answered = None;
        self.conversation = None;
        self.transcript = crate::reply::transcript::Transcript::default();
        self.live = None;
        // **The tuning pane goes with the wall it was opened on** (bl-4a2c).
        // It holds no aim of its own, deliberately — every gesture it composes
        // reads `Model::aim` at the moment of composing — so leaving it open
        // over a new wall would leave an operator looking at one wall's rows
        // while its controls wrote to another's. Retiring the ROWS beside it is
        // the same reading `answered` above gets: `None` is a wall nobody has
        // asked yet, and the old wall's answer is not this wall's.
        self.tuning = None;
        self.roles = None;
        // **The login pane goes with the wall too** (bl-e3c5), and for the
        // sharper form of the same reason: its act signs a credential into ONE
        // wall's own store, so a pane left standing over a new aim would offer
        // to sign the operator in somewhere they are no longer looking.
        self.retire_login();
        self.retire_records();
    }

    /// **Select a conversation**, by the id every gesture addresses it with.
    pub fn select(&mut self, root_id: &str) {
        self.conversation = Some(root_id.to_owned());
        self.transcript = crate::reply::transcript::Transcript::default();
        self.live = None;
        // **The records pane goes with the conversation it was opened on**
        // (bl-2cf7) — the same rule the tuning pane keeps for its wall, one
        // noun over (`super::records`).
        self.retire_records();
    }

    /// **Assert this wall's place in the strip** — pinned, or not (bl-7782).
    ///
    /// `pinned` is what the operator is asking FOR, not what the row currently
    /// is: the wire's two ops are assertions rather than a toggle, and upstream
    /// says why — *"unpinning one that is not pinned leaves the list alone,
    /// which is what lets two seats send it at once and agree."* So this end
    /// composes the act it means and never flips a flag it read a beat ago.
    ///
    /// The aim is the gate, exactly as it is for every other per-wall act, and
    /// what says it landed is the next roster answer: both ops reply with the
    /// listing carrying the ranks it now has, which is a kind this seat already
    /// paints.
    pub fn post_pin(&mut self, pinned: bool) {
        let Some(Aim { address, .. }) = self.aim.clone() else {
            return;
        };
        let gesture = if pinned {
            crate::verbs::pin(address)
        } else {
            crate::verbs::unpin(address)
        };
        self.outbox.push(super::Posted::act(gesture));
    }

    /// **Put the notice down.** An operator who has read a refusal should not
    /// have to wait for the next answer to clear it.
    pub fn dismiss(&mut self) {
        self.notice = None;
    }

    /// **What Escape means**, in the order an operator means it (bl-7574,
    /// extended for the tuning pane in bl-4a2c).
    ///
    /// A ladder from the innermost thing on the glass outwards, which is the
    /// order the key is actually reached for. The enrollment covers the window
    /// and holds a secret, so it goes first — and closing it is the same act
    /// `done — forget it` performs, material and all, because the control that
    /// closes that pane is the control that forgets. Then a draft assignment,
    /// which is a thing inside a pane rather than the pane: Escape over a
    /// half-typed model puts the draft down and leaves the rows standing.
    /// Then the tuning pane itself, then the records pane (bl-2cf7), then the
    /// decision queue (bl-f0ef), then whichever of the window's own two is
    /// standing (bl-40ec — one arm, because one field holds both), then the
    /// login pane (bl-e3c5), then an unmaking (bl-48fa) — no two of the seven
    /// ever stand together, so the order among them is never spent. The
    /// unmaking is on the ladder because Escape over a destructive pane means
    /// what [`Model::close_unmaking`] means and nothing else: it unmakes
    /// nothing, and a key that could arm or spend one would be the second
    /// surface `crate::ui::keys` refuses to be. With nothing covering, the notice
    /// is the only thing left to put down, and Escape is its × reached without
    /// a pointer.
    pub fn escape(&mut self) {
        if self.enroll.is_some() {
            self.close_enrollment();
        } else if matches!(self.tuning, Some(super::Tuning::Editing(_))) {
            self.cancel_assignment();
        } else if self.tuning.is_some() {
            self.close_tuning();
        } else if self.records {
            self.close_records();
        } else if self.queue {
            self.close_queue();
        } else if self.lookup.is_some() {
            self.close_lookup();
        } else if self.login.is_some() {
            self.close_login();
        } else if self.unmaking.is_some() {
            self.close_unmaking();
        } else {
            self.dismiss();
        }
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
        self.outbox.push(super::Posted::act(gesture));
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
