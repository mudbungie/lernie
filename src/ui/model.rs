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
/// A start, between its two acts.
mod start;

pub use channel::{Channel, Chunk};
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

/// What the seat last heard that it could not turn into content.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Notice {
    /// The engine refused, in its own words.
    Refused(String),
    /// This seat could not read what arrived — a malformed frame, or a kind
    /// this build does not paint. A statement about **this seat**, which is why
    /// it reads differently from a refusal.
    Unreadable(String),
    /// This seat could not **reach** the far end: a channel that will not open,
    /// an engine that is not there, a preface that did not agree.
    ///
    /// Three arms and not two, because the remedies are three different acts. A
    /// refusal is answered by typing something else; an unreadable frame by
    /// upgrading the seat; an unreachable channel by looking at this box's own
    /// files or at whether the engine is up. A seat that collapsed them would
    /// send an operator to check a certificate over a workspace they mistyped.
    Unreachable(String),
}

impl Notice {
    /// The line the shell paints, with the half that says whose sentence it is.
    pub fn line(&self) -> String {
        match self {
            Self::Refused(said) => format!("the engine refused: {said}"),
            Self::Unreadable(why) => format!("this seat could not read the answer: {why}"),
            Self::Unreachable(why) => format!("this seat could not reach it: {why}"),
        }
    }
}

/// Everything the window holds between frames.
#[derive(Debug, Clone, Default)]
pub struct Model {
    /// Every channel's workspaces, in the order the channels were asked.
    pub roster: Vec<Chunk>,
    /// The aimed wall's conversations.
    pub convs: Vec<ConvRow>,
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
    /// The selected conversation's id.
    pub conversation: Option<String>,
    /// What the operator has typed and not yet sent.
    pub draft: String,
    /// **A start, while it is happening** — the one thing this window holds
    /// across a round trip, because starting is two acts and the second is
    /// composed from the first's answer ([`Start`]).
    pub start: Option<Start>,
    /// **The gestures this frame composed**, for whoever can send them. A frame
    /// that posted its own would be a frame that waits.
    pub outbox: Vec<Value>,
}

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
                self.resolve();
            }
            Reply::Transcript(transcript) => self.transcript = transcript,
            Reply::Follow(stream) => self.live = Some(stream),
            // The start family's two, whose whole product is each other: the
            // staged body composes the fire, and the fire's receipt is the
            // minted name. [`Start`] holds the chain.
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
