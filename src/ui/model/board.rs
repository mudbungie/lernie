//! **The ball pane between frames** (bl-d2af): open or not, what each channel
//! last said its board and its bindings are, and what the aimed wall last said
//! it holds.
//!
//! # It is the queue's shape and the tuning pane's, at once
//!
//! Two of its four reads name no workspace — `balls` and `board` — so their
//! subject is every channel this box holds and each answer replaces its own
//! channel's section (`super::queue`'s rule, and the roster's before it). Two
//! name one — `workspace-balls` and `marks` — so their subject is the aimed
//! wall, and they are retired when the aim moves exactly as the roles are
//! (`super::tuning`).
//!
//! **The PANE is the channel-wide one, and only its wall half is retired.** A
//! pane about every channel's board does not stop being about them because the
//! operator aimed somewhere else; what would be wrong is painting one wall's
//! balls under another wall's name, and that is the answers, not the pane. So
//! [`Model::aim_at`](super::Model) drops the two wall answers and leaves the
//! pane standing — which is the same rule the tuning pane keeps, read on a
//! pane whose subject is wider than its aimed section.
//!
//! # All four reads STAND, on the trail's terms
//!
//! A board is what is happening: a drone starts, a ball is claimed, a loop
//! ticks, and every one of those changes the pane under an operator who is
//! looking at it. So the set is keyed on the pane (`crate::state::Open::Board`)
//! and asked only while somebody is looking — the trail's reasoning
//! (`super::trail`), one noun over.

use super::{Lookup, Model};
use crate::reply::balls::BallRow;
use crate::ui::Channel;

/// **One channel's board** — its columns and its loops, and the channel they
/// came down as the client's own stamp.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Columns {
    pub channel: Channel,
    pub board: crate::reply::board::Board,
}

/// **One channel's binding table**, stamped the same way.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Bindings {
    pub channel: Channel,
    pub rows: Vec<BallRow>,
}

impl Model {
    /// **Open the ball pane.** It takes no subject: two of its four reads name
    /// no workspace, so *what is on the board* is answerable from an unaimed
    /// seat — and that is the seat most likely to be asking it. An aim only
    /// adds the wall's own half.
    pub fn begin_board(&mut self) {
        self.lookup = Some(Lookup::Board);
    }

    /// **Whether the ball pane is the one standing.**
    pub fn boarding(&self) -> bool {
        self.lookup == Some(Lookup::Board)
    }

    /// File one channel's board, on [`Model::asking`](super::Model)'s terms:
    /// this channel's section is replaced and every other stands.
    pub(super) fn columned(&mut self, channel: &Channel, board: crate::reply::board::Board) {
        let answered = Columns {
            channel: channel.clone(),
            board,
        };
        match self
            .columns
            .iter_mut()
            .find(|held| held.channel.name == channel.name)
        {
            Some(held) => *held = answered,
            None => self.columns.push(answered),
        }
    }

    /// File one channel's binding table, on the same terms.
    pub(super) fn bound(&mut self, channel: &Channel, rows: Vec<BallRow>) {
        let answered = Bindings {
            channel: channel.clone(),
            rows,
        };
        match self
            .bindings
            .iter_mut()
            .find(|held| held.channel.name == channel.name)
        {
            Some(held) => *held = answered,
            None => self.bindings.push(answered),
        }
    }

    /// **The aimed wall's two answers go with the wall they are about**,
    /// called by the act that moves the aim. `None` is a wall nobody has asked
    /// yet, and the old wall's answer is not this wall's — the reading
    /// `Model::roles` gets, for the same reason.
    pub(super) fn retire_wall_balls(&mut self) {
        self.holding = None;
        self.marks = None;
    }
}

#[cfg(test)]
mod tests;
