//! The transcript writer's staging sink (ARCH §2.3 *The transcript
//! writer*, §4.4 segment authority).
//!
//! Both sinks of a model call are streams off the one adapter pass:
//! every event appends verbatim to the diagnostic `response.json`
//! (§4.4), and *content* additionally streams here — the assistant
//! transcript entry under construction, a JSON array of brazen's
//! canonical [`Content`] blocks written **block-by-block as each
//! content block completes** (`content_stop`). A block is the smallest
//! unit that exists before it is whole, so the in-progress block's
//! deltas are the only buffering anywhere (§2.3): a completed block is
//! flushed to disk and dropped from memory at its `content_stop`.
//!
//! Segment authority (§4.4) drives the file mechanically:
//!
//! - an `Error`-terminated segment [`truncate_segment`]s — the
//!   discarded attempt's audit home is `response.json`;
//! - a `Pause`-terminated segment is left un-truncated, so its blocks
//!   *accumulate* and the continuation (§2.10) resumes past them,
//!   reading the writer's own sink — never a diagnostic record;
//! - the final `Finish` [`seal`]s the array, ready to rename into the
//!   worktree as `messages/NNN-assistant.json` (§2.3).
//!
//! Nothing here reads `response.json` back: the staging file is the
//! writer's own sink, so the diagnostic-only contract (§2.3) is
//! untouched.
//!
//! [`truncate_segment`]: StagingWriter::truncate_segment
//! [`seal`]: StagingWriter::seal

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

/// Incremental writer for one model call's assistant transcript entry
/// (ARCH §2.3). The file is a JSON array grown one element per completed
/// block; it is invalid JSON (an open `[…`) until [`Self::seal`] closes
/// it, which is fine — it is debris until sealed and renamed.
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
}

impl StagingWriter {
    /// Create the staging file and open the JSON array with `[`.
    pub(super) fn create(path: &Path) -> Result<Self, Error> {
        let mut file = File::create(path)?;
        file.write_all(b"[")?;
        Ok(Self {
            file,
            len: 1,
            sep: "",
            seg_len: 1,
            seg_sep: "",
            cur: None,
        })
    }

    /// Checkpoint the file at this attempt's start (§4.4 — one segment
    /// per attempt). A subsequent [`Self::truncate_segment`] rolls back
    /// here; leaving it un-truncated accumulates the segment's blocks.
    pub(super) fn begin_segment(&mut self) {
        self.seg_len = self.len;
        self.seg_sep = self.sep;
        self.cur = None;
    }

    /// Fold one canonical event into the entry under construction. Only
    /// content framing matters here; terminal framing (`finish`,
    /// `error`, `end`) is acted on by the caller via segment authority.
    pub(super) fn feed(&mut self, event: &Event) -> Result<(), Error> {
        match event {
            Event::ContentStart { kind, .. } => self.cur = Some(open_block(kind)),
            Event::ContentDelta { delta, .. } => self.on_delta(delta),
            Event::ContentStop { .. } => self.finalize()?,
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
            },
            Block::ToolUse { id, name, json } => {
                let input = if json.is_empty() {
                    serde_json::Value::Object(serde_json::Map::new())
                } else {
                    serde_json::from_str(&json).map_err(Error::AdapterJson)?
                };
                Content::ToolUse { id, name, input }
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

    /// Discard the current segment's blocks (§4.4 — an `Error`-terminated
    /// segment contributes nothing): roll the file back to the checkpoint
    /// [`Self::begin_segment`] captured.
    pub(super) fn truncate_segment(&mut self) -> Result<(), Error> {
        self.file.set_len(self.seg_len)?;
        self.file.seek(SeekFrom::Start(self.seg_len))?;
        self.len = self.seg_len;
        self.sep = self.seg_sep;
        self.cur = None;
        Ok(())
    }

    /// Close the JSON array at the model call's final `Finish` (§2.3):
    /// the file is now valid and ready to rename into the worktree.
    pub(super) fn seal(mut self) -> Result<(), Error> {
        self.file.write_all(b"]")?;
        self.file.flush()?;
        Ok(())
    }
}

/// Begin the in-progress block for an opening `content_start` kind.
fn open_block(kind: &ContentKind) -> Block {
    match kind {
        ContentKind::Text {} => Block::Text(String::new()),
        ContentKind::Thinking {} => Block::Thinking(String::new()),
        ContentKind::ToolUse { id, name } => Block::ToolUse {
            id: id.clone(),
            name: name.clone(),
            json: String::new(),
        },
        _ => Block::Skip,
    }
}

/// The staging path beside a step's `response.json`
/// (`…/<NNN>/assistant.staging.json`, §2.3).
pub(super) fn staging_path_for(response_path: &Path) -> PathBuf {
    response_path.with_file_name(crate::prompt::step::STAGING_FILE)
}

#[cfg(test)]
mod tests;
