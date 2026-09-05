//! **What the window holds between frames**, and the one door a reply comes in
//! through.
//!
//! Nothing here paints and nothing here dials. It is the snapshot a frame reads
//! and the rules for changing it, so every one of them is a pure function a
//! test reads back as a value.
//!
//! **[`Model::absorb`] is the single door, and it drops nothing.** A frame that
//! arrives is either content this build paints or a [`Notice`] the shell paints
//! *where that content would have been* — which is the reply vocabulary's own
//! policy (`crate::reply`, rung 2) honoured on the screen rather than only in
//! the type: a refusal and an unreadable frame are both visible rows, and
//! neither is a silent drop.

use crate::reply::convs::ConvRow;
use crate::reply::stream::Stream;
use crate::reply::transcript::Transcript;

/// The one door a reply comes in through, and the leg that brought none.
mod absorb;
/// What a control does, whichever control did it.
mod acts;
/// Which wall the window is aimed at, and the two questions asked of a name.
mod aim;
/// What a channel is, and what a gesture aimed down one must be addressed as.
mod channel;
/// The claim a start leaves on the selection, and the row it stands in for.
mod claim;
/// The clients pane between frames: whether it is open, and what it filed.
mod clients;
/// An enrollment, between the control that opened it and the symbol it ends at.
mod enroll;
/// Which composer box a row menu's navigation asked for the cursor in.
mod fill;
/// The three panes that are pure listings, and the field that says which is up.
mod listing;
/// The login pane between frames: what it asks about, and the acts it spends.
mod login;
/// What the seat last heard that was not content, and how the shell says it.
mod notice;
/// What a frame composed, and what a lost reply would mean for it.
mod posted;
/// The decision queue between frames: what is asking, and the acts on a row.
mod queue;
/// The records pane between frames: open or not, and what its two reads filed.
mod records;
/// The spine's own state: the draft a fork is composed from.
mod spine;
/// A start, between its two acts.
mod start;
/// The trail pane between frames: open or not, and what each channel has done.
mod trail;
/// The tuning pane between frames, and the four acts its controls spend.
mod tuning;
/// An unmaking between frames: its subject, its arming, and whether it is asked.
mod unmake;
/// The window's own two panes: the engines' verb table, and what a needle found.
mod window;

pub use aim::Aim;
pub use channel::{Channel, Chunk, Held};
pub use enroll::{Enrolling, Grade, Shown};
pub use fill::Fill;
pub use listing::Listing;
pub use login::Login;
pub use notice::Notice;
pub use posted::Posted;
pub use queue::Asking;
pub use spine::Forking;
pub use start::{Phase, Start};
pub use trail::Trail;
pub use tuning::{Edit, Tuning};
pub use unmake::Unmaking;
pub use window::{Hits, Lookup, Pages};

