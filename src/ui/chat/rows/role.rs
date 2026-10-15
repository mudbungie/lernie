//! **A row's role**: whose weight it is said in, and which bubble it stands
//! on (DESIGN §4.45 item 7; `docs/STYLE.md` §5, *a block*).
//!
//! Split from [`super`] at the cap on the seam the design draws: the bubble
//! is decided HERE, in the projection, where the turn's shape is known — a
//! prose row is intermediate because a trail follows it within its turn, or
//! because its turn has not settled and what follows is not known yet — and
//! never by the painter counting. The weight of a delivered message is the
//! same kind of fact, read off the sender, so it stands beside the bubble.

use super::{Kind, Row};
use crate::ui::theme::{Bubble, Speaker};

/// **The senders that are the operator's own.** litany writes a deposit from
/// the seat with `user` (`corpus/answers/transcript.json`), and `op` is the
/// same claim written short — the form a hand-written transcript and this
/// crate's own fixtures carry. Reading either as a peer would paint the
/// operator's own words in a peer's weight, which is the one weight on the
/// glass a reader is certain about.
pub(crate) const OPERATOR: [&str; 2] = ["user", "op"];

/// **Who a delivered message is, by weight**: the operator's own deposit is
/// their act and wears the brand, a sender the engine buried has ended, and
/// anything else that speaks here is a peer.
pub(super) fn delivered(sender: &str, epitaph: Option<&String>) -> Speaker {
    if OPERATOR.contains(&sender) {
        Speaker::Operator
    } else if epitaph.is_some() {
        Speaker::Ended
    } else {
        Speaker::Peer
    }
}

/// **The bubble a row is born with**, from what it is: the operator's turn
/// on the operator's, a step of the trail on the trail's, and any other prose
/// on the final rung until [`settle`] reads the turn around it.
pub(super) fn born(weight: Speaker, kind: Kind) -> Bubble {
    match (weight, kind) {
        (Speaker::Operator, Kind::Said) => Bubble::Operator,
        (_, Kind::Said) => Bubble::Final,
        _ => Bubble::Trail,
    }
}

/// **The pass that reads each turn's shape.** A turn is an operator's row
/// and everything to the next one; `running` says the transcript holds a
/// `Streaming` entry, which rides the open response (REMOTE §5.5) and so
/// stands in the LAST turn — the one that has not settled. Within a turn,
/// prose a further step follows is intermediate; in the running turn every
/// prose is, since what follows it is not known yet; and every row of the
/// running turn is [`Row::live`], which is what a trail's fold reads for its
/// default (§4.45 item 5).
pub(super) fn settle(rows: &mut [Row], running: bool) {
    let mut rest = rows;
    while !rest.is_empty() {
        let next = rest
            .iter()
            .skip(1)
            .position(|row| row.bubble == Bubble::Operator)
            .map_or(rest.len(), |i| i + 1);
        let live = running && next == rest.len();
        let (turn, after) = rest.split_at_mut(next);
        // Read back to front, so *a step follows* is one bit carried along.
        let mut followed = false;
        for row in turn.iter_mut().rev() {
            row.live = live;
            if row.bubble == Bubble::Final && (followed || live) {
                row.bubble = Bubble::Intermediate;
            }
            followed |= row.kind != Kind::Said;
        }
        rest = after;
    }
}

#[cfg(test)]
mod tests;
