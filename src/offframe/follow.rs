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

use crate::state::{Link, Said};

/// Hold the line on the focused conversation until it moves or the engine ends
/// the stream.
pub fn tick(link: &Link, root: &Path) {
    let standing = link.standing();
    let (Some((channel, aim)), Some(conversation)) = (standing.aimed(), standing.conversation)
    else {
        return;
    };
    let envelope = crate::verbs::follow(aim.address.clone(), conversation.clone());
    let held = crate::seat::route(root, &envelope).and_then(|(open, carried)| {
        open.follow(&carried, &mut |frame| {
            link.live(&channel, &conversation, frame);
            !link.stopped() && still_on(link, &aim, &conversation)
        })
    });
    if let Err(why) = held {
        link.heard(&channel, Said::Unreachable(why));
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
