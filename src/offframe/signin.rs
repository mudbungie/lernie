//! **The sign-in lane**: one connection held open on the row the login pane is
//! following (REMOTE §8.3, bl-e3c5).
//!
//! It is a thread of its own for [`super::follow`]'s reason verbatim, and the
//! reason is sharper here: a sign-in is **minutes of a human's attention**, so
//! this is the read in this seat whose N takes longest to finish. Putting it in
//! the serial pass would stall the roster, the conversation list and the
//! transcript behind a browser somebody has not got to yet.
//!
//! **Every frame says which row it is about, and the frame decides whether it
//! is still wanted.** A second sign-in on a live pair terminates and replaces
//! the first upstream, so a frame of the old run that arrived after the
//! operator moved on would paint one run's lines under another's name. The
//! guard is a pure comparison at the settle — where what the pane is following
//! is known for certain — rather than a poll here racing the socket.
//!
//! **Closing the pane ends the lane and terminates nothing.** The run is engine
//! RAM, one per workspace × provider, bounded by its own hour sweep (REMOTE
//! §8.3): hanging up is this end saying it has stopped watching, and a run with
//! no lane at all still settles and still writes its `ops.jsonl` row. That is a
//! property this side gets for free and must not re-implement.

use std::path::Path;

use crate::reply::login::Signin;
use crate::reply::{Read, Reply};
use crate::state::{Link, Said};

/// Hold the line on the followed row's sign-in until it moves or the engine
/// ends the stream.
///
/// **The fold's whole lifetime is this call**, exactly as the tail's is: one
/// read is one `tick`, so a lane that dropped re-opens on an empty fold and the
/// engine's own cursor starts at zero — which is why *re-ask replays* needs no
/// rule of its own at either end.
pub fn tick(link: &Link, root: &Path) {
    let standing = link.standing();
    let (Some((channel, aim)), Some(provider)) = (standing.aimed(), standing.signin()) else {
        return;
    };
    let envelope = crate::verbs::login_tail(aim.address.clone(), provider.clone());
    let mut fold = Signin::default();
    let held = crate::seat::route(root, &envelope)
        .map_err(crate::channel::Reach::Unsent)
        .and_then(|(open, carried)| {
            open.follow(&carried, &mut |frame| {
                link.signing(&channel, &provider, absorbed(&mut fold, &frame));
                !link.stopped() && still_on(link, &aim, &provider)
            })
        });
    // **A held read is still a read** (REMOTE §3): the lane re-opens on the
    // next pass, so a connection that died mid-flow costs nothing but the
    // lines already painted, which the replay brings back.
    if let Err(reach) = held {
        link.heard(&channel, Said::Unreachable(reach.said()));
    }
}

/// Read one frame and, when it is the run, absorb it — answering the fold
/// rather than the append.
///
/// **Decoding happens here, on the lane's own thread**, for `super::follow`'s
/// reason: this is a read whose frames arrive at the provider's pace rather
/// than the asker's, and the settle is the frame's side of the lock.
fn absorbed(fold: &mut Signin, frame: &serde_json::Value) -> Read {
    match crate::reply::read(frame) {
        Read::Answer(Reply::Login(later)) => {
            fold.absorb(later);
            Read::Answer(Reply::Login(fold.clone()))
        }
        other => other,
    }
}

/// Whether the window is still following this run.
///
/// **Both halves**, because either can move: an operator who aims at another
/// wall has left this sign-in as surely as one who starts another on the same
/// wall — and aiming away retires the pane outright
/// (`crate::ui::Model::retire_login`), which is the same answer reached by the
/// other field.
fn still_on(link: &Link, aim: &crate::ui::Aim, provider: &str) -> bool {
    let standing = link.standing();
    standing.aim.as_ref() == Some(aim) && standing.signin().as_deref() == Some(provider)
}

#[cfg(test)]
mod tests;
