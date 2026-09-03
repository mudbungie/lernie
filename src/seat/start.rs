//! **The composite start**: both acts of yog's `docs/DESIGN.md` §8.1, spelled
//! as one word because a one-shot process can hold the first reply between
//! them.
//!
//! It is a **serialization and not a gesture** (REMOTE §3: *"one dispatch
//! surface, N serializations, never two implementations"*). Nothing new crosses
//! the wire: what goes out is [`crate::verbs::prepare`] and then
//! [`crate::verbs::prompt`], the boundary's own two envelopes, each down its
//! own connection because a seat dials per ask. The only thing this file adds
//! is the *local* between them — the staged body, held in a variable — which is
//! precisely what the window holds in its model.
//!
//! **Both streams are the product.** A start is two answers and an operator
//! wants to read both, so every frame of both prints, in order, exactly as one
//! ask's do. What the exit code says is the *second* act: a stage that answered
//! something other than a staged body exits non-zero with its frames on stdout,
//! because the frames are still the engine answering and the start still did
//! not happen.
//!
//! **A stage that lands and a fire that cannot be sent is its own sentence.** It
//! is the one outcome the two-act shape has and one act does not: the workspace
//! exists, the seed is spent, and nothing is running. Saying so beats printing a
//! transport error under a receipt that looks like success.
//!
//! **And the remedy is not one sentence, because the fire is an ACT** (REMOTE
//! §3, bl-3969). *Type it again* is right for a fire that never left this box —
//! the stage's steps are convergent (§8.1: *"steps are individually
//! idempotent-or-convergent"*) and the fire did not happen. It is exactly wrong
//! for a fire that crossed and was not answered: an act with no reply is IN
//! DOUBT, a second `lernie start` is a second conversation on a wall that may
//! already have one running, and the recovery is to LOOK. This file was the one
//! place in the crate that told an operator to resend an act, and it now tells
//! them which of the two happened and what the read is.

use std::path::Path;

use serde_json::Value;

use crate::cli::Verdict;
use crate::envelope;
use crate::reply::{Read, Reply, read};

/// What the seat says when the stage landed and the fire never left this box.
const UNFIRED: &str = "the start was staged and the fire could not be sent";

/// What it says when the fire crossed and no answer came back. **The one
/// sentence in this crate that has to refuse the obvious remedy**: the stage is
/// convergent and the fire is not, so a start that may already be running must
/// not be typed again (REMOTE §3).
const INDOUBT: &str = "the start was staged and the fire crossed with no answer, so it is IN DOUBT — \
     the conversation may be running. Do NOT start it again: ask what the wall \
     holds (`lernie conversations <workspace>`)";

/// **Begin a conversation**: stage a start in `address`, then fire it with
/// `goal`.
pub fn start(data_root: &Path, address: &str, goal: &str) -> Verdict {
    let staged = match super::sent(data_root, &crate::verbs::prepare(address.to_owned())) {
        Ok(frames) => frames,
        Err(reach) => return Verdict::failed(reach.said()),
    };
    let Some(prepared) = prepared(&staged) else {
        // The engine refused, answered a kind this build cannot read, or
        // terminated saying nothing. Its frames are the product either way, and
        // the exit code is what says no start happened — a stage that answered
        // `ok: true` and no staged body must not exit zero.
        return Verdict::answered(super::lines(&staged), false);
    };
    let fire = crate::verbs::prompt(&prepared, address.to_owned(), goal.to_owned());
    match super::sent(data_root, &fire) {
        Ok(fired) => Verdict::answered(
            super::lines(&[staged, fired.clone()].concat()),
            envelope::succeeded(&fired),
        ),
        Err(reach) if reach.crossed() => Verdict::failed(format!("{INDOUBT}: {}", reach.said())),
        Err(reach) => Verdict::failed(format!("{UNFIRED}: {}", reach.said())),
    }
}

/// The staged body, when the stage's last frame is one.
///
/// **The last frame**, for [`envelope::succeeded`]'s own reason: every answer
/// is a stream and its newest frame is its state.
fn prepared(staged: &[Value]) -> Option<crate::reply::start::Prepared> {
    match staged.last().map(read) {
        Some(Read::Answer(Reply::Prepared(prepared))) => Some(prepared),
        _ => None,
    }
}

#[cfg(test)]
mod tests;
