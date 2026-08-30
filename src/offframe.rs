//! **The off-frame threads** — the asker, the poster and the follow lane.
//!
//! A frame that never blocks means no read and no act happens on one. These are
//! where they happen: three loops on three sockets, each talking to the window
//! only through [`Link`](crate::state::Link) — frames that landed one way,
//! gestures to send the other, and the standing question set published by the
//! frame that last settled.
//!
//! # Three, because they wait for different things
//!
//! The **asker** goes round the standing set at human cadence: every channel's
//! roster, the aimed wall's conversations, the selected conversation's
//! transcript. The **poster** sends what a click composed and files the receipt
//! it earns, on its own thread because an act must not wait behind a read that
//! is mid-pass. The **follow lane** holds one connection open on the focused
//! conversation and is answered as the tail moves, which is a read that
//! deliberately never finishes and must therefore never be in the serial pass.
//!
//! # Every pass is a function a test calls directly
//!
//! Each worker's body is `tick`, and a thread is [`pump`] around one. So the
//! suite drives a real pass against the stand-in engine with no thread at all,
//! and the one end-to-end beat is about the threading rather than about what
//! the passes do.
//!
//! **A stop is seen between passes, not during one.** The follow lane is the
//! case that matters: it is parked on a socket read, so it learns of a stop
//! when the engine writes, when the connection closes, or when the transport's
//! own read timeout expires. That is not a leak — closing the window ends the
//! process — and the alternative is a second signal path into a blocking read,
//! which is a mechanism for a case that has no consequence.

use std::path::{Path, PathBuf};
use std::thread::JoinHandle;

use crate::state::Link;

/// One pass of the standing reads.
pub mod asker;
/// One pass of the follow lane.
pub mod follow;
/// One pass of the outbox.
pub mod poster;

/// **Start the three threads.** The handles come back so a caller that stopped
/// the link can wait for the passes in flight to finish; a caller that does not
/// want to wait can drop them.
pub fn run(link: &Link, root: &Path) -> Vec<JoinHandle<()>> {
    let passes: [fn(&Link, &Path); 3] = [asker::tick, poster::tick, follow::tick];
    passes
        .into_iter()
        .map(|pass| {
            let link = link.clone();
            let root: PathBuf = root.to_path_buf();
            std::thread::spawn(move || pump(&link, || pass(&link, &root)))
        })
        .collect()
}

/// One worker's loop: pass, wait a beat, pass again, until the link says stop.
///
/// The wait is the **cadence** and not a timeout. A seat asks at the rate an
/// operator looks, and the two surfaces that move faster than that are held
/// reads rather than a faster loop.
pub fn pump(link: &Link, mut pass: impl FnMut()) {
    while !link.stopped() {
        pass();
        std::thread::sleep(link.beat());
    }
}

#[cfg(test)]
mod tests;
