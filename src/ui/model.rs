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
/// What a channel is, and what a gesture aimed down one must be addressed as.
mod channel;
/// The claim a start leaves on the selection, and the row it stands in for.
mod claim;
/// An enrollment, between the control that opened it and the symbol it ends at.
mod enroll;
/// Which composer box a row menu's navigation asked for the cursor in.
mod fill;
/// What the seat last heard that was not content, and how the shell says it.
mod notice;
/// What a frame composed, and what a lost reply would mean for it.
mod posted;
/// The decision queue between frames: what is asking, and the acts on a row.
mod queue;
/// The records pane between frames: open or not, and what its two reads filed.
mod records;
/// A start, between its two acts.
mod start;
/// The tuning pane between frames, and the four acts its controls spend.
mod tuning;
/// An unmaking between frames: its subject, its arming, and whether it is asked.
mod unmake;
/// The window's own two panes: the engines' verb table, and what a needle found.
mod window;

pub use channel::{Channel, Chunk, Held};
pub use enroll::{Enrolling, Grade, Shown};
pub use fill::Fill;
pub use notice::Notice;
pub use posted::Posted;
pub use queue::Asking;
pub use start::{Phase, Start};
pub use tuning::{Edit, Tuning};
pub use unmake::Unmaking;
pub use window::{Hits, Lookup, Pages};

/// Which wall the window is aimed at: the channel it came down, and the address
/// a gesture must carry. **The address rather than the row's name**, because
/// the two differ exactly where an entry renames — and this is the value every
/// composed gesture is built from.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Aim {
    pub channel: String,
    pub address: String,
}

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
    /// **Whether the records pane is open** on the selected conversation — the
    /// third covering pane, and a flag because it has one state (`records`).
    pub records: bool,
    /// **Whether the decision queue is open** — the fourth covering pane, and
    /// the first whose subject is neither the aim nor the selection (`queue`).
    pub queue: bool,
    /// **Which of the window's own two panes is standing**, if either — the
    /// sixth and seventh covering panes, and the second and third whose
    /// subject is every channel (`window`; bl-40ec). One field rather than two
    /// flags, because no two panes ever stand together and a pair of bools
    /// would make *both* representable.
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

    /// **Whether this seat holds a channel by that name.** The roster carries
    /// every channel this box holds from boot — read off the disk, before
    /// anything is dialled (`crate::seat::channels`) — so a name it does not
    /// carry is a name no worker will ever ask anything about
    /// (`crate::state::Standing::aimed`), which is the one aim whose emptiness
    /// is permanent.
    pub fn holds(&self, channel: &str) -> bool {
        self.roster
            .iter()
            .any(|chunk| chunk.channel.name == channel)
    }

    /// Whether this row is the one the window is aimed at.
    pub fn aimed_at(&self, channel: &str, address: Option<&String>) -> bool {
        match (&self.aim, address) {
            (Some(aim), Some(address)) => aim.channel == channel && aim.address == *address,
            _ => false,
        }
    }
}

#[cfg(test)]
mod tests;
