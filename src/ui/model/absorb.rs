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
            // **The one kind whose meaning is the op and not the reply**
            // (bl-a43a): `fleet`, `disband`, `arm` and `disarm` all answer
            // `armed`, and the two families they span are the fleet loop and
            // the alignment monitor. The poster still knows which was sent, so
            // the join happens here rather than by the pane guessing.
            Read::Answer(Reply::Armed(on)) => self.armed(op, on),
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
            // The config pane's two, on the same terms.
            Reply::Config(file) => self.config = Some(file),
            Reply::Lineages(rows) => self.lineages = Some(rows),
            // **The queue, one channel's slice at a time** — the fan's answer
            // replaces what this channel last said and leaves the others
            // standing, exactly as a roster answer does (`queue`).
            Reply::Attention(rows) => self.asking(channel, rows),
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
            // The trail, on the same terms — every action that crossed ONE
            // engine's boundary, replacing that channel's section (`trail`).
            Reply::Ops(rows) => self.crossed(channel, rows),
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
            // The three receipts. None carries content, so what they change is
            // whether the operator is told something happened — and a captured
            // run that failed is told in the child's own words. The advance and
            // the raise are ONE arm rather than two identical ones, which is
            // the honest shape: what each changed arrives on the transcript and
            // on the next queue respectively, and this end predicts neither.
            // **The trail's two receipts join them** (bl-b8f7), and for the
            // identical reason: what each changed is on the trail, and the
            // standing read is what says so. A receipt that restated it would
            // be this end predicting a listing.
            Reply::Nudged | Reply::Flagged | Reply::Acked | Reply::TrailCleared => {
                self.notice = None;
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
                advanced,
            } => self.notice = Some(Notice::answered(&tool, &tool_use, &verdict, advanced)),
            Reply::Floored { standing } => self.notice = Some(Notice::floored(standing)),
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
}
