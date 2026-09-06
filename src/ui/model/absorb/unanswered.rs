//! **The leg that brought no reply at all** — a channel this seat could not
//! reach, and an act that earned no answer.
//!
//! Split from [`super`] at the 300-line cap on the seam that module's own doc
//! draws in its third paragraph: what arrives is a frame, and what does not
//! arrive is a fact about a **relationship** or about an **exchange**. The two
//! are not the same sentence and are not painted in the same place, and they
//! change for different reasons — a reply kind lands there, a failure
//! vocabulary lands here.

use super::{Channel, Held, Model, Notice};

impl Model {
    /// **A leg that never reached an engine**, said on that channel's own
    /// section (bl-e620).
    ///
    /// It is not a reply and so it does not come through
    /// [`absorb`](Self::absorb): there is no frame, no channel answered, and
    /// nothing to file. And it is not the shell's bar either, which is where it
    /// used to go. **A refusal is an exchange; an unreachable channel is a
    /// relationship**, and REMOTE §8.2 rules that one *"is that channel's
    /// workspaces painted unreachable, never the whole shell, which stays
    /// reserved for the one wire the window cannot exist without."* Three
    /// things followed from getting that wrong, all driven live: the sentence
    /// named no subject at all (*"this seat could not reach **it**"*), a seat
    /// holding two dead channels heard about exactly one of them forever
    /// because there is one bar and the last writer wins, and the bar's dismiss
    /// was inert — a relationship that is down is down on every beat, so it
    /// re-posted faster than a hand can clear it. A row's state is not
    /// something one dismisses.
    ///
    /// The bar is kept for a channel this box holds no section for, which is
    /// the one case with nowhere else to say it — and it names the channel,
    /// because a fact with no home still has a subject.
    ///
    /// **An ACT that failed is not this** — see [`Model::acted`], which is that
    /// same division applied rather than bent.
    pub fn unreachable(&mut self, channel: &Channel, why: String) {
        match self
            .roster
            .iter_mut()
            .find(|chunk| chunk.channel.name == channel.name)
        {
            Some(held) => held.held = Held::Unheld(why),
            None => self.notice = Some(Notice::Unreachable(format!("{}: {why}", channel.name))),
        }
    }

    /// **An act that earned no reply** (yog's `docs/REMOTE.md` §3, bl-3969).
    ///
    /// It goes to the bar and takes no channel, which is
    /// [`unreachable`](Self::unreachable)'s own rule read the other way: *a
    /// refusal is an exchange; an unreachable channel is a relationship*, and a
    /// gesture an operator made is an exchange whatever went wrong with it. Two
    /// further things follow and both are the section's reasons inverted. The
    /// section is the slot a `workspaces` answer overwrites (`Model::seat` sets
    /// `Held::Heard` on every one), and the asker answers `workspaces` on every
    /// beat — so an act's sentence written there is erased within one beat by a
    /// read that succeeded, which is exactly the wrong outcome for the one fact
    /// on this window that nothing will say again. And the bar's dismiss is
    /// live here rather than inert: an act is an event, not a state, so it does
    /// not re-post on a beat, and *I have looked* is a real thing for an
    /// operator to say about one.
    ///
    /// **The recovery is on the glass already.** The contract's answer to a
    /// lost reply is a read, and this window's reads never stopped — the
    /// standing set is re-derived from the focus and asked every beat, so the
    /// conversation, the roster and whichever pane is open are being asked again
    /// while the sentence stands. That is why there is no control here and no
    /// mapping from an act to *its* read: a control would be a second spelling
    /// of the beat, and the parity roster judges controls, not states (§4.16).
    ///
    /// **A start whose act this was is taken back** (bl-b180): the bar says
    /// what happened to the act, and the box comes back with the goal in it —
    /// held forever it would be a composer with no box, and IN DOUBT's own
    /// remedy is to look, which an operator does with the goal in front of
    /// them rather than behind a sentence that never moves.
    ///
    /// **And an act that never left this box gives its words back** (bl-e85f).
    /// The notice for one says, in the seat's own sentence, *nothing happened —
    /// it is safe to do it again*, and the seat then threw away the one thing
    /// doing it again needs: the composer was empty and showing its
    /// placeholder, so *again* meant retyping a paragraph-long goal. The refund
    /// is `Model::refund`'s, which is the start's own rule (bl-b180) — words go
    /// back only into a box that is empty, so a draft typed since is never
    /// clobbered.
    ///
    /// **Only where nothing crossed**, which is the whole of the asymmetry. IN
    /// DOUBT means the engine had the gesture and may have run it, and its
    /// sentence says so — putting the words back under a box with `send` beside
    /// it would invite exactly the resend the contract forbids (REMOTE §3).
    pub fn acted(&mut self, op: &str, reach: &crate::channel::Reach, said: Option<String>) {
        if self.starting(op) {
            self.take_back_start();
        }
        if let Some(words) = said.filter(|_| !reach.crossed()) {
            self.refund(&words);
        }
        self.notice = Some(Notice::act(op, reach));
    }
}
