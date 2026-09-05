//! **The trail pane between frames** (bl-4c48): open or not, and what each
//! channel last said has crossed its boundary.
//!
//! # The decision queue's shape, one noun over
//!
//! `ops` names no workspace, so its subject is every channel this box holds
//! and no aim or selection can invalidate it. Everything follows from that and
//! all of it is [`super::queue`]'s already: the pane holds nothing of its own,
//! it opens from an unaimed seat, it survives
//! [`Model::aim_at`] and [`Model::select`], and one channel's answer replaces
//! **its own** section and leaves every other standing — REMOTE §8.2's *"a
//! refusal is one entry's, never the set's"*.
//!
//! # The read stands, because the trail is what is happening
//!
//! The window's other two channel-wide panes post their reads once
//! (`super::window`): a verb table is fixed for the life of an engine build
//! and a search answers a needle somebody typed. A trail is neither. Every act
//! this seat spends appends a row to it, and an alarm goes up and comes down
//! under an operator who is looking at it — so it stands while the pane is
//! open, on the decision queue's own terms and for the same reason.

use super::{Lookup, Model};
use crate::reply::ops::OpRow;
use crate::ui::Channel;

/// **One channel's trail** — the rows, and the channel they came down as the
/// client's own stamp.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Trail {
    pub channel: Channel,
    pub rows: Vec<OpRow>,
}

impl Model {
    /// **Open the trail pane.** It takes no subject, so nothing gates it:
    /// *what has this box been doing* is answerable from an unaimed,
    /// unselected seat, and that is the seat most likely to be asking it.
    ///
    /// It sets the window's one *which channel-wide pane is standing* field
    /// rather than a flag of its own ([`Lookup`]), so what closes it is
    /// [`Model::close_lookup`](super::Model) — one door for all three, and for
    /// Escape as well.
    pub fn begin_trail(&mut self) {
        self.lookup = Some(Lookup::Trailing);
    }

    /// **Whether the trail is the pane standing.**
    pub fn trailing(&self) -> bool {
        self.lookup == Some(Lookup::Trailing)
    }

    /// File one channel's trail, on [`Model::asking`](super::Model)'s own
    /// terms: this channel's section is replaced and every other stands.
    pub(super) fn crossed(&mut self, channel: &Channel, rows: Vec<OpRow>) {
        let answered = Trail {
            channel: channel.clone(),
            rows,
        };
        match self
            .trails
            .iter_mut()
            .find(|held| held.channel.name == channel.name)
        {
            Some(held) => *held = answered,
            None => self.trails.push(answered),
        }
    }
}

#[cfg(test)]
mod tests;
