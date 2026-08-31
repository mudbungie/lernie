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

use serde_json::Value;

use crate::reply::convs::ConvRow;
use crate::reply::stream::Stream;
use crate::reply::transcript::Transcript;
use crate::reply::{Read, Reply};

/// What a control does, whichever control did it.
mod acts;
/// What a channel is, and what a gesture aimed down one must be addressed as.
mod channel;
/// The claim a start leaves on the selection, and the row it stands in for.
mod claim;
/// An enrollment, between the control that opened it and the symbol it ends at.
mod enroll;
/// What the seat last heard that was not content, and how the shell says it.
mod notice;
/// A start, between its two acts.
mod start;

pub use channel::{Channel, Chunk, Held};
pub use enroll::{Enrolling, Grade, Shown};
pub use notice::Notice;
pub use start::{Phase, Start};

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
    /// The selected conversation, as committed.
    pub transcript: Transcript,
    /// The newest live tail. It **replaces**, never accretes: a follow frame is
    /// the whole accumulated fold, so the newest one wins and nothing has to be
    /// reassembled.
    pub live: Option<Stream>,
    /// What the seat last heard that was not content.
    pub notice: Option<Notice>,
    /// Which wall the window is aimed at.
    pub aim: Option<Aim>,
    /// **Which list the arrow keys belong to**, and the one thing the keyboard
    /// holds that the pointer does not need: a click names its own row, and a
    /// key has to be told which list it is in ([`crate::ui::keys`]).
    pub focus: crate::ui::keys::Pane,
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
    pub outbox: Vec<Value>,
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
            // The two receipts. Neither carries content, so what they change is
            // whether the operator is told something happened — and a captured
            // run that failed is told in the child's own words.
            Reply::Nudged => self.notice = None,
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

    /// **A leg that never reached an engine**, said in this seat's own words.
    ///
    /// It is not a reply and so it does not come through
    /// [`absorb`](Self::absorb): there is no frame, no channel answered, and
    /// nothing to file. What it changes is exactly what a refusal changes —
    /// what the operator is told — and it is told differently, because it is
    /// about this box or the far end rather than about what was asked.
    pub fn unreachable(&mut self, why: String) {
        self.notice = Some(Notice::Unreachable(why));
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
