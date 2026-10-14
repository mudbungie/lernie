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
//! **And a leg that reached no engine is neither** — [`unanswered`], split off
//! at the 300-line cap on exactly the seam this doc's own third paragraph
//! already draws. What arrives is a frame; what does not arrive is a fact
//! about a relationship or about an exchange, and the two halves change for
//! different reasons: a reply kind lands here, a failure vocabulary lands
//! there.

/// The leg that brought no reply at all: a channel this seat could not reach,
/// and an act that earned no answer.
mod unanswered;

use super::{Channel, Chunk, Held, Model, Notice};
use crate::reply::{Read, Reply};

impl Model {
    /// **Take one reply frame.** The single door: an answer is filed, and
    /// anything else becomes the notice the shell paints where that answer's
    /// content would have been. `channel` is the client-side stamp, applied
    /// here and nowhere else.
    pub fn absorb(&mut self, channel: &Channel, read: Read) {
        match read {
            Read::Answer(reply) => {
                self.landing(&reply);
                self.file(channel, reply);
            }
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
        self.founded(op);
        self.refused(op, &read);
        match read {
            // **The one kind whose meaning is the op and not the reply**
            // (bl-a43a): `fleet`, `disband`, `arm` and `disarm` all answer
            // `armed`, and the two families they span are the fleet loop and
            // the alignment monitor. The poster still knows which was sent, so
            // the join happens here rather than by the pane guessing.
            Read::Answer(Reply::Armed(on)) => self.armed(op, on),
            // **A deposit's receipt is the transcript** (bl-9760): the words
            // the operator sent land in the pane they are looking at, so the
            // run's `ok` would only cover the conversation's own state in the
            // slot after every send. A role write's is the rule's own segment
            // (bl-a7f5): the roles read repaints it on the next beat. Every
            // other act's receipt is the bar's.
            Read::Answer(Reply::Outcome(ref run)) if run.ok() && witnessed(op) => {
                self.notice = None;
            }
            // **The roster's refusal is the channel's, not the bar's**
            // (bl-693d): it re-posts every beat while it holds, and it stops
            // holding the moment a founding registers this client.
            Read::Refusal(said) if op == crate::verbs::WORKSPACES.word => {
                self.unseated(channel, said);
            }
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
                self.prune_tabs();
            }
            // Filed whether or not the pane is open: the read stands only while
            // it is, so a frame after it closed is the last one in flight.
            Reply::Roles(rows) => self.roles = Some(rows),
            // The clients pane's one read, on the roles' own terms.
            Reply::Clients(rows) => self.machines = Some(rows),
            // The config pane's two, on the same terms.
            Reply::Config(file) => self.config = Some(file),
            Reply::Lineages(rows) => self.lineages = Some(rows),
            // **And the learning loop's read, on the same terms** (bl-a1d6):
            // the listing and — at the deeper depth — one proposal whole, filed
            // as one answer because it is one.
            Reply::Proposals(staged) => self.proposals = Some(staged),
            // **The queue, one channel's slice at a time** — the fan's answer
            // replaces what this channel last said and leaves the others
            // standing, exactly as a roster answer does (`queue`).
            Reply::Attention(rows) => self.asking(channel, rows),
            // **`seen`'s receipt carries the queue that remains** (bl-6802),
            // so it is filed exactly as an attention answer from that channel
            // is — which is what takes the ask box down on the frame the
            // receipt lands — and the bar says what was answered.
            Reply::Acknowledged(acked) => {
                let named = self.seen_name(channel, &acked);
                self.notice = Some(Notice::seen(&acked, &named));
                self.asking(channel, acked.rows);
            }
            // The records pair, on the same terms as the roles above.
            Reply::Steps(listing) => self.records.steps = Some(listing),
            Reply::Files(answer) => self.records.files = Some(answer),
            // The spine pair, on the same terms again — the records pane's
            // second two reads, standing while it is open (`spine`).
            Reply::Rail(spine) => self.records.rail = Some(spine),
            Reply::Governing(config) => self.records.governing = Some(config),
            // The deeper three, on the same terms (`deep`). The drill-in is
            // posted rather than standing, and it still comes in through this
            // one door: what makes it different is who asked, not how it is
            // filed.
            Reply::Agent(row) => self.records.agent = Some(*row),
            Reply::Inbox(rows) => self.records.mail = Some(rows),
            Reply::Step(records) => self.records.drilled = Some(*records),
            // **The window's own two, one channel's slice at a time** — each
            // op names no workspace, so each answer replaces what its channel
            // last said and leaves the others standing, exactly as a roster
            // answer and a queue answer do (`window`; bl-40ec).
            Reply::Help(rows) => self.paged(channel, rows),
            // The doctor, on the same terms — each channel's own wiring
            // replacing that channel's section (`doctor`; bl-9bbb).
            Reply::Doctor(rows) => self.diagnosed(channel, rows),
            // The trail, on the same terms — every action that crossed ONE
            // engine's boundary, replacing that channel's section (`trail`).
            Reply::Ops(rows) => self.crossed(channel, rows),
            // The price table, on the same terms — and the receipt both money
            // acts answer with, so an act replaces the section it fired from.
            Reply::Prices(table) => self.priced(channel, table),
            // **The ball pane's two channel-wide reads**, on the same terms
            // again — each names no workspace, so each answer replaces its own
            // channel's section (`board`; bl-d2af).
            Reply::Board(board) => self.columned(channel, board),
            Reply::Balls(rows) => self.bound(channel, rows),
            // **And its two aimed reads**, on the roles' terms: filed whether
            // or not the pane is open, because a frame that arrives after it
            // closed is the last one in flight rather than a thing to drop.
            Reply::WorkspaceBalls(rows) => self.holding = Some(rows),
            // **The fleet pane's two reads**, on the roles' terms again — the
            // aimed wall's attempts and what its agents changed (`fleet`;
            // bl-a43a).
            Reply::Science(rows) => self.scienced(rows),
            Reply::Work(rows) => self.worked(rows),
            // **The receipt four ops share, filed under the op that earned
            // it** — never under a family read off the reply, which cannot say
            // which of the two it answers (DESIGN §4.33). A frame that reached
            // this door without an op is a standing read's, and no standing
            // read answers this kind, so it is filed under no name at all.
            Reply::Armed(on) => self.armed("", on),
            Reply::Marks { branch } => self.marks = Some(branch),
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
            Reply::Prepared(prepared) => self.staged(&prepared),
            // **The spread's product is n starts** (§4.36), so the frame that
            // absorbs it composes n fires — §4.26's own argument read over n:
            // the second act belongs to the frame that took the first's
            // receipt, and a candidate prepared and never fired is a worktree
            // balls materialized for nothing.
            Reply::Fanned(rows) => self.fanned(rows),
            // The candidate family's two receipts. Neither is content — there
            // is no row either belongs under — so each is the bar's one line
            // about an act just performed, exactly as §4.34's pair are.
            Reply::Delivered {
                base,
                target,
                source,
                commit,
            } => self.notice = Some(Notice::delivered(&base, &target, source, commit)),
            Reply::Retired { discarded } => self.notice = Some(Notice::retired(discarded)),
            Reply::Started { conversation } => self.started(conversation),
            // **The receipts that carry no fact of their own, and a captured
            // run that succeeded** (bl-9760, bl-7ec8). None is content, but
            // each is the engine's answer to a gesture, and a window that
            // cleared the slot on it could not tell an act that did nothing
            // from one that never fired. So each stands in the state slot as
            // the CLI's own sentence ([`crate::render::receipt`]) — one source
            // for both seats — until Escape or the next notice replaces it. A
            // run that failed is told in the child's own words instead.
            Reply::Outcome(outcome) if !outcome.ok() => {
                self.notice = Some(Notice::Refused(outcome.stderr));
            }
            said @ (Reply::Outcome(_)
            | Reply::Nudged
            | Reply::Flagged
            | Reply::Acked
            | Reply::TrailCleared) => {
                self.notice = Some(Notice::Said(crate::render::receipt(&said)));
            }
            // **The capability boundary's two, which DO carry a fact**
            // (§4.34). Neither is content — there is no row either belongs
            // under — so each becomes the bar's one line about an act just
            // performed, and the sentence is the whole product: what the
            // answer landed on and whether the conversation is running again,
            // or whether a floor stands over it now.
            Reply::Answered {
                tool,
                tool_use,
                verdict,
                scope,
                advanced,
            } => {
                self.notice = Some(Notice::answered(
                    &tool, &tool_use, &verdict, &scope, advanced,
                ));
            }
            Reply::Floored { standing } => self.notice = Some(Notice::floored(standing)),
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
}

/// Whether `op`'s receipt is already on the glass: the two deposits put the
/// operator's own words on the transcript, and the three role writes the
/// rule's selectors spend repaint the segment they were picked on.
fn witnessed(op: &str) -> bool {
    [
        crate::verbs::MESSAGE.word,
        crate::verbs::INTERRUPT.word,
        crate::verbs::MODEL.word,
        crate::verbs::EFFORT,
        crate::verbs::PRIORITY,
    ]
    .contains(&op)
}

#[cfg(test)]
mod tests;
