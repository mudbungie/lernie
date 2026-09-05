//! **Which wall the window is aimed at**, and the two questions the window
//! asks about a channel's name.
//!
//! Split from [`super`] at the design-time budget on the seam that module's
//! own doc draws twice already: [`super`] is *what the window holds between
//! frames*, and the pieces that are a subject of their own live beside it —
//! the reply door, the acts, the panes. This is the aim, which is the subject
//! every composed gesture is addressed by.
//!
//! **The two readings here are the aim's, and they are the same fact from two
//! ends.** [`Model::holds`] asks whether this box has a channel by that name
//! at all, which is the one aim whose emptiness is permanent; [`Model::aimed_at`]
//! asks whether a given row is the one the window is on. Both are pure
//! functions of the roster and the aim, so a test reads each back as a value.

use super::Model;

/// Which wall the window is aimed at: the channel it came down, and the address
/// a gesture must carry. **The address rather than the row's name**, because
/// the two differ exactly where an entry renames — and this is the value every
/// composed gesture is built from.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Aim {
    pub channel: String,
    pub address: String,
}

impl Model {
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
