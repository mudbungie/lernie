//! **When a watch stops, and what it says then** (bl-3dca, bl-87ab, bl-3ecd,
//! bl-3a1f) — the readings [`super::follow`] decides on, and the two sentences
//! an ending can be.
//!
//! Split from [`super`] at the 300-line cap on the seam the word already has:
//! that file is the LOOP — ask what it is doing, hold a read, print what
//! lands, ask again — and this is when to stop and what to say, which is the
//! half that has grown three times and the only half with a remedy to name.
//!
//! # A HELD conversation is not at rest either (bl-3a1f)
//!
//! The same mistake as the mail below, one noun over. The capability control
//! parks a tool call *before* the executor is entered and the conversation's
//! state read then says `quiescent`, so this word answered *"nothing more will
//! arrive until it is nudged or messaged"* — false in both halves. Something
//! more will arrive the moment the operator answers, and neither remedy it
//! named is that answer. It said it at the exact moment the operator reading
//! the line was the thing the conversation was waiting for, and on a foot lane
//! every call to a non-shell tool is held, which makes it most of the endings.
//!
//! The fact was one field away the whole time: the `agent` row this loop
//! already reads to decide rest carries the hold mark, the same three facts
//! `reply/follow`'s parked entry carries. So a park is a distinct KIND of rest
//! — the watch still ends, because nothing is going to happen until the
//! operator acts — and the line names the call, the control's reason and
//! `answer`, which is the gesture that lifts it.
//!
//! # Mail waiting is not rest (bl-87ab, bl-3ecd)
//!
//! `message` (or `start`) then `follow` is the pair every operator types, and
//! it answered *"nothing more will arrive until it is nudged or messaged"* in
//! zero seconds — about a conversation that took a lease twelve seconds later
//! and then ran for another seventy-eight. The deposit had landed and no
//! driver held the inbox lock yet, so the state read was correct as of that
//! instant and the sentence it produced was false: something more was going to
//! arrive, and nothing further was going to be asked of the operator.
//!
//! The engine has no field for *about to start* and inventing one here would
//! be a guess. What it does have is the deposit itself: **an inbox with mail in
//! it is a turn that has not begun**, so the state read alone was never enough
//! and the second read is what makes the answer true. So a rest with mail
//! waiting is not an ending — the watch holds, exactly as it holds on a live
//! one, and says once that it is waiting so the hold never reads as a hang.
//! Ctrl-C is the way out it always was.
//!
//! **It costs one read, and only on the path that was wrong.** A conversation
//! genuinely at rest has an empty inbox and answers as fast as it ever did;
//! nothing is asked before the state read, and nothing extra is asked while a
//! conversation is working. And a mail count this seat could not READ is
//! treated as no mail: the state read has already said rest, and a probe that
//! failed is not grounds for holding somebody's connection open forever.
//!
//!
//! # Mail waiting is not rest, and neither is a park
//!
//! Both are the same correction: the state read is true as of the instant it
//! was taken and the sentence it produced was false. A deposit that no driver
//! has taken is a turn that has not begun; a parked call is a turn that cannot
//! begin until the operator answers. The first holds the line, because
//! something is coming with nothing asked of the operator; the second ends it,
//! because nothing is coming until they act. One reading each, and the
//! difference between them is who the conversation is waiting for.

use std::path::Path;
use std::time::Duration;

use serde_json::Value;

use crate::render::{Form, said};
use crate::reply::convs::AgentState;
use crate::reply::{Read, Reply, read};

/// What the standing read says the conversation is doing.
pub(super) enum Rest {
    /// A driver is on it, or it is streaming. Hold the line.
    Working,
    /// It has come to rest, in the engine's own word for how.
    At(String),
    /// It is at rest **because a tool call is parked at the capability
    /// boundary**, which is a rest that waits on the operator rather than on
    /// nothing (bl-3a1f). Read off the same row the state is.
    Parked(crate::reply::queue::Held),
    /// The engine answered something no state can be read out of.
    Unreadable,
}

/// Read the conversation's own state off its row.
pub(super) fn resting(standing: &[Value]) -> Rest {
    let Some(Read::Answer(Reply::Agent(row))) = standing.last().map(read) else {
        return Rest::Unreadable;
    };
    // **One match over the pair the row already carries**, so the three
    // readings are three arms and none of them is unreachable: a driver on the
    // lease is working whatever else is true of the row, a settled row with a
    // hold mark is PARKED, and a settled row without one is at rest in the
    // engine's own word for how. Reading the state first is what keeps a
    // working conversation working — a driver holding the lease with a call
    // parked under it is still advancing, and the line that names the park is
    // the one the watch prints when it settles.
    match (row.state, row.held) {
        (AgentState::Live | AgentState::InFlight, _) => Rest::Working,
        (_, Some(held)) => Rest::Parked(held),
        (settled, None) => Rest::At(settled.label()),
    }
}

/// **How much mail is waiting to be taken**, which is what tells a rest that
/// is about to end from one that is not.
///
/// **Unreadable is NO mail, deliberately.** The state read above has already
/// said this conversation is at rest, so every way this probe can fail — a
/// channel that dropped, a refusal, an answer of a kind this build does not
/// paint — leaves the watch with exactly the reading it had before the probe
/// existed. Holding a connection open forever on the strength of a question
/// that was not answered is the one outcome worse than the sentence this
/// exists to fix.
pub(super) fn waiting(data_root: &Path, mail: &Value) -> usize {
    let Ok(frames) = super::super::sent(data_root, mail) else {
        return 0;
    };
    match frames.last().map(read) {
        Some(Read::Answer(Reply::Inbox(rows))) => rows.len(),
        _ => 0,
    }
}

/// **The line a watch ends on**, in the form the caller asked for.
///
/// Rendered, it is this seat's own sentence, said where every other line of
/// its product is said ([`crate::render::at_rest`]) — and how long the watch
/// held is the one fact only this end has, so it is measured here and rendered
/// there. As JSON it is the `agent` frame that ended the watch, exactly as it
/// crossed: `--json` is the frame stream and this seat adds nothing to it
/// (bl-87ab).
pub(super) fn ending(
    standing: &[Value],
    agent: &str,
    state: &str,
    held: Duration,
    form: Form,
) -> String {
    match form {
        Form::Json => said(standing, form),
        Form::Rendered => crate::render::at_rest(
            agent,
            state,
            i64::try_from(held.as_secs()).unwrap_or(i64::MAX),
        ),
    }
}

/// **The line a watch ends on when a call is parked**, in the form the caller
/// asked for — [`ending`]'s shape exactly, and split from it rather than
/// folded into it because the two say different things and only one of them
/// has a remedy to name.
///
/// `--json` is unchanged and says nothing of this seat's own: what ended the
/// watch is the `agent` frame that ended it, printed as it crossed, and that
/// frame already carries the hold mark this sentence is made of (bl-87ab).
pub(super) fn parked(
    standing: &[Value],
    agent: &str,
    workspace: &str,
    held: &crate::reply::queue::Held,
    waited: Duration,
    form: Form,
) -> String {
    match form {
        Form::Json => said(standing, form),
        Form::Rendered => crate::render::held_at(
            agent,
            workspace,
            held,
            i64::try_from(waited.as_secs()).unwrap_or(i64::MAX),
        ),
    }
}
