//! In-memory assembly of brazen `v=1` canonical events (ARCH §4.4).
//!
//! One `bz` process per attempt streams its stdout as NDJSON canonical
//! events (`brazen::Event`). Each line is appended to `response.json`
//! for diagnostics (§2.3) **and** fed through this module to build the
//! *in-memory* shape the next step's request needs:
//!
//! - `content_delta` text fragments accumulate into a [`Content::Text`];
//! - a `tool_use` block folds its `json_delta` fragments into one JSON
//!   string, parsed once on stop into the wire `input` value;
//! - `finish` carries the terminal reason (`tool_use` drives another
//!   loop iteration, §2.5; anything else terminates the step);
//! - an in-band `error` event carries the [`CanonicalError`] the retry
//!   loop classifies (§2.10);
//! - the first `message_start`'s `v` is the schema-handshake the adapter
//!   override path checks (§4.4).
//!
//! Per the diagnostic-only contract (§2.3) the harness never reads
//! `response.json` back at runtime. This accumulator is the shipped
//! path that assembles the *next request*; the same adapter pass also
//! feeds the transcript writer's staging sink ([`super::staging`], §2.3),
//! which commits the durable `messages/NNN-assistant.json` entry. Both
//! are written off one pass with no read-back; re-pointing request
//! assembly at the committed transcript (retiring this accumulator) is
//! bl-26cb.

use brazen::{CanonicalError, Content, ContentKind, Delta, Event, FinishReason};

/// Accumulator over one attempt's event stream. [`Self::into_outcome`]
/// consumes the final state and classifies the segment.
pub(super) struct Assembler {
    blocks: Vec<Option<BlockState>>,
    finish: Option<FinishReason>,
    error: Option<CanonicalError>,
    handshake_v: Option<u8>,
    ended: bool,
}

/// One in-flight content block. Tool-use blocks hold the raw joined
/// `json_delta` buffer and parse on completion — invalid JSON surfaces
/// as [`AssemblyError::ToolInputJson`]. `Other` covers thinking and any
/// forward-compat kind: not replayed into the next request in v1.
enum BlockState {
    Text(String),
    ToolUse {
        id: String,
        name: String,
        json: String,
    },
    Other,
}

/// Assembled message ready to feed back into the step loop.
#[derive(Debug)]
pub(super) struct Completion {
    pub(super) content: Vec<Content>,
    finish: Option<FinishReason>,
    handshake_v: Option<u8>,
}

impl Completion {
    /// A `Finish{ToolUse}` drives another loop iteration (§2.5); any
    /// other terminal reason ends the step.
    pub(super) fn is_tool_use(&self) -> bool {
        matches!(self.finish, Some(FinishReason::ToolUse))
    }

    /// The schema version stamped on the first `message_start`, if one
    /// arrived. The adapter-override handshake (§4.4) checks this equals
    /// `brazen::EVENT_SCHEMA_VERSION`.
    pub(super) fn handshake_v(&self) -> Option<u8> {
        self.handshake_v
    }
}

/// How one attempt's segment settled.
#[derive(Debug)]
pub(super) enum SegmentOutcome {
    /// Trailing `end`, no `error` — the model call completed (§4.4).
    Complete(Completion),
    /// An in-band `error` event — the attempt failed. The retry loop
    /// classifies retryability via [`CanonicalError::retryable`] (§2.10).
    Failed(CanonicalError),
    /// The stream ended without a trailing `end` — the writer died
    /// mid-stream (§2.9 kill signature).
    HalfStream,
}

/// A tool_use block's accumulated `json_delta` did not parse.
#[derive(Debug)]
pub(super) struct AssemblyError(pub(super) serde_json::Error);

impl Assembler {
    pub(super) fn new() -> Self {
        Self {
            blocks: Vec::new(),
            finish: None,
            error: None,
            handshake_v: None,
            ended: false,
        }
    }

    /// Fold one event into the accumulator. Events after `end` are
    /// ignored (defensive — brazen emits exactly one terminator).
    pub(super) fn feed(&mut self, event: Event) {
        if self.ended {
            return;
        }
        match event {
            Event::MessageStart { v, .. } => self.handshake_v = Some(v),
            Event::ContentStart { index, kind } => {
                set_slot(&mut self.blocks, index as usize, initial_block(kind));
            }
            Event::ContentDelta { index, delta } => self.on_delta(index as usize, delta),
            Event::Finish { reason } => self.finish = Some(reason),
            Event::Error(err) => self.error = Some(err),
            Event::End => self.ended = true,
            // `ContentStop`, `Usage`, `Raw`, `Other`, and any
            // forward-compat variant carry nothing the in-memory shape
            // needs (§4.4 v=1 contract): tokens/framing ride to disk via
            // the JSONL, never re-surfaced here.
            _ => {}
        }
    }

    fn on_delta(&mut self, index: usize, delta: Delta) {
        match (self.blocks.get_mut(index), delta) {
            (Some(Some(BlockState::Text(buf))), Delta::TextDelta(t)) => buf.push_str(&t),
            (Some(Some(BlockState::ToolUse { json, .. })), Delta::JsonDelta(t)) => {
                json.push_str(&t)
            }
            // Thinking deltas, forward-compat deltas, and deltas for a
            // block we don't model are dropped (§4.4 v=1 contract).
            _ => {}
        }
    }

    pub(super) fn into_outcome(self) -> Result<SegmentOutcome, AssemblyError> {
        if let Some(err) = self.error {
            return Ok(SegmentOutcome::Failed(err));
        }
        if !self.ended {
            return Ok(SegmentOutcome::HalfStream);
        }
        let mut content = Vec::new();
        for state in self.blocks.into_iter().flatten() {
            if let Some(block) = finalize_block(state)? {
                content.push(block);
            }
        }
        Ok(SegmentOutcome::Complete(Completion {
            content,
            finish: self.finish,
            handshake_v: self.handshake_v,
        }))
    }
}

fn initial_block(kind: ContentKind) -> BlockState {
    match kind {
        ContentKind::Text {} => BlockState::Text(String::new()),
        ContentKind::ToolUse { id, name } => BlockState::ToolUse {
            id,
            name,
            json: String::new(),
        },
        _ => BlockState::Other,
    }
}

fn finalize_block(state: BlockState) -> Result<Option<Content>, AssemblyError> {
    Ok(match state {
        BlockState::Text(text) => Some(Content::Text(text)),
        BlockState::ToolUse { id, name, json } => {
            let input = if json.is_empty() {
                serde_json::Value::Object(serde_json::Map::new())
            } else {
                serde_json::from_str(&json).map_err(AssemblyError)?
            };
            Some(Content::ToolUse { id, name, input })
        }
        // Thinking / forward-compat blocks are not replayed in v1.
        BlockState::Other => None,
    })
}

fn set_slot(blocks: &mut Vec<Option<BlockState>>, index: usize, state: BlockState) {
    if blocks.len() <= index {
        blocks.resize_with(index + 1, || None);
    }
    blocks[index] = Some(state);
}

#[cfg(test)]
mod tests;
