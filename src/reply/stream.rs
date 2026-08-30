//! **The live tail** (yog's `docs/REMOTE.md` §3, §9.7) — the follow-class
//! read's answer, and the one reply whose N is greater than one.
//!
//! **A frame replaces what a seat holds; it is never a delta.** The engine
//! answers the whole accumulated fold every time, so nothing has to be
//! reassembled, a seat that missed a frame has missed nothing, and the follow
//! lane needs no second parser beside the pull read's. That is also why the
//! same fold is what a transcript's live entry is built from: one value, said
//! one way, arriving at two cadences.
//!
//! **Absence is a reading in all three fields.** `text` and `thinking` are
//! absent until a delta of that kind has landed — which is not the same claim
//! as an empty string — and `delta` is absent while the stream has produced
//! nothing at all, which under an open response file is exactly *waiting for
//! the API*. A seat that read those as empty strings would paint "the model
//! answered nothing" over "the model has not answered yet".

use serde_json::{Map, Value};

use super::fields;

/// This reply's kind token. One word for the query and the reply alike, on the
/// engine's side; this end only ever reads it.
pub(crate) const KIND: &str = "follow";

/// What one read of the tail says — every fact off one pass, so they cannot
/// describe two different mid-write states of one file.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Stream {
    /// The accumulated answer text.
    pub text: Option<String>,
    /// The accumulated reasoning, held apart from [`text`](Self::text) because
    /// they are two different things being said and the transcript paints them
    /// as two rows.
    pub thinking: Option<String>,
    /// The kind of the **last** delta seen — which of the two the model is
    /// doing right now.
    pub last_delta: Option<Delta>,
}

/// Which kind of content the last delta carried.
///
/// **Rung 3, and this is a deliberate divergence from the engine's own
/// reader**, which refuses an unrecognised token here. The two readers are not
/// doing the same job: the engine's decodes bytes the engine wrote, so a
/// mismatch there means its own codec has drifted and refusing is the
/// diagnosis. This one is the last reader of somebody else's answer, and
/// refusing would throw away an entire accumulated turn to avoid painting one
/// word — the silent drop [`super`]'s policy exists to exclude, at the worst
/// possible moment, which is while the operator is watching the tail move.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Delta {
    /// Answer text.
    Text,
    /// Reasoning.
    Thinking,
    /// A word this build does not know, verbatim.
    Unknown(String),
}

impl Delta {
    /// The word the live mark paints.
    pub fn label(&self) -> String {
        match self {
            Self::Text => TEXT.to_owned(),
            Self::Thinking => THINKING.to_owned(),
            Self::Unknown(word) => word.clone(),
        }
    }
}

const TEXT: &str = "text";
const THINKING: &str = "thinking";
/// The key the fold rides under inside the reply envelope.
const STREAM: &str = "stream";

/// Read the reply: the envelope, then the fold inside it.
pub(crate) fn follow(obj: &Map<String, Value>) -> Result<Stream, String> {
    let body = obj
        .get(STREAM)
        .and_then(Value::as_object)
        .ok_or_else(|| format!("missing or non-object field {STREAM:?}"))?;
    Ok(Stream {
        text: fields::opt_text(body, TEXT)?,
        thinking: fields::opt_text(body, THINKING)?,
        last_delta: fields::opt_text(body, "delta")?.map(|word| match word.as_str() {
            TEXT => Delta::Text,
            THINKING => Delta::Thinking,
            _ => Delta::Unknown(word.clone()),
        }),
    })
}

#[cfg(test)]
mod tests;
