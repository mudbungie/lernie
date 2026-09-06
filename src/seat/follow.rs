//! **Holding the line on one conversation** — the seat's own word for
//! watching work happen, which is more than one gesture.
//!
//! # What was wrong, and both halves were the same mistake
//!
//! `follow` was one `ask`: one held connection, its frames collected, the
//! stream printed at the end, and the exit code read off the last frame. That
//! gave two defects with one cause.
//!
//! - **On a quiescent conversation it blocked 31 seconds, printed one newline
//!   and exited 1** (bl-3dca). Nothing is wrong with a conversation at rest —
//!   it is the ordinary state of one an operator has just looked at — but exit
//!   1 with no output is byte for byte what a failed dial, a refused
//!   certificate and a dead engine look like, and those are what a first-time
//!   user suspects.
//! - **On a live one it returned mid-turn, every time** (bl-f076). The engine
//!   ends the stream at the STEP boundary (yog's REMOTE §5.1), not at the
//!   turn's, so three consecutive reads of one conversation ended after 3, 8
//!   and 6 seconds while it worked on. Watching an agent was
//!   `while true; do lernie follow …; done`, which the operator had to invent
//!   and which loses whatever landed between two reads.
//!
//! # The rule this file implements
//!
//! **Hold the line until the conversation rests, or until the user quits.**
//! The engine's own step boundary is not the end of anything an operator
//! cares about, so a read that ends there is re-asked; what ends the WORD is
//! the conversation coming to rest, and that is a fact the engine already
//! answers — `agent`'s `state`. So the loop is: ask what it is doing, and if
//! it is not working, say so and exit 0; otherwise hold a read and print what
//! lands, then ask again.
//!
//! **The state read comes first, which is what fixes the quiescent case at no
//! cost.** A conversation already at rest never opens a held connection at
//! all, so the half-minute of silence is not shortened — it never happens.
//!
//! **An unknown state ends the follow rather than looping on it.** The reply
//! vocabulary paints an unrecognised token as itself (DESIGN §4.9 rung 3), and
//! the safe reading here is *this seat does not know that this is working* —
//! ending, and printing the word, beats holding a connection open forever on a
//! state nobody here understands.
//!
//! # Why the frames are handed to a sink
//!
//! A held read that printed only at the end would not be a follow. So this is
//! the one place in the crate where the product is written as it arrives, and
//! the writing stays the entry point's: [`follow`] takes a sink, `src/main.rs`
//! hands it a printer, and the suite hands it a `Vec`. The decision is still
//! entirely in the library, where a test reads it back.

use std::path::Path;
use std::time::Duration;

use serde_json::Value;

use crate::channel::Reach;
use crate::cli::Verdict;
use crate::render::{Form, said};
use crate::reply::convs::AgentState;
use crate::reply::{Read, Reply, read};

/// **How long a read that brought nothing waits before asking again.**
///
/// The engine paces this loop on its own — a held read stays open while the
/// tail is quiet — so this is insurance against an engine that closes an empty
/// read at once, where the alternative is a busy loop on somebody else's
/// socket. It is not a poll interval: a read that brought a frame asks again
/// immediately.
const SETTLE: Duration = Duration::from_millis(250);

/// **Watch one conversation until it rests.** `say` takes each frame's
/// rendering as it lands; the verdict is the sentence that ends the watch.
pub fn follow(
    data_root: &Path,
    workspace: &str,
    agent: &str,
    form: Form,
    say: &mut dyn FnMut(&str),
) -> Verdict {
    let asking = crate::verbs::agent(workspace.to_owned(), agent.to_owned());
    let holding = crate::verbs::follow(workspace.to_owned(), agent.to_owned());
    loop {
        let standing = match super::sent(data_root, &asking) {
            Ok(frames) => frames,
            Err(reach) => return Verdict::failed(reach.said()),
        };
        match resting(&standing) {
            // The engine answered something this build cannot read a state
            // out of — a refusal, or a kind it does not paint. That answer is
            // the product and the exit code is what says the watch never
            // started.
            Rest::Unreadable => return Verdict::answered(said(&standing, form), false),
            Rest::At(state) => return Verdict::ok(rested(agent, &state)),
            Rest::Working => {}
        }
        match held(data_root, &holding, form, say) {
            Err(reach) => return Verdict::failed(reach.said()),
            Ok(0) => std::thread::sleep(SETTLE),
            Ok(_) => {}
        }
    }
}

/// One held read, printed as it arrives. Answers how many frames landed, which
/// is the one fact the loop above needs from it.
fn held(
    data_root: &Path,
    envelope: &Value,
    form: Form,
    say: &mut dyn FnMut(&str),
) -> Result<usize, Reach> {
    let (channel, carried) = super::route(data_root, envelope)
        .sent
        .map_err(Reach::Unsent)?;
    let mut heard = 0usize;
    channel.follow(&carried, &mut |frame| {
        say(&said(std::slice::from_ref(&frame), form));
        heard += 1;
        true
    })?;
    Ok(heard)
}

/// What the standing read says the conversation is doing.
enum Rest {
    /// A driver is on it, or it is streaming. Hold the line.
    Working,
    /// It has come to rest, in the engine's own word for how.
    At(String),
    /// The engine answered something no state can be read out of.
    Unreadable,
}

/// Read the conversation's own state off its row.
fn resting(standing: &[Value]) -> Rest {
    let Some(Read::Answer(Reply::Agent(row))) = standing.last().map(read) else {
        return Rest::Unreadable;
    };
    match row.state {
        AgentState::Live | AgentState::InFlight => Rest::Working,
        settled => Rest::At(settled.label()),
    }
}

/// **The line a watch ends on.** It says which conversation, what state it
/// came to rest in, and what makes it move again — because a watch that simply
/// stopped is the silence this word was fixed for.
fn rested(agent: &str, state: &str) -> String {
    format!(
        "{agent} is at rest ({state}) — nothing more will arrive until it is \
         nudged or messaged"
    )
}

#[cfg(test)]
mod tests;
