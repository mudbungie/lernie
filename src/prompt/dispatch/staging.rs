//! The transcript writer's staging sink (ARCH §2.3 *The transcript
//! writer*, §4.4 segment authority).
//!
//! Both sinks of a model call are streams off the one adapter pass:
//! every event appends verbatim to the diagnostic `response.json`
//! (§4.4), and *content and usage* additionally stream here — the
//! model-output transcript entry under construction, whose shape has its
//! one home in [`entry`]: brazen's canonical [`Content`] blocks written
//! **block-by-block as each content block completes** (`content_stop`),
//! with the provider's token usage as their sibling. A block is the
//! smallest unit that exists before it is whole, so the in-progress
//! block's deltas are the only buffering anywhere (§2.3): a completed
//! block is flushed to disk and dropped from memory at its
//! `content_stop`. Usage is the one exception the wire forces — the
//! counters arrive across a segment's `usage` events and the sealed
//! object states them once (§2.3 *Usage rides the entry*, [`entry::UsageReport`]), so the
//! report is per-segment state beside the in-progress block.
//!
//! Segment authority (§4.4) drives the file mechanically:
//!
//! - an `Error`-terminated segment [`truncate_segment`]s — the
//!   discarded attempt's audit home is `response.json`;
//! - a `Pause`-terminated segment is left un-truncated, so its blocks
//!   *accumulate* and the continuation (§2.10) resumes past them,
//!   reading the writer's own sink — never a diagnostic record;
//! - the final `Finish` [`seal`]s the entry object — closing the array and
//!   writing the authoritative segment's usage report — ready to rename
//!   into the worktree as `messages/NNN-<model-id>.json` (§2.3).
//!
//! Nothing here reads `response.json` back: the staging file is the
//! writer's own sink, so the diagnostic-only contract (§2.3) is
//! untouched.
//!
//! [`truncate_segment`]: StagingWriter::truncate_segment
//! [`seal`]: StagingWriter::seal

use super::entry::{self, UsageReport};
use crate::prompt::Error;
use brazen::{Content, ContentKind, Delta, Event};
use std::fs::File;
use std::io::{Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};

/// The single content block being streamed. Only ever one is in RAM
/// (§2.3 — "the in-progress block's deltas are the only buffering
/// anywhere"); it is finalized to disk and dropped at `content_stop`.
enum Block {
    Text(String),
    /// Thinking text. The v=1 stream carries no signature (brazen drops
    /// the non-canonical `signature_delta`), so the sealed block is a
    /// `signature: None` thinking block — the faithful capture of what
    /// the adapter emits.
    Thinking(String),
    ToolUse {
        id: String,
        name: String,
        json: String,
    },
    /// `redacted_thinking` and any forward-compat kind: nothing
    /// canonical to reconstruct from the v=1 deltas, so the block
    /// contributes no transcript content (parity with the diagnostic
    /// assembler's `Other`).
    Skip,
}

/// Incremental writer for one model call's model-output transcript entry
/// (ARCH §2.3). The file is the [`entry`] object, its `content` array
/// grown one element per completed block; it is invalid JSON (an open
/// `{"content":[…`) until [`Self::seal`] closes it, which is fine — it is
/// debris until sealed and renamed.
pub(super) struct StagingWriter {
    file: File,
    /// Bytes written so far — tracked, never `stat`'d.
    len: u64,
    /// Separator before the next block: `""` right after the opening
    /// `[`, `","` once a block has been written.
    sep: &'static str,
    /// `(len, sep)` checkpoint captured at the current segment's start;
    /// [`Self::truncate_segment`] rolls the file back to it.
    seg_len: u64,
    seg_sep: &'static str,
    /// The one in-progress block (§2.3 — the only RAM buffer).
    cur: Option<Block>,
    /// This segment's usage report (§2.3 *Usage rides the entry*). Per-segment state
    /// exactly like `cur`: a fresh attempt's counters supersede the last
    /// one's, and a discarded segment's are discarded with its blocks.
    usage: UsageReport,
}

impl StagingWriter {
    /// Create the staging file and open the [`entry`] object.
    pub(super) fn create(path: &Path) -> Result<Self, Error> {
        let mut file = File::create(path)?;
        let open = entry::open();
        file.write_all(open)?;
        let len = open.len() as u64;
        Ok(Self {
            file,
            len,
            sep: "",
            seg_len: len,
            seg_sep: "",
            cur: None,
            usage: UsageReport::default(),
        })
    }

    /// Checkpoint the file at this attempt's start (§4.4 — one segment
    /// per attempt). A subsequent [`Self::truncate_segment`] rolls back
    /// here; leaving it un-truncated accumulates the segment's blocks.
    pub(super) fn begin_segment(&mut self) {
        self.seg_len = self.len;
        self.seg_sep = self.sep;
        self.cur = None;
        self.usage = UsageReport::default();
    }

