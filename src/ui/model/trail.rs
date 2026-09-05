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

    /// **Acknowledge every alarm, on every channel this box holds.**
    ///
    /// `ack` names no workspace, so the poster fans it — and that is the right
    /// reading rather than a shape to work around (DESIGN §4.35): the pane is
    /// the union across channels, and an acknowledgement made while looking at
    /// the union is an acknowledgement of the union. What it changed arrives
    /// on the next standing read, which answers `acked` on the rows that were
    /// standing.
    pub fn post_ack(&mut self) {
        self.outbox.push(super::Posted::act(crate::verbs::ack()));
    }

    /// **Open the place a trail is cut in** (DESIGN §4.20's idiom, §4.35's
    /// reading of it). It stands the trail down, being the same field, and
    /// [`Model::close_clearing`] is what brings it back.
    pub fn begin_clearing(&mut self) {
        self.lookup = Some(Lookup::Clearing);
    }

    /// **Whether that place is the pane standing.**
    pub fn clearing(&self) -> bool {
        self.lookup == Some(Lookup::Clearing)
    }

    /// **The way out, which cuts nothing** — it re-opens the trail rather than
    /// leaving the operator on no pane at all, because the trail is where the
    /// gesture that opened this came from and its read is standing.
    pub fn close_clearing(&mut self) {
        self.begin_trail();
    }

    /// **Cut every trail this box can reach**, and go back to looking at them.
    ///
    /// The pane stands down on firing rather than saying *asked*, which is
    /// where this parts from the unmaking (§4.20) and it parts on that pane's
    /// own reasoning: an unmaking's refusal is the COMMON case, so its pane
    /// stays up to hold the arming. Nothing refuses a truncation, and what
    /// answers this one is the trail itself on the next beat — so the place to
    /// be standing when the answer lands is the trail.
    pub fn post_clear_trail(&mut self) {
        self.outbox
            .push(super::Posted::act(crate::verbs::clear_trail()));
        self.begin_trail();
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
