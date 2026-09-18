//! **Which wall the window is aimed at**, and the two questions the window
//! asks about a channel's name.
//!
//! Split from [`super`] at the design-time budget on the seam that module's
//! own doc draws twice already: [`super`] is *what the window holds between
//! frames*, and the pieces that are a subject of their own live beside it —
//! the reply door, the acts, the panes. This is the aim, which is the subject
//! every composed gesture is addressed by.
//!
//! **The readings here are the aim's own.** [`Model::channel`] resolves the
//! channel a gesture goes down and [`Model::holds`] asks whether this box has
//! one by that name at all, which is the one aim whose emptiness is permanent.
//! Both are pure functions of the roster and the aim, so a test reads each
//! back as a value.
//!
//! **`Model::aimed_at` went with the middle column** (DESIGN §4.39, bl-b9a3).
//! It asked whether a row was the aimed one, for a pane that painted rows
//! about a wall it was not standing under; a wall's row now builds the [`Aim`]
//! it would compose and compares that, which is the same question asked with
//! the value the click already needs.

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
    /// **The channel the aim is on**, where this seat still holds one by that
    /// name — [`crate::state::Standing::aimed`]'s lookup on the model rather
    /// than on the question set, because a control composes its gesture before
    /// any standing set is derived from the frame (bl-4855).
    ///
    /// A focus on a channel that has since gone answers `None`, which is the
    /// honest reading: there is nothing left to address.
    pub fn channel(&self) -> Option<super::Channel> {
        let aim = self.aim.as_ref()?;
        self.roster
            .iter()
            .find(|chunk| chunk.channel.name == aim.channel)
            .map(|chunk| chunk.channel.clone())
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
}

#[cfg(test)]
mod tests;
