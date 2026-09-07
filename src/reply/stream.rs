//! **The live tail** (yog's `docs/REMOTE.md` §3, §9.7) — the follow-class
//! read's answer, and the one reply whose N is greater than one.
//!
//! **A frame is an APPEND, and the seat holds the accumulation** (REMOTE §5.5,
//! PROTOCOL 2). One rule, with no flag and no case:
//!
//! > Absorb every frame of a read, in order, onto an empty fold. What you hold
//! > after the last frame you have received is what you paint.
//!
//! [`Stream::absorb`] is that operation, and it is the engine's own — the two
//! ends agree by contract (`fold(a).absorb(fold(b)) == fold(a ++ b)` on any
//! line boundary) rather than by coincidence. A read starts holding nothing
//! and the engine's reader opens at byte zero, so the FIRST frame of any read
//! is the whole tail so far: a seat that dropped a connection mid-answer
//! re-asks and is whole on its first frame, with nothing to reconcile. The
//! one-shot answer is the same rule with one frame in it.
//!
//! **The wire spelling did not move, which is the hazard.** The body is still
//! `{"delta", "text", "thinking"}`; what changed is that the two text fields
//! are the *appended* part rather than the accumulated one. Nothing in a field
//! signature can see that, so this paragraph is the record.
//!
//! **The frame carries the tool window beside the prose** (REMOTE §5.5,
//! PROTOCOL 15), by the same rule: concatenate the lists of a read's frames in
//! order. [`window`] holds it, and the one thing to know here is that the two
//! accumulate together — a frame is *what landed*, and what landed is some
//! prose, some window, or both.
//!
//! **Absence is a reading in all three prose fields.** `text` and `thinking` are
//! absent until a delta of that kind has landed — which is not the same claim
//! as an empty string — and `delta` is absent while the stream has produced
//! nothing at all, which under an open response file is exactly *waiting for
//! the API*. A seat that read those as empty strings would paint "the model
//! answered nothing" over "the model has not answered yet".

use serde_json::{Map, Value};

use super::fields;

/// The tool window the fold carries beside the prose.
pub mod window;

use window::Window;

/// This reply's kind token. One word for the query and the reply alike, on the
/// engine's side; this end only ever reads it.
pub(crate) const KIND: &str = "follow";

/// What one read of the tail says — every fact off one pass, so they cannot
/// describe two different mid-write states of one file.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Stream {
    /// The accumulated answer text — accumulated by [`absorb`](Self::absorb),
    /// on this end, out of what each frame appended.
    pub text: Option<String>,
    /// The accumulated reasoning, held apart from [`text`](Self::text) because
    /// they are two different things being said and the transcript paints them
    /// as two rows.
    pub thinking: Option<String>,
    /// The kind of the **last** delta seen — which of the two the model is
    /// doing right now.
    pub last_delta: Option<Delta>,
    /// **The tool window** (REMOTE §5.5): what ran, where, and how it ended.
    /// Folded into calls by [`window::fold`], so the two transitions the wire
    /// spends per call are one entry here.
    pub tools: Vec<Window>,
}

impl Stream {
    /// **Absorb the fold of what landed after this one's** — the whole of the
    /// §5.5 reassembly, and the engine's own operation (`Stream::absorb` in
    /// yog's `src/git_tree/streaming.rs`) rather than a second reading of the
    /// same bytes. Text accretes in stream order and the newer delta kind wins
    /// where the later frame had one at all, which is exactly
    /// `fold(a).absorb(fold(b)) == fold(a ++ b)` for any split on a line
    /// boundary. That equality is the contract: this end never needs a second
    /// parser, only a second *place* to start reading.
    pub fn absorb(&mut self, later: Self) {
        append(&mut self.text, later.text);
        append(&mut self.thinking, later.thinking);
        self.last_delta = later.last_delta.or(self.last_delta.take());
        for entry in later.tools {
            window::fold(&mut self.tools, entry);
        }
    }

    /// **Absorb one frame's append and answer what THAT FRAME said**, with
    /// every call it moved whole.
    ///
    /// It is the follower's half of REMOTE §5.5's window rule. A closing entry
    /// *"restates neither name nor input — key it by `tool_use`"*, so a
    /// surface that prints frames as they land — the command line, where a
    /// line already printed cannot be gone back to — would say `exit 0` about
    /// a call it could no longer name. The fold is the only thing that can
    /// name it, and this is the fold answering.
    ///
    /// The prose is the APPEND rather than the accumulation, because that is
    /// what such a surface prints; the window entries are the frame's own, each
    /// looked up whole. A surface that paints the accumulation — the window's
    /// transcript — absorbs and paints [`self`] instead.
    #[must_use]
    pub fn appended(&mut self, later: Self) -> Self {
        let moved: Vec<String> = later
            .tools
            .iter()
            .map(|entry| entry.tool_use.clone())
            .collect();
        let said = Self {
            text: later.text.clone(),
            thinking: later.thinking.clone(),
            last_delta: later.last_delta.clone(),
            tools: Vec::new(),
        };
        self.absorb(later);
        Self {
            tools: self
                .tools
                .iter()
                .filter(|call| moved.contains(&call.tool_use))
                .cloned()
                .collect(),
            ..said
        }
    }
}

/// Accrete `more` onto an accumulator that may not exist yet. **Absent stays
/// absent** — a stream that has said nothing has said nothing, and an empty
/// `Some("")` would read as *it spoke* to every surface downstream, which is
/// the distinction this module's third paragraph exists to keep.
fn append(slot: &mut Option<String>, more: Option<String>) {
    if let Some(more) = more {
        slot.get_or_insert_default().push_str(&more);
    }
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
        // Beside the fold rather than inside it, which is where the wire puts
        // it: the prose is what the model is saying and the window is what the
        // step is doing, and the two accumulate by the same rule.
        tools: window::windows(obj)?,
    })
}

#[cfg(test)]
mod tests;
