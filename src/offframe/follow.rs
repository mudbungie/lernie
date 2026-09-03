//! **The follow lane**: one connection held open on the focused conversation.
//!
//! It is a thread of its own for the reason the framing already gives: every
//! answer is a stream, and a follow-class read is the general path with more
//! frames in it — but it is the one whose N never finishes, so putting it in
//! the serial pass would stall every other read behind a question that is
//! *supposed* to stay open.
//!
//! **Every frame says what it is about, and the frame decides whether it is
//! still wanted.** The engine was asked about a conversation and answers about
//! that one, so a tail that arrives after the operator has moved on would paint
//! one conversation's words under another's name. The guard is a pure
//! comparison at the settle — where what is selected is known for certain —
//! rather than a poll here racing the socket it is parked on.
//!
//! What this end still decides is whether to **stay**: returning `false` hangs
//! up, which is the whole of how a follower whose subject moved says so, and it
//! has to, because the next pass is what opens the lane on the new one.

use std::path::Path;

use crate::reply::{Read, Reply, stream::Stream};
use crate::state::{Link, Said};

/// Hold the line on the focused conversation until it moves or the engine ends
/// the stream.
///
/// **The fold's whole lifetime is this call**, and that is the whole of how
/// REMOTE §5.5's *"onto an empty fold"* is implemented: one read is one `tick`,
/// so a read boundary needs no flag, no field and no representation anywhere —
/// it is a local variable's scope. The frame is handed the ACCUMULATION, which
/// is why [`crate::ui::Model::live`] can go on replacing rather than accreting:
/// what it receives is already whole.
pub fn tick(link: &Link, root: &Path) {
    let standing = link.standing();
    let (Some((channel, aim)), Some(conversation)) = (standing.aimed(), standing.conversation)
    else {
        return;
    };
    let envelope = crate::verbs::follow(aim.address.clone(), conversation.clone());
    let mut fold = Stream::default();
    let held = crate::seat::route(root, &envelope)
        .map_err(crate::channel::Reach::Unsent)
        .and_then(|(open, carried)| {
            open.follow(&carried, &mut |frame| {
                link.live(&channel, &conversation, absorbed(&mut fold, &frame));
                !link.stopped() && still_on(link, &aim, &conversation)
            })
        });
    // **A held read is still a read** (REMOTE §3): the lane re-opens on the next
    // pass onto an empty fold, so a connection that died mid-tail costs nothing
    // and the classification the poster spends is a fact this one has no use
    // for.
    if let Err(reach) = held {
        link.heard(&channel, Said::Unreachable(reach.said()));
    }
}

/// Read one frame and, when it is a tail, absorb it — answering the fold rather
/// than the append.
///
/// **Decoding happens here, on the lane's own thread**, not at the settle: this
/// is the one read whose frames arrive faster than an operator looks, and the
/// settle is the frame's side of the lock. Everything else a held read can
/// answer — a refusal mid-stream, bytes this build cannot read — crosses
/// untouched, because only the tail has anything to accumulate.
fn absorbed(fold: &mut Stream, frame: &serde_json::Value) -> Read {
    match crate::reply::read(frame) {
        Read::Answer(Reply::Follow(later)) => {
            fold.absorb(later);
            Read::Answer(Reply::Follow(fold.clone()))
        }
        other => other,
    }
}

/// Whether the window is still looking at what this read is about.
///
/// **Both halves**, because either can move: an operator who aims at another
/// wall has left this conversation as surely as one who picks another
/// conversation on the same wall.
fn still_on(link: &Link, aim: &crate::ui::Aim, conversation: &str) -> bool {
    let standing = link.standing();
    standing.aim.as_ref() == Some(aim) && standing.conversation.as_deref() == Some(conversation)
}

#[cfg(test)]
mod tests;