/// Everything the window holds between frames.
#[derive(Debug, Clone, Default)]
pub struct Model {
    /// Every channel's workspaces, in the order the channels were asked.
    pub roster: Vec<Chunk>,
    /// The aimed wall's conversations.
    pub convs: Vec<ConvRow>,
    /// **Which wall [`Self::convs`] is the ANSWER to**, or `None` while the aim
    /// has not been answered about at all (bl-f780).
    ///
    /// It rides beside the rows rather than being derived, because emptiness
    /// cannot tell the two apart: a wall that answered zero conversations and a
    /// wall nobody has asked about yet hold the same `Vec`, and painting *"no
    /// conversations here"* over the second states a definite fact about a wall
    /// nobody looked at — the same thing `crate::ui::convs::UNCERTAIN` refuses
    /// to do one level down, on a conversation nobody could take a reading of.
    ///
    /// It is set and cleared at exactly the two places `convs` is, and nowhere
    /// else.
    pub answered: Option<Aim>,
    /// **What the aimed wall's roles are set to**, or `None` while nobody has
    /// been answered about them — one field where [`Self::convs`] needs a pair,
    /// because here the option carries the whole distinction (`tuning`).
    pub roles: Option<Vec<crate::reply::roles::RoleRow>>,
    /// **The tuning pane, while it is open** — the second pane in this window
    /// that covers the conversation ([`Tuning`]).
    pub tuning: Option<Tuning>,
    /// **Which of the three LISTING panes is standing**, if any — the records
    /// pane, the decision queue, or the clients pane (`listing`). One field
    /// rather than a flag each: they hold nothing of their own, no two ever
    /// stand together, and three bools would make *two of them open at once* a
    /// state only the derivation order resolves — the reframe clippy's
    /// `struct_excessive_bools` names, taken rather than suppressed.
    pub listing: Option<Listing>,
    /// **The machines registered in the aimed wall's workspace**, or `None`
    /// while nobody has been answered about it — the one-option reading
    /// [`Self::roles`] gets, one noun over (`clients`).
    pub machines: Option<Vec<crate::reply::clients::ClientRow>>,
    /// **The login pane, while it is open** — the eighth covering pane, and
    /// the second whose subject is the aimed wall (`login`; DESIGN §4.24). A
    /// struct rather than a flag because it holds two questions of its own:
    /// which row a sign-in is being followed on, and which was asked what it
    /// offers.
    pub login: Option<Login>,
    /// **What the aimed wall can sign in to**, or `None` while nobody has been
    /// answered about it — the one-option reading [`Self::roles`] gets, and
    /// filed here rather than on the pane because the rows are the engine's.
    pub providers: Option<Vec<crate::reply::providers::ProviderRow>>,
    /// **What the last row asked answered with** — the posted read's answer,
    /// dropped by the act that asks another (`login`). Which row it is about
    /// is the pane's `asking`, because the reply carries no name.
    pub offered: Option<Vec<String>>,
    /// **The sign-in run this seat is following**, as the lane has folded it —
    /// the login pane's held read, and [`Self::live`]'s shape one noun over
    /// (`login`).
    pub signin: Option<crate::reply::login::Signin>,
    /// **Which of the window's own three panes is standing**, if any — the
    /// sixth, seventh and tenth covering panes, and the three whose subject is
    /// every channel and which are opened from the roster's own ops row
    /// (`window`, `trail`; bl-40ec, bl-4c48). One field rather than three
    /// flags, because no two panes ever stand together and a set of bools
    /// would make *all three* representable — which is also the reframe
    /// clippy's `struct_excessive_bools` asks for by name, taken rather than
    /// suppressed.
    ///
    /// It is a second field beside [`Self::listing`] and not one with it,
    /// because the two name different axes: these three are the WINDOW's own —
    /// reached from the ops row, about every channel, and one of them holds a
    /// needle — while a listing is a pane about one thing on the glass that
    /// holds nothing at all. DESIGN §4.28 records the fold that would make
    /// them one.
    pub lookup: Option<Lookup>,
    /// **An unmaking, while it stands** — the fifth covering pane, and the only
    /// one this window has whose act cannot be undone by doing the other thing
    /// (`unmake`; DESIGN §4.20). It carries the wall it was opened on rather
    /// than following the aim, because the roster stays live under it.
    pub unmaking: Option<Unmaking>,
    /// **What each channel last said is asking for the operator**, one section
    /// per channel and the union across them. A `Vec` rather than an option
    /// because the emptiness that matters is per channel: nothing here at all
    /// is nobody answered yet, and a section holding no row is an engine that
    /// answered and holds nothing waiting (`queue`).
    pub waiting: Vec<Asking>,
    /// **What each channel last said has crossed its boundary** — the same
    /// per-channel reading, one noun over (`trail`; bl-4c48).
    pub trails: Vec<Trail>,
    /// **What each channel last said it answers to** — the same per-channel
    /// reading, one noun over (`window`; bl-40ec).
    pub pages: Vec<Pages>,
    /// **What each channel last found**, on the same terms (`window`).
    pub found: Vec<Hits>,
    /// **What to look for.** A box the find pane holds and does not spend on
    /// firing, because refining a needle is the common act — unlike the
    /// composer's draft, which was sent (`window`).
    pub needle: String,
    /// **The steps its loop has taken**, or `None` while nobody has been
    /// answered — the same one-option reading [`Self::roles`] gets (`records`).
    pub steps: Option<crate::reply::steps::Steps>,
    /// **What its worktree holds**, on the same standing (`records`).
    pub files: Option<crate::reply::files::Files>,
    /// **Its spine** — every operable commit and the cards off them — or
    /// `None` while nobody has been answered, on the same standing
    /// (`records`, `spine`; DESIGN §4.28).
    pub rail: Option<crate::reply::rail::Rail>,
    /// **The config commit governing it**, on the same standing (`records`).
    pub governing: Option<crate::reply::governing::Governing>,
    /// **The draft a fork is composed from** (`spine`): the role and the goal,
    /// beside the `from` each control carries off its own notch. A bare field
    /// and not an option, exactly as the composer's two parameter boxes are —
    /// the boxes are on the glass whenever the spine is.
    pub forking: Forking,
    /// The selected conversation, as committed.
    pub transcript: Transcript,
    /// The live tail as this seat has accumulated it. It **replaces**, never
    /// accretes — and since PROTOCOL 2 that is a statement about where the
    /// accretion happens rather than whether it does: a follow frame is an
    /// append (REMOTE §5.5), so `crate::offframe::follow` absorbs each frame
    /// onto the read's own fold and what reaches here is already whole. The
    /// fold's lifetime is one read, which is what keeps two reads of one
    /// conversation from running into each other.
    pub live: Option<Stream>,
    /// What the seat last heard that was not content.
    pub notice: Option<Notice>,
    /// Which wall the window is aimed at.
    pub aim: Option<Aim>,
    /// **Which list the arrow keys belong to**, and the one thing the keyboard
    /// holds that the pointer does not need: a click names its own row, and a
    /// key has to be told which list it is in ([`crate::ui::keys`]).
    pub focus: crate::ui::keys::Pane,
    /// **Which column is on the glass in the narrow shape** — a navigation the
    /// operator performed, so it is the one thing here no other fact can be
    /// asked for. The broad shape never reads it, because every column is on
    /// the glass there (`crate::ui::shell::policy`).
    pub column: crate::ui::Column,
    /// **Whether the focused list owes its selection a place on the glass**,
    /// set by a keyboard walk and taken by the pane that paints it
    /// ([`Model::revealing`]).
    ///
    /// The keyboard is the only surface that can move a selection out of view:
    /// a click names a row the operator is already looking at, and a scroll IS
    /// the operator deciding what to look at — so a pane that revealed on every
    /// frame would drag the glass back every time they scrolled away.
    pub reveal: bool,
    /// The selected conversation's id.
    pub conversation: Option<String>,
    /// What the operator has typed and not yet sent.
    pub draft: String,
    /// **The arming for the unmaking** (`crate::ui::composer::acts`): the name
    /// typed back, which is what admits a conversation's descendants into its
    /// deletion. Empty is the bare form and deletes the one conversation, so
    /// this is not a second control's enablement — it is the gesture's own
    /// third parameter, held where the box that fills it is.
    pub typed: String,
    /// **Which of the two boxes above a row menu asked for the cursor in**, or
    /// `None` where nobody has asked (`fill`; bl-dbc9). A conversation row's
    /// menu cannot hold either box, so the item that would need one goes to it
    /// instead — and this is the request, taken by the frame that paints it.
    pub fill: Option<Fill>,
    /// **The words a flag is raised with** (`crate::ui::composer::acts`). The
    /// wire requires them — a flag with nothing in it is a row nobody can
    /// triage — so the control is disabled until there are any, and the box is
    /// SPENT on firing: what a flag says is said, exactly as a deposit is.
    pub reason: String,
    /// **A start, while it is happening** — the one thing this window holds
    /// across a round trip, because starting is two acts and the second is
    /// composed from the first's answer ([`Start`]).
    pub start: Option<Start>,
    /// **An enrollment, while it is happening** — the second thing this window
    /// holds across a round trip, and the only thing it ever holds that is a
    /// secret. It is dropped by a control and written down nowhere
    /// ([`Enrolling`]).
    pub enroll: Option<Enrolling>,
    /// **The gestures this frame composed**, for whoever can send them. A frame
    /// that posted its own would be a frame that waits.
    ///
    /// Each carries whether it is an ACT, said by the control that composed it,
    /// because a lost reply means opposite things for the two ([`Posted`]).
    pub outbox: Vec<Posted>,
}

impl Model {
    /// **Whether `pane` must bring its selection onto the glass this frame**,
    /// answered once: the flag is taken, so two panes cannot both act on one
    /// keypress and a stale one cannot fight the next frame's scroll.
    pub fn revealing(&mut self, pane: crate::ui::keys::Pane) -> bool {
        self.focus == pane && std::mem::take(&mut self.reveal)
    }
}

#[cfg(test)]
mod tests;