    /// Fold one canonical event into the entry under construction. Only
    /// content framing matters here; terminal framing (`finish`,
    /// `error`, `end`) is acted on by the caller via segment authority.
    pub(super) fn feed(&mut self, event: &Event) -> Result<(), Error> {
        match event {
            Event::ContentStart { kind, .. } => self.cur = Some(open_block(kind)),
            Event::ContentDelta { delta, .. } => self.on_delta(delta),
            Event::ContentStop { .. } => self.finalize()?,
            Event::Usage(usage) => self.usage.fold(usage),
            _ => {}
        }
        Ok(())
    }

    fn on_delta(&mut self, delta: &Delta) {
        match (&mut self.cur, delta) {
            (Some(Block::Text(s)), Delta::TextDelta(t)) => s.push_str(t),
            (Some(Block::Thinking(s)), Delta::ThinkingDelta(t)) => s.push_str(t),
            (Some(Block::ToolUse { json, .. }), Delta::JsonDelta(t)) => json.push_str(t),
            _ => {}
        }
    }

    /// Turn the completed in-progress block into a canonical [`Content`]
    /// and append it. A tool-use block whose accumulated `json_delta`
    /// does not parse surfaces as [`Error::AdapterJson`] — the same fact
    /// the diagnostic assembler would reject at segment end.
    fn finalize(&mut self) -> Result<(), Error> {
        let block = match self.cur.take() {
            Some(b) => b,
            None => return Ok(()),
        };
        let content = match block {
            Block::Text(text) => Content::Text(text),
            Block::Thinking(text) => Content::Thinking {
                text,
                signature: None,
                id: None,
                encrypted_content: None,
            },
            Block::ToolUse { id, name, json } => {
                let input = if json.is_empty() {
                    serde_json::Value::Object(serde_json::Map::new())
                } else {
                    serde_json::from_str(&json).map_err(Error::AdapterJson)?
                };
                Content::ToolUse {
                    id,
                    name,
                    input,
                    signature: None,
                }
            }
            Block::Skip => return Ok(()),
        };
        let bytes = serde_json::to_vec(&content).expect("Content serializes");
        self.file.write_all(self.sep.as_bytes())?;
        self.file.write_all(&bytes)?;
        self.len += self.sep.len() as u64 + bytes.len() as u64;
        self.sep = ",";
        Ok(())
    }

    /// Discard the current segment's blocks *and its usage* (§4.4 — an
    /// `Error`-terminated segment contributes nothing, and a discarded
    /// attempt's counters are no part of the committed output's cost;
    /// their audit home is `response.json`, which §6/§8 bill from): roll
    /// the file back to the checkpoint [`Self::begin_segment`] captured.
    pub(super) fn truncate_segment(&mut self) -> Result<(), Error> {
        self.file.set_len(self.seg_len)?;
        self.file.seek(SeekFrom::Start(self.seg_len))?;
        self.len = self.seg_len;
        self.sep = self.seg_sep;
        self.cur = None;
        self.usage = UsageReport::default();
        Ok(())
    }

    /// Close the entry object at the model call's final `Finish` (§2.3): the
    /// array closes and the sealing segment's usage report lands as its
    /// sibling, so the file is now valid and ready to rename into the
    /// worktree. A block still in progress at `Finish` — a stream that
    /// reached its terminal without a closing `content_stop` — is
    /// finalized here, so its content is captured (and a malformed
    /// tool-use `json_delta` still surfaces as [`Error::AdapterJson`])
    /// rather than silently dropped.
    pub(super) fn seal(mut self) -> Result<(), Error> {
        self.finalize()?;
        self.file.write_all(&entry::close(&self.usage))?;
        self.file.flush()?;
        Ok(())
    }
}

/// Begin the in-progress block for an opening `content_start` kind.
fn open_block(kind: &ContentKind) -> Block {
    match kind {
        ContentKind::Text {} => Block::Text(String::new()),
        ContentKind::Thinking { .. } => Block::Thinking(String::new()),
        ContentKind::ToolUse { id, name } => Block::ToolUse {
            id: id.clone(),
            name: name.clone(),
            json: String::new(),
        },
        _ => Block::Skip,
    }
}

/// The staging path beside a step's `response.json`
/// (`…/<NNN>/staging.json`, §2.3).
pub(super) fn staging_path_for(response_path: &Path) -> PathBuf {
    response_path.with_file_name(crate::prompt::step::STAGING_FILE)
}

#[cfg(test)]
mod tests;
