//! **The census of what an engine can answer** — the kinds this window draws,
//! and the captured run three of the ops on this surface come back as.
//!
//! Split from [`super`] at the 300-line cap on the seam that module's own doc
//! already draws: [`super`] is *the reading* — the three outcomes one frame
//! can be, the four-rung policy every decoder obeys, and the module list — and
//! this is *what an answer turned out to be*. The first changes when the
//! policy does, which is almost never; the second every time a pane lands, and
//! that asymmetry is the whole reason a seam is here.
//!
//! **[`Reply`] is the one census** and DESIGN §4.9 holds the ledger of what a
//! later pane adds: a kind nothing renders is a kind nobody has to carry, and
//! the ball that lands a pane is the ball that adds its kind.

use super::{
    balls, board, clients, config, convs, enrolled, files, governing, help, lineages, login, ops,
    providers, queue, rail, roles, roster, search, start, steps, stream, transcript,
};

/// **The kinds the window draws.** Twenty-eight, and each is here because a
/// surface paints it; DESIGN §4.9 holds the ledger of what a later pane adds.
#[derive(Debug, Clone, PartialEq)]
pub enum Reply {
    /// A short verb's captured run — what a deposit, a stop or a ball verb
    /// earns. `ok` is `exit == 0` and is read off the exit code alone.
    Outcome(Outcome),
    /// The detached advance was launched. It carries nothing because there is
    /// nothing yet: what the model does with the turn arrives on the
    /// transcript, at its own pace, and a receipt that guessed at it would be
    /// a receipt that lied.
    Nudged,
    /// The enumerated workspaces with their rollups — the roster, and how
    /// current the derivation behind it is.
    Workspaces(roster::Workspaces),
    /// One workspace's conversation list.
    Conversations(Vec<convs::ConvRow>),
    /// **What one workspace's roles are set to** — the read the tuning pane
    /// opens on, and the read back of what its own three writes landed. It is
    /// a listing rather than a map: the engine's order is the config's order,
    /// and a seat that re-sorted it would be holding a second opinion about a
    /// file it did not write.
    Roles(Vec<roles::RoleRow>),
    /// One conversation's committed entries with the live tail folded on —
    /// the whole of what the chat pane paints.
    Transcript(transcript::Transcript),
    /// **Everything waiting on the operator**, across every workspace the
    /// engine can see — the decision queue's rows, standing while its pane is
    /// open (bl-f0ef). It is the answer to two ops rather than one: `attention`
    /// asks for the whole queue and `seen` answers with the queue that remains,
    /// so a reading of it is never a receipt to discard.
    Attention(Vec<queue::QueueRow>),
    /// **A flag was raised**, and the row it lands on arrives on the next
    /// [`Attention`](Self::Attention). It carries nothing for the reason
    /// [`Nudged`](Self::Nudged) carries nothing — what changed is on the queue,
    /// and a receipt that restated it would be this end predicting a listing.
    Flagged,
    /// **The steps the selected conversation's loop has taken** — one half of
    /// what the records pane paints, standing while it is open (bl-2cf7).
    Steps(steps::Steps),
    /// **What the selected conversation's worktree holds** — the other half,
    /// on the same standing (bl-2cf7).
    Files(files::Files),
    /// **The selected conversation's spine** — every operable commit it has
    /// and the children dispatched off them, on the records pane's standing
    /// (§4.28, bl-b52c). It is the read the `fork` control's one argument is
    /// discoverable off, which is why the two landed together.
    Rail(rail::Rail),
    /// **The config commit that conversation resolves its policy from** — the
    /// spine's other half, on the same standing (§4.28, bl-b52c). Its `oid`
    /// changed meaning at PROTOCOL 5 under an unchanged spelling, and
    /// `governing`'s own module doc is where that is written down.
    Governing(governing::Governing),
    /// **One engine's own verb table** — what the commands pane paints, and
    /// the same rows the parity roster is generated from (bl-40ec). It names
    /// no workspace, so it is one channel's answer and the pane is the union.
    Help(Vec<help::HelpRow>),
    /// **What a needle found**, across everything one engine can see — the
    /// find pane's answer, on the same fanned terms (bl-40ec).
    Found(search::Found),
    /// **The whole box's ball⇄workspace binding table** — the ball pane's
    /// widest read (bl-d2af). It names no workspace, so it is one channel's
    /// answer and the pane is the union.
    Balls(Vec<balls::BallRow>),
    /// **One engine's fleet board** — every live ball in its column, and the
    /// armed loops running them, which is the only place a loop's own facts
    /// are answered at all (bl-d2af). It names no workspace either.
    Board(board::Board),
    /// **The balls one wall holds**, with what each has cost — the ball pane's
    /// aimed half, standing on the pane exactly as the roles read does.
    WorkspaceBalls(Vec<balls::BoundBall>),
    /// **The branch a wall tracks its tasks on**, re-read. It is one field, so
    /// it is carried as one rather than wrapped in a struct with nothing else
    /// in it.
    Marks { branch: String },
    /// **The trail one engine keeps** — every action that crossed its boundary,
    /// standing while the trail pane is open (bl-4c48). It names no workspace,
    /// so it is one channel's answer and the pane is the union.
    Ops(Vec<ops::OpRow>),
    /// **What this wall can sign in to** — the login pane's first read,
    /// standing while it is open (bl-e3c5). It is the engine's listing order,
    /// which is brazen's own routing order, and a seat that re-sorted it would
    /// be holding a second opinion about a table it does not own.
    Providers(Vec<providers::ProviderRow>),
    /// **The machines registered in one workspace** — the clients pane's one
    /// read, standing while it is open (bl-e53c). Presence is answered at the
    /// moment it is asked and the advertised set is what that machine last
    /// presented, which are two lifetimes on one row and are painted as two.
    Clients(Vec<clients::ClientRow>),
    /// **One config file's bytes, and the settings its schema found in them**
    /// — the config pane's second read, standing on the destination it is
    /// pointed at (bl-5c53; DESIGN §4.30).
    Config(config::Config),
    /// **The config lineages one workspace holds** — that pane's first, and
    /// the listing its two pickers are filled from.
    Lineages(Vec<lineages::Lineage>),
    /// **What one provider row is offering** — the same pane, one depth down,
    /// and posted rather than standing: a model list is fixed for the life of
    /// a provider's own answer, so a standing read would spend a round trip a
    /// beat forever on something that cannot change under the operator.
    Models(Vec<String>),
    /// **One sign-in run**, whether it is the act's own receipt or a frame of
    /// the lane that follows it — upstream answers both with this kind,
    /// because they are the same value at the same moment (REMOTE §8.3).
    Login(login::Signin),
    /// One frame of the live tail, and the whole accumulated stream rather
    /// than a delta: a frame **replaces** what a seat holds, so nothing has to
    /// be reassembled and a follow lane needs no second parser.
    Follow(stream::Stream),
    /// **A start, staged.** The fire-time parameters as the engine settled
    /// them, which the next act hands straight back — the one reply on this
    /// surface a seat has to hold between two gestures.
    Prepared(start::Prepared),
    /// **A new box's material** — the one reply on this surface that carries a
    /// secret, held while a symbol is on screen and written down nowhere
    /// (REMOTE §8.4; DESIGN §3).
    Enrolled(enrolled::Enrolled),
    /// **A start, fired**, and the name the engine minted for it. It carries
    /// nothing else for the reason [`Nudged`](Self::Nudged) carries nothing:
    /// what the model does with the turn arrives on the transcript. What is
    /// new is the name, and the name is an address the reply just made
    /// answerable.
    Started { conversation: String },
}

/// A captured run: what the child said and how it ended.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Outcome {
    /// The child's exit status.
    pub exit: i32,
    pub stdout: String,
    pub stderr: String,
}

impl Outcome {
    /// Whether the run succeeded. **Derived from the exit code**, never read
    /// off the `ok` beside it: one fact with one home, and a second copy could
    /// only ever disagree with it.
    pub fn ok(&self) -> bool {
        self.exit == 0
    }
}
