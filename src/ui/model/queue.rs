//! **The decision queue between frames** (bl-f0ef): open or not, what each
//! channel last said was asking, and the two acts its rows spend.
//!
//! # The first pane whose subject is not a focus
//!
//! The tuning pane is about the aimed wall and the records pane about the
//! selected conversation, so both are retired when their subject moves. This
//! one is about **everything**: `attention` names no workspace, so its subject
//! is every channel this box holds and no aim or selection can invalidate it.
//! It therefore survives [`Model::aim_at`] and [`Model::select`] — which is not
//! an omission but the same rule those two keep, read on a pane whose subject
//! nothing on the glass can move.
//!
//! # A union across channels, replaced one channel at a time
//!
//! The read fans (`crate::offframe::asker`), so one pass is one frame per
//! channel and each answer is that engine's whole queue. So an answer replaces
//! **its own channel's** [`Asking`] and leaves every other standing — REMOTE
//! §8.2's *"a refusal is one entry's, never the set's"*, the same shape
//! [`Model::seat`](super::Model) keeps for the roster.
//!
//! # A row is addressed off the ROSTER, never off the section it came down
//!
//! A queue row names its workspace as **its host** names it, which is not what
//! a gesture from this box must carry when an entry renames (§8.2). The
//! resolution already has one home in this window — a roster row and
//! `crate::ui::Channel::address` — so [`Model::wall`] asks it there rather than
//! re-deriving it from the channel a frame was stamped with. Two things follow
//! and both are the point. A stamp is a display fact and cannot mis-aim a
//! gesture, which matters because a receipt's stamp is the *aimed* channel
//! rather than the one that answered (`crate::offframe::poster`). And a row for
//! a wall this seat holds no name for is honestly unaddressable, and paints as
//! such, rather than being aimed by a guess at a different wall.

use super::{Aim, Model};
use crate::reply::queue::QueueRow;
use crate::ui::Channel;

/// **One channel's answer to *what is asking*** — the rows, and the channel
/// they came down as the client's own stamp.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Asking {
    pub channel: Channel,
    pub rows: Vec<QueueRow>,
}

impl Model {
    /// **Open the queue pane.** It takes no subject, so nothing gates it: the
    /// question *what is waiting on me* is answerable from an unaimed,
    /// unselected seat, and that is the seat most likely to be asking it.
    pub fn begin_queue(&mut self) {
        self.stand(super::Listing::Queue);
    }

    /// **Close it.** The rows stay, for the reason the roles and the records
    /// do: the next open is about the same queue, and the standing read
    /// replaces them anyway.
    pub fn close_queue(&mut self) {
        self.put_down(super::Listing::Queue);
    }

    /// File one channel's queue, replacing what that channel last said and
    /// leaving every other channel standing.
    pub(super) fn asking(&mut self, channel: &Channel, rows: Vec<QueueRow>) {
        let answered = Asking {
            channel: channel.clone(),
            rows,
        };
        match self
            .waiting
            .iter_mut()
            .find(|held| held.channel.name == channel.name)
        {
            Some(held) => *held = answered,
            None => self.waiting.push(answered),
        }
    }

    /// **The aim that reaches the wall a queue row names**, or `None` where
    /// this seat holds no name for it.
    ///
    /// It walks the roster because the roster is where this window's half of
    /// §8.2's mapping lives: a wall's row plus the channel it came down is
    /// exactly what `Channel::address` answers, and asking it here means the
    /// queue cannot hold a second opinion about how a wall is addressed.
    pub fn wall(&self, workspace: &str) -> Option<Aim> {
        self.roster.iter().find_map(|chunk| {
            chunk
                .walls
                .iter()
                .find(|wall| wall.workspace == workspace)
                .and_then(|wall| chunk.channel.address(wall))
                .map(|address| Aim {
                    channel: chunk.channel.name.clone(),
                    address,
                })
        })
    }

    /// **Answer a row's place in the queue**, or do nothing where this seat
    /// cannot address the wall it names. The gate is the address rather than a
    /// separate check: a gesture with nowhere to go is a gesture nobody can
    /// compose, and the pane paints that row's own sentence instead of a
    /// control.
    pub fn post_seen(&mut self, row: &QueueRow) {
        if let Some(aim) = self.wall(&row.workspace) {
            self.outbox.push(super::Posted::act(crate::verbs::seen(
                aim.address,
                row.agent.clone(),
            )));
        }
    }

    /// **Answer the invocation parked at a row's conversation**, or do nothing
    /// where this seat cannot address the wall it names — [`Self::post_seen`]'s
    /// gate, for its reason exactly.
    ///
    /// The verdict is the only parameter: *which* call is answered is read at
    /// the far end off the conversation's own hold mark at fire time, so this
    /// end names no invocation and cannot spend one that is no longer parked
    /// (`crate::verbs::capability`).
    pub fn post_answer(&mut self, row: &QueueRow, verdict: &str) {
        if let Some(aim) = self.wall(&row.workspace) {
            self.outbox.push(super::Posted::act(crate::verbs::answer(
                aim.address,
                row.agent.clone(),
                verdict.to_owned(),
            )));
        }
    }

    /// **Go to the conversation a row is about**: aim at its wall, select it,
    /// and stand the pane down.
    ///
    /// It crosses no wire of its own — an aim and a selection are views, which
    /// yog's `docs/PARITY.md` §2 puts out of the parity contract — and it
    /// carries no `act:` token for exactly that reason. What it spends is the
    /// two doors a click on the roster and a click on the list already spend,
    /// so a queue row reaches the conversation by the same gestures rather than
    /// by a third spelling of *look at this one*.
    pub fn go_to(&mut self, row: &QueueRow) {
        let Some(aim) = self.wall(&row.workspace) else {
            return;
        };
        self.aim_at(&aim.channel, &aim.address);
        self.select(&row.agent.clone());
        self.close_queue();
    }
}

#[cfg(test)]
mod tests;
