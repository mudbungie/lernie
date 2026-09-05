//! **One step's records, drilled in** (yog's `docs/REMOTE.md` §8.5; bl-3257) —
//! the tier under the steps list, addressed by the `seq` that list paints.
//!
//! Upstream: *"a step the tree does not hold answers absent records rather
//! than refusing, the same forgiving read the window makes"* — so nothing here
//! is an error path. A record that is not there says so, and a record that is
//! not JSON comes back verbatim and framed as unparseable rather than dropped,
//! because *"rendered bare it is indistinguishable from a file whose content
//! happens to be that text"*.
//!
//! # Two vocabularies, and telling them apart is which SLOT you are looking at
//!
//! The four record files, the response events and every tool call's two sides
//! are [`Doc`]s — `json`, `absent`, `unparsed`. The two capture logs are
//! [`crate::reply::files::Preview`]s — `text`, `truncated`, `binary` — the
//! same bounded reading the worktree's file preview already spends, because
//! nothing parsed them and they are bytes rather than records. Both wear a
//! `kind`, neither is the other's, and each takes [`super`]'s rung 3 on a word
//! this build does not know.
//!
//! # The bytes are carried and the tree is not, which is §4.9 spending itself
//!
//! Every record rides both ways: the parsed `value` and the `raw` it parsed
//! from. Upstream keeps both because *"a `serde_json::Value` is not a lossless
//! record of its source (key order, spacing and number spelling all go), so
//! the tree alone could never answer 'what does the file say'"* — and this
//! seat paints exactly that question, the bytes. So the tree is the one field
//! on this shape that is **not decoded**: it is rung 4, ignored structurally,
//! because a field held for no glass is a field this vocabulary does not
//! carry. The day something here paints a JSON tree is the day it is read.

use serde_json::{Map, Value};

use super::fields;
use super::files::{self, Preview};

/// The kind token this reading answers to.
pub(crate) const KIND: &str = "step";

/// The sentence the engine writes above an unparseable record — its word, not
/// one composed here, so the two faces say it alike.
pub const UNPARSED: &str = "unparseable JSON — bytes verbatim below";

/// One step's whole drill-in.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Step {
    /// The sequence this answers, echoed back — the address it was asked by,
    /// and the one fact that says which row it belongs under.
    pub seq: String,
    /// The step's own metadata record.
    pub meta: Doc,
    /// The wire request that was sent.
    pub request: Doc,
    /// The transcript entry that was staged.
    pub staging: Doc,
    /// Every event of the response stream, one per line. An empty list is a
    /// step whose stream landed nothing, which is not the same claim as an
    /// [`Doc::Absent`] record.
    pub response: Vec<Doc>,
    /// Every tool call this step made.
    pub tools: Vec<ToolCall>,
    /// The step's own captured stderr, absent when it has no bytes.
    pub stderr: Option<Preview>,
    /// The **conversation's** driver log, on the same terms — not this step's
    /// file, read here because the drill-in is the surface that shows a whole
    /// file.
    pub driver: Option<Preview>,
}

/// One tool call's records.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ToolCall {
    pub tool_id: String,
    /// Whether the call ended non-zero, as the engine folded it — `false`
    /// where the output is absent or carried no exit code.
    pub is_error: bool,
    pub input: Doc,
    pub output: Doc,
}

/// A drill-in record: the three things a record file can be, and the word for
/// a fourth this build has no reading of.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Doc {
    /// Parsed — and what is kept is the bytes it parsed from, never the tree
    /// (see the module doc).
    Json { raw: String },
    /// No bytes at all: missing, unreadable, or empty.
    Absent,
    /// Bytes that are not JSON, kept verbatim under the engine's own note.
    Unparsed { note: String, raw: String },
    /// A class this seat has no word for, carried verbatim to paint as itself.
    Unknown(String),
}

/// The whole drill-in, strictly where the shape promises ([`super`]'s rung 1).
pub(crate) fn step(obj: &Map<String, Value>) -> Result<Step, String> {
    Ok(Step {
        seq: fields::text(obj, "seq")?,
        meta: record(obj, "meta")?,
        request: record(obj, "request")?,
        staging: record(obj, "staging")?,
        response: fields::list(obj, "response", doc)?,
        tools: fields::list(obj, "tools", tool)?,
        stderr: capture(obj, "stderr")?,
        driver: capture(obj, "driver")?,
    })
}

/// One record under a key of its own.
fn record(obj: &Map<String, Value>, key: &str) -> Result<Doc, String> {
    obj.get(key)
        .ok_or_else(|| format!("missing field {key:?}"))
        .and_then(doc)
}

/// One record, wherever it sits.
fn doc(value: &Value) -> Result<Doc, String> {
    let obj: &Map<String, Value> = value.as_object().ok_or("record: not an object")?;
    Ok(match fields::text(obj, "kind")?.as_str() {
        "json" => Doc::Json {
            raw: fields::text(obj, "raw")?,
        },
        "absent" => Doc::Absent,
        "unparsed" => Doc::Unparsed {
            note: fields::text(obj, "note")?,
            raw: fields::text(obj, "raw")?,
        },
        other => Doc::Unknown(other.to_owned()),
    })
}

/// One tool call.
fn tool(value: &Value) -> Result<ToolCall, String> {
    let obj: &Map<String, Value> = value.as_object().ok_or("tool call: not an object")?;
    Ok(ToolCall {
        tool_id: fields::text(obj, "tool_id")?,
        is_error: fields::flag(obj, "is_error")?,
        input: record(obj, "input")?,
        output: record(obj, "output")?,
    })
}

/// One capture log, where it has bytes. Its absence is a file with nothing in
/// it, which the encoder spells by leaving the key out rather than by carrying
/// an empty text.
fn capture(obj: &Map<String, Value>, key: &str) -> Result<Option<Preview>, String> {
    match obj.get(key) {
        None | Some(Value::Null) => Ok(None),
        Some(value) => files::preview(value).map(Some),
    }
}

#[cfg(test)]
mod tests;
