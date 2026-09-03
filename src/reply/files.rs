//! **What a conversation's worktree holds** — the listing half of the records
//! pane (yog's `docs/REMOTE.md` §8.5; bl-2cf7).
//!
//! # The worktree's absence is a fact, not an empty listing
//!
//! `worktree` is the discriminant the encoder chose so a reader never has to
//! tell a torn-down worktree from one with nothing in it: `rows` is present
//! exactly when there is a worktree to list. The reading keeps the two claims
//! two — [`Files::listing`] is `None` where no worktree stands, and a listing
//! of zero rows where one does.
//!
//! # The preview is decoded whole and composed narrow
//!
//! The reply carries a bounded preview when the gesture named a file, and the
//! reader takes all three of its classes — plus a fourth arm for a class this
//! build has no word for, which is [`super`]'s rung 3: an unknown *kind* here
//! refuses nothing, because the listing beside it is entirely readable, and
//! the word paints as itself. What this seat *composes* is the bare read only
//! (`crate::verbs::records`), so a live preview arrives only from an engine
//! answering somebody else's gesture — and is painted rather than dropped,
//! because an answer that arrived is an answer.

use serde_json::{Map, Value};

use super::fields;

/// The kind token this reading answers to.
pub(crate) const KIND: &str = "files";

/// The `files` answer whole: the listing where a worktree stands, the bounded
/// preview where one was asked for, and where the work actually lands when
/// this listing does not reach it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Files {
    /// The walked worktree, or `None` where none stands to walk.
    pub listing: Option<Listing>,
    /// The asked-for file, bounded.
    pub preview: Option<Preview>,
    /// Where the conversation's work lands, named exactly when it is somewhere
    /// this listing is not.
    pub working_dir: Option<String>,
}

/// The walked entries, and whether the walk was cut short.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Listing {
    pub rows: Vec<FileRow>,
    /// Whether the listing stops before the worktree does.
    pub truncated: bool,
}

/// One walked entry: its identity, its size, and whether it is a directory.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileRow {
    pub path: String,
    pub size: u64,
    pub dir: bool,
}

/// A bounded preview — the engine's three classes, and the rung-3 arm for a
/// fourth this build has no word for.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Preview {
    /// The whole file, as text.
    Text(String),
    /// The head of a file too big to carry, and how big it really is.
    Truncated { text: String, size: u64 },
    /// Bytes no text preview can say, and how many of them.
    Binary { size: u64 },
    /// A class this seat has no word for, carried verbatim to paint as itself.
    Unknown(String),
}

/// The whole answer, strictly where the shape promises and optionally where
/// absence is a reading ([`super`]'s rung 1).
pub(crate) fn files(obj: &Map<String, Value>) -> Result<Files, String> {
    let listing = if fields::flag(obj, "worktree")? {
        Some(Listing {
            rows: fields::rows(obj, row)?,
            truncated: fields::flag(obj, "truncated")?,
        })
    } else {
        None
    };
    let preview = match obj.get("preview") {
        None => None,
        Some(value) => Some(preview(value)?),
    };
    Ok(Files {
        listing,
        preview,
        working_dir: fields::opt_text(obj, "working_dir")?,
    })
}

/// One walked entry, strictly.
fn row(value: &Value) -> Result<FileRow, String> {
    let obj: &Map<String, Value> = value.as_object().ok_or("file row: not an object")?;
    Ok(FileRow {
        path: fields::text(obj, "path")?,
        size: fields::count(obj, "size")?,
        dir: fields::flag(obj, "dir")?,
    })
}

/// The bounded preview: three known classes and the word itself for a fourth.
fn preview(value: &Value) -> Result<Preview, String> {
    let obj: &Map<String, Value> = value.as_object().ok_or("preview: not an object")?;
    Ok(match fields::text(obj, "kind")?.as_str() {
        "text" => Preview::Text(fields::text(obj, "text")?),
        "truncated" => Preview::Truncated {
            text: fields::text(obj, "text")?,
            size: fields::count(obj, "size")?,
        },
        "binary" => Preview::Binary {
            size: fields::count(obj, "size")?,
        },
        other => Preview::Unknown(other.to_owned()),
    })
}

#[cfg(test)]
mod tests;
