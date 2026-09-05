//! **The one door a reply comes in through** — and the one leg that brought no
//! reply at all.
//!
//! Split from [`super`] at the 300-line cap on the seam that module's own doc
//! already draws: [`super`] is *what the window holds between frames*, and this
//! is *how what arrives becomes part of it*. The first changes when a pane
//! learns to hold something; the second when a kind lands.
//!
//! **Nothing that arrives is dropped.** An answer is filed, and a refusal or an
//! unreadable frame becomes the [`Notice`](super::Notice) the shell paints
//! *where that content would have been* — `crate::reply`'s rung 2 honoured on
//! the glass rather than only in the type. The two read differently on purpose:
//! a refusal is the engine's sentence and an unreadable frame is a statement
//! about this seat, of which only the second is fixed by an upgrade.
//!
//! **And a leg that reached no engine is neither.** It is not a frame and does
//! not come through [`Model::absorb`]; it is a fact about a *relationship*
//! rather than about an exchange, so it is said on that channel's own section
//! (REMOTE §8.2, bl-e620).

use super::{Channel, Chunk, Held, Model, Notice};
use crate::reply::{Read, Reply};

impl Model {
    /// **Take one reply frame.** The single door: an answer is filed, and
    /// anything else becomes the notice the shell paints where that answer's
    /// content would have been. `channel` is the client-side stamp, applied
    /// here and nowhere else.
    pub fn absorb(&mut self, channel: &Channel, read: Read) {
        match read {
            Read::Answer(reply) => self.file(channel, reply),
            Read::Refusal(said) => self.notice = Some(Notice::Refused(said)),
            Read::Unreadable(why) => self.notice = Some(Notice::Unreadable(why)),
        }
    }

    /// **Take one act's receipt**, knowing which act it answers (bl-b180).
    ///
    /// An answer goes through [`absorb`](Self::absorb) like any frame. What
    /// differs is the two readings that are *not* an answer, and only for the
    /// start's own two acts, which are the one gesture this window holds
    /// across a round trip: the engine's refusal retires the start into its
    /// own sentence with the goal back in the box, and a frame this seat cannot
    /// read retires it into the bar's sentence the same way — because a start
    /// nothing will ever answer is a composer with no box. Every other op's
    /// refusal is exactly what it was: the bar.
    pub fn receipt(&mut self, channel: &Channel, op: &str, read: Read) {
        match read {
            Read::Refusal(said) if self.starting(op) => self.refuse_start(said),
            Read::Unreadable(_) if self.starting(op) => {
                self.take_back_start();
                self.absorb(channel, read);
            }
            other => self.absorb(channel, other),
        }
    }

    /// File one answer. A roster answer replaces its **own channel's** chunk
    /// and leaves every other one standing, which is REMOTE §8.2's *"a refusal
    /// is one entry's, never the set's"* read from the other side: a box
    /// serving three engines does not lose the two that are fine.
    fn file(&mut self, channel: &Channel, reply: Reply) {
        match reply {
            Reply::Workspaces(view) => self.seat(channel, view),
            // The one answer a claim can be spent against: a listing is where
            // the started conversation first becomes addressable
            // ([`Model::resolve`]).
            Reply::Conversations(rows) => {
                self.convs = rows;
                self.answered = self.aim.clone();
                self.resolve();
            }
            // Filed whether or not the pane is open: the read stands only while
            // it is, so a frame after it closed is the last one in flight.
            Reply::Roles(rows) => self.roles = Some(rows),
            // The clients pane's one read, on the roles' own terms.
            Reply::Clients(rows) => self.machines = Some(rows),
            // **The queue, one channel's slice at a time** — the fan's answer
            // replaces what this channel last said and leaves the others
            // standing, exactly as a roster answer does (`queue`).
            Reply::Attention(rows) => self.asking(channel, rows),
            // The records pair, on the same terms as the roles above.
            Reply::Steps(listing) => self.steps = Some(listing),
            Reply::Files(answer) => self.files = Some(answer),
            // The spine pair, on the same terms again — the records pane's
            // second two reads, standing while it is open (`spine`).
            Reply::Rail(spine) => self.rail = Some(spine),
            Reply::Governing(config) => self.governing = Some(config),
            // **The window's own two, one channel's slice at a time** — each
            // op names no workspace, so each answer replaces what its channel
            // last said and leaves the others standing, exactly as a roster
            // answer and a queue answer do (`window`; bl-40ec).
            Reply::Help(rows) => self.paged(channel, rows),
            // The trail, on the same terms — every action that crossed ONE
            // engine's boundary, replacing that channel's section (`trail`).
            Reply::Ops(rows) => self.crossed(channel, rows),
            Reply::Found(found) => self.hit(channel, found),
            // The login pane's three, on the roles' own terms — filed whether
            // or not the pane is open, because a frame that arrives after it
            // closed is the last one in flight rather than a thing to drop.
            // The table's rows and one row's offering are plain answers; the
            // sign-in run replaces, exactly as the live tail does, because the
            // lane hands over the whole fold (`crate::offframe::signin`).
            Reply::Providers(rows) => self.providers = Some(rows),
            Reply::Models(rows) => self.offered = Some(rows),
            Reply::Login(run) => self.signin = Some(run),
            Reply::Transcript(transcript) => self.transcript = transcript,
            Reply::Follow(stream) => self.live = Some(stream),
            // The start family's two, whose whole product is each other: the
            // staged body composes the fire, and the fire's receipt is the
            // minted name. [`Start`] holds the chain.
            // The one answer that is never filed as content: it is drawn, held
            // while the picture is on screen, and dropped with the pane.
            Reply::Enrolled(material) => self.enrolled(&material),
            Reply::Prepared(prepared) => self.fire(&prepared),
            Reply::Started { conversation } => self.started(conversation),
            // The three receipts. None carries content, so what they change is
            // whether the operator is told something happened — and a captured
            // run that failed is told in the child's own words. The advance and
            // the raise are ONE arm rather than two identical ones, which is
            // the honest shape: what each changed arrives on the transcript and
            // on the next queue respectively, and this end predicts neither.
            Reply::Nudged | Reply::Flagged => self.notice = None,
            Reply::Outcome(outcome) => {
                self.notice = (!outcome.ok()).then_some(Notice::Refused(outcome.stderr));
            }
        }
    }

    /// Seat one channel's roster answer. **The channel comes in with the
    /// answer** rather than being looked up: what a channel is called here and
    /// what it is called on its host are facts the asker holds, and a model
    /// that re-derived them would be a second authority for them.
    fn seat(&mut self, channel: &Channel, view: crate::reply::roster::Workspaces) {
        let seated = Chunk {
            channel: channel.clone(),
            // **The answer spends whatever the section was standing on**: a
            // channel that has answered is neither unheard nor unheld, and an
            // engine that answered zero workspaces holds zero — which is a
            // different sentence from either (bl-08b6).
            held: Held::Heard,
            stale: view.stale,
            growth: view.growth,
            walls: view.rows,
        };
        match self
            .roster
            .iter_mut()
            .find(|chunk| chunk.channel.name == channel.name)
        {
            Some(held) => *held = seated,
            None => self.roster.push(seated),
        }
    }

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
    pub fn acted(&mut self, op: &str, reach: &crate::channel::Reach) {
        if self.starting(op) {
            self.take_back_start();
        }
        self.notice = Some(Notice::act(op, reach));
    }
}
