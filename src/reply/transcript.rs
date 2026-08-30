//! **The conversation** (yog's `docs/REMOTE.md` §8, §9.7) — the committed
//! entries with the live tail folded among them, which is the whole of what
//! the chat pane paints.
//!
//! **Bytes ride as text, and that is the engine's ruling read from this side.**
//! Every entry carries the verbatim bytes it was read from, and they cross as
//! a string: a transcript entry is a text file, the Raw toggle is the only
//! other door to the envelope the parsed view drops, and a byte array beside
//! the text would be a second spelling of one content. A byte no string can
//! name was already replaced on the way out, so this holds exactly what the
//! wire carried and never claims to hold the file.
//!
//! **Every entry is kept, including the ones nothing can parse.** An entry
//! whose filename or bytes the engine could not read arrives as
//! [`EntryKind::Raw`], and an entry of a kind *this build* does not know
//! arrives as [`EntryKind::Unknown`] — rung 3 of [`super`]'s policy, spent
//! here rather than rung 2's refusal, because the entry still carries its
//! `raw` and a pane that can show the bytes has something true to paint. The
//! two are held apart deliberately: "the engine could not read this" and "this
//! seat is behind" are different sentences and only one of them is fixed by an
//! upgrade.

use serde_json::{Map, Value};

use super::fields;

/// The canonical content blocks a model entry is made of.
pub mod blocks;

pub use blocks::{Block, Usage};

/// This reply's kind token.
pub(crate) const KIND: &str = "transcript";

/// A conversation's ordered entries.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Transcript {
    pub entries: Vec<Entry>,
}

/// One entry: where it came from, what it says, and the bytes behind it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Entry {
    /// The source filename, or the synthetic name the live tail wears.
    pub name: String,
    /// The verbatim backing text (see the module doc on why it is not bytes).
    pub raw: String,
    /// The parsed projection.
    pub kind: EntryKind,
}

/// What one entry turned out to be.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EntryKind {
    /// A delivered message. `epitaph` is present exactly on a **result
    /// deposit** — a child's terminal, asserting how it ended.
    ///
    /// The epitaph rides as its **label**, not as an enum this seat mirrors.
    /// The engine's own reader for it is total by design — an unrecognised
    /// ending reads back as the word it was written from — so a seat that
    /// re-derived a closed set here would be strictly stricter than the
    /// authority it implements against, which is a way of being wrong.
    Delivered {
        sender: String,
        epitaph: Option<String>,
        body: String,
    },
    /// Model output as canonical content blocks, with the provider's committed
    /// token counters when the bytes carried them.
    Model {
        model_id: String,
        blocks: Vec<Block>,
        usage: Usage,
    },
    /// A tool result. The id is **opaque** and pairs by byte equality: no
    /// provider's shape is assumed, here or anywhere.
    ToolResult {
        tool_use_id: String,
        content: String,
        is_error: bool,
    },
    /// The live tail — the model's reasoning and its answer so far, held apart
    /// because they are two things being said and each becomes its own row,
    /// exactly as the committed entry that supersedes them will. Either may be
    /// empty, and an empty half is simply no row.
    Streaming { thinking: String, text: String },
    /// A span of entries the compactor **deleted**, standing where they were.
    /// `first` and `last` are the missing counter values, inclusive, and are
    /// the only thing this entry asserts; `summary` is what the compactor left
    /// in their place, and is empty wherever it left none.
    Compacted {
        first: u64,
        last: u64,
        summary: String,
    },
    /// The engine could not parse it, and surfaced it rather than dropping it.
    Raw,
    /// **This build** could not name it. The word is kept verbatim, and `raw`
    /// still holds what the entry said.
    Unknown(String),
}

/// Read the whole conversation.
pub(crate) fn transcript(obj: &Map<String, Value>) -> Result<Transcript, String> {
    Ok(Transcript {
        entries: fields::rows(obj, entry)?,
    })
}

/// Read one entry: the two facts every entry carries, then whatever its kind
/// adds.
fn entry(v: &Value) -> Result<Entry, String> {
    let o = v.as_object().ok_or("transcript row: not an object")?;
    Ok(Entry {
        name: fields::text(o, "name")?,
        raw: fields::text(o, "raw")?,
        kind: kind(o)?,
    })
}

/// The kind discriminant and the fields it brings with it.
fn kind(o: &Map<String, Value>) -> Result<EntryKind, String> {
    Ok(match fields::text(o, "kind")?.as_str() {
        "delivered" => EntryKind::Delivered {
            sender: fields::text(o, "sender")?,
            epitaph: fields::opt_text(o, "epitaph")?,
            body: fields::text(o, "body")?,
        },
        "model" => EntryKind::Model {
            model_id: fields::text(o, "model_id")?,
            blocks: fields::list(o, "blocks", blocks::block)?,
            usage: blocks::usage(o)?,
        },
        "tool-result" => EntryKind::ToolResult {
            tool_use_id: fields::text(o, "tool_use_id")?,
            content: fields::text(o, "content")?,
            is_error: fields::flag(o, "is_error")?,
        },
        "streaming" => EntryKind::Streaming {
            thinking: fields::text(o, "thinking")?,
            text: fields::text(o, "text")?,
        },
        "compacted" => EntryKind::Compacted {
            first: fields::count(o, "first")?,
            last: fields::count(o, "last")?,
            summary: fields::text(o, "summary")?,
        },
        "raw" => EntryKind::Raw,
        other => EntryKind::Unknown(other.to_owned()),
    })
}

#[cfg(test)]
mod tests;
