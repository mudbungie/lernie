//! **What a channel is**, and what a gesture aimed down one must be addressed
//! as (yog's `docs/REMOTE.md` §8.2).
//!
//! Split from [`super`] at the design-time budget on a seam the two already
//! have: [`super`] is what the window holds *right now* and how a reply changes
//! it, and this is the standing fact about where an answer came from. The
//! second changes when the operator provisions a channel; the first changes
//! every frame.

use crate::reply::roster::WsRow;

/// **One channel this box holds**, as the roster stamps the rows that came down
/// it. The stamp is the client's, applied here: no origin crosses the wire.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Channel {
    /// What this box calls it — an entry's leaf, or the label the flat root
    /// wears. It is the roster's section header.
    pub name: String,
    /// The name that workspace bears **on its host**, or `None` for this box's
    /// own engine.
    ///
    /// It is what makes [`Self::address`] exact rather than a guess, and the
    /// fact is §8.2's: an entry resolves by its leaf and by nothing else.
    pub named_there: Option<String>,
}

impl Channel {
    /// **The name a gesture must carry to reach `row` down this channel**, or
    /// `None` where this seat holds no name for it.
    ///
    /// Three cases and the third is a real one. This box's own engine rewrites
    /// nothing, so a row is addressed by its own name. An entry rewrites its
    /// leaf to the host's name at the channel boundary
    /// ([`crate::seat::route`]), so the leaf is the address of the one
    /// workspace that entry names. And an entry's engine may answer a row the
    /// entry does **not** name — a workspace this client is registered in and
    /// holds no entry for — which is reachable by no envelope this seat can
    /// write. It is painted, and painted as unreachable: dropping it would hide
    /// a workspace the operator has, and addressing it by the leaf would aim a
    /// gesture at a different wall.
    pub fn address(&self, row: &WsRow) -> Option<String> {
        match &self.named_there {
            None => Some(row.workspace.clone()),
            Some(there) if *there == row.workspace => Some(self.name.clone()),
            Some(_) => None,
        }
    }
}

/// One channel's answer, as the roster shows it: the section, its currency, and
/// its walls.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Chunk {
    pub channel: Channel,
    /// How stale the derivation behind these rows is, when the engine said.
    pub stale: Option<String>,
    /// What grew since the previous one, when anything did.
    pub growth: Option<String>,
    pub walls: Vec<WsRow>,
}

impl Chunk {
    /// A channel with nothing behind it yet — what the roster holds before any
    /// answer has come down it, and what a box that has never been asked shows.
    pub fn of(channel: Channel) -> Self {
        Self {
            channel,
            ..Self::default()
        }
    }
}
