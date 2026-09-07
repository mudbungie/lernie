//! **The rendering**: what a reply looks like to a person (DESIGN §4.36).
//!
//! The boundary is JSON and stays JSON — that is REMOTE §3's whole shape, and
//! `--json` prints it byte for byte. What this module adds is the other half of
//! being a seat: **the part you look at**. A day of CLI use was a day of piping
//! into a JSON formatter, because the machine form was the only form (bl-6ae7);
//! the seat's own siblings set the standard the other way round, `bl list`
//! rendering and `bl list --json` being the machine form.
//!
//! # It is one rendering per reply KIND, never per verb
//!
//! That is the ruling (round-1 triage, ruling 4) and it is also the only shape
//! that stays one implementation. A verb is a serialization of a gesture
//! ([`crate::verbs`], DESIGN §4.10); its ANSWER is a kind of
//! [`crate::reply::Reply`], and several verbs answer with one kind — `attention`
//! and `seen` both answer a queue, `fleet`, `arm`, `disarm` and `disband` all
//! answer whether something stands. A render per verb would be a second table
//! keyed on the wrong fact and it would disagree with itself within a week.
//!
//! # It is a reading, and the frame is always one act away
//!
//! A rendering paints the facts a person scans a listing for, and elides the
//! rest: an entry's `raw` bytes, a preview's third paragraph, a step's whole
//! captured document. That is a deliberate loss and it is safe precisely
//! because it is not the only form — every one of these lines has a `--json`
//! beside it that is the frame exactly as it crossed. What must never happen is
//! the other direction: a rendering that states something the frame does not.
//!
//! # Where it hangs
//!
//! [`said`] is the one place this seat's product is written, so the rendering
//! reaches every surface that prints a reply stream at once — one gesture
//! ([`crate::seat::ask`]), a fan across channels ([`crate::seat::fanned`]) and
//! the composite start's two streams ([`crate::seat::start`]).

use serde_json::Value;

use crate::reply::stream::Stream;
use crate::reply::{Read, Reply, read};

pub(crate) use walls::{at_rest, held_at, waiting_on_mail};

/// The receipts and the runs — what an ACT answers with.
mod acts;
/// The per-kind renderings, in the one match that dispatches them.
mod answer;
/// The conversation, as prose: its transcript and its own whole row.
mod chat;
/// The vocabulary every rendering is built from.
mod parts;
/// What a wall's policy is written in: roles, lineages, config, clients.
mod policy;
/// The three reads whose subject is an engine: its words, a search, the trail.
mod reads;
/// The conversation's own records, and the files a wall's policy is in.
mod records;
/// One step, drilled into — its records, its tool calls and its logs.
mod step;
/// The balls, the board, the fleet and what a wall's agents changed.
mod tasks;
/// The roster, the conversation list, the queue, the transcript and the tail.
mod walls;
/// What a wall's agents changed, as a diff rather than a row of counters.
mod work;

/// **Which form a reply stream is printed in.**
///
/// Two, and there will never be a third: one for a person and one for a
/// machine. It is decided by argv alone — nothing about the channel, the
/// gesture or the answer can change it — which is why it is a value
/// [`crate::cli::run`] hands over rather than a field on anything.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Form {
    /// What a person reads. The default, because the default reader is a
    /// person.
    #[default]
    Rendered,
    /// The frames exactly as they crossed, one per line.
    Json,
}

/// **The reply stream as this seat's product** — the one place that shape is
/// written, whichever form it takes.
pub fn said(stream: &[Value], form: Form) -> String {
    stream
        .iter()
        .map(|frame| match form {
            Form::Json => frame.to_string(),
            Form::Rendered => rendered(frame),
        })
        .collect::<Vec<String>>()
        .join("\n")
}

/// **One frame of a HELD read**, rendered against the fold the follower holds.
///
/// Every other surface renders a frame alone, because every other frame stands
/// alone. A follow frame does not: it is an append, and the closing half of a
/// tool window *"restates neither name nor input — key it by `tool_use`"*
/// (REMOTE §5.5). So a caller printing frames as they land — the one surface
/// that cannot go back and amend a line — hands its fold in, and what comes
/// back is this frame's own append with every call in it named.
///
/// It is the one entry point here that takes state, and the state is the
/// caller's: one read is one fold, and the fold's whole lifetime is that call
/// (`crate::seat::follow`, `crate::offframe::follow`).
pub fn tail(frame: &Value, fold: &mut Stream, form: Form) -> String {
    match (form, read(frame)) {
        (Form::Json, _) => frame.to_string(),
        (Form::Rendered, Read::Answer(Reply::Follow(later))) => {
            answer::answer(&Reply::Follow(fold.appended(later)))
        }
        (Form::Rendered, other) => painted(other),
    }
}

/// One frame, read and rendered. **Read here rather than by the caller**: a
/// rendering is a statement about what a frame turned out to be, and
/// [`crate::reply::read`] is the one thing that can say.
fn rendered(frame: &Value) -> String {
    painted(read(frame))
}

/// What a reading looks like to a person — the three answers [`Read`] has, and
/// the one place each is worded.
fn painted(answered: Read) -> String {
    match answered {
        Read::Answer(reply) => answer::answer(&reply),
        // The engine's own sentence, unadorned. It is the answer to what was
        // asked, so it is printed rather than interpreted — this seat has
        // nothing to add to a wall saying no.
        Read::Refusal(why) => format!("refused: {why}"),
        // A statement about THIS SEAT, and the upgrade prompt (DESIGN §4.9's
        // rung 2). The frame is still on the wire and `--json` still prints it,
        // which is what makes the sentence actionable rather than a dead end.
        Read::Unreadable(why) => {
            format!("this seat cannot read that answer: {why} (`--json` prints the frame)")
        }
    }
}

#[cfg(test)]
mod tests;
