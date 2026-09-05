//! **What one attempt changed** — the row the work diff answers with, and the
//! same row nested inside every science attempt (yog's `docs/REMOTE.md` §9.7;
//! bl-a43a).
//!
//! One type, because upstream writes one: a science row's `diff` object *is* a
//! work-diff row, encoded by the same encoder, *"so an attempt's identity has
//! one spelling anywhere"*. A second reading here would be the drift that
//! sentence exists to prevent.
//!
//! # The state rides verbatim and the fields hang off it
//!
//! `unreadable`, `absent`, `diff` — and the seat carries the word rather than
//! an enum ([`super`]'s rung 3), so a state upstream grows paints as itself
//! instead of refusing a whole listing. What follows from that is the shape of
//! this struct: every field a state does not write is an `Option` or an empty
//! list, because *which fields are here* is exactly what the state says and
//! this seat does not re-derive it.
//!
//! **`missing` and `files` read absent as empty, and they are the one place
//! this vocabulary does** (`super::fields::strings` states why): a list is
//! written only by the state that has one, so *no such list on this state* and
//! *this list is empty* are one claim about one row.
//!
//! # Binary is said by shape rather than by a token
//!
//! A changed file carries `added` and `removed`, or it carries `binary: true`
//! and nothing else — upstream's own division, *"binary said as itself"*. So
//! all three ride as options and the pane reads the shape, which is the only
//! reading that cannot disagree with the encoder.

use serde_json::{Map, Value};

use super::fields;

/// The kind token the work diff answers to. It lives here because the row is
/// the answer: `work-diff`'s body is a list of these and nothing else this
/// seat reads.
pub(crate) const KIND: &str = "work-diff";

/// **One attempt's changes**: which ball, on which two refs, and what moved.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Diff {
    pub project: String,
    pub ball_id: String,
    /// The candidate's handle, absent on the ordinary claim.
    pub handle: Option<String>,
    /// The acceptance mark, absent on anything undelivered.
    pub delivered: Option<String>,
    /// `unreadable`, `absent` or `diff`, carried verbatim.
    pub state: String,
    /// The ref it would deliver into.
    pub target: Option<String>,
    /// The ref the work is on.
    pub source: Option<String>,
    pub target_oid: Option<String>,
    pub source_oid: Option<String>,
    /// The refs that are not there yet, on an `absent` row.
    pub missing: Vec<String>,
    /// What changed, on a `diff` row.
    pub files: Vec<Churn>,
    /// Whether the engine stopped listing files before the end.
    pub truncated: Option<bool>,
}

/// **One changed file**: its path, and its churn said in the shape upstream
/// wrote it in.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Churn {
    pub path: String,
    pub added: Option<u64>,
    pub removed: Option<u64>,
    /// `Some(true)` where the file is bytes no line count describes.
    pub binary: Option<bool>,
}

/// One diff row, strictly ([`super`]'s rung 1: every refusal names its field).
pub(crate) fn diff(value: &Value) -> Result<Diff, String> {
    let obj: &Map<String, Value> = value.as_object().ok_or("attempt: not an object")?;
    Ok(Diff {
        project: fields::text(obj, "project")?,
        ball_id: fields::text(obj, "ball_id")?,
        handle: fields::opt_text(obj, "handle")?,
        delivered: fields::opt_text(obj, "delivered")?,
        state: fields::text(obj, "state")?,
        target: fields::opt_text(obj, "target")?,
        source: fields::opt_text(obj, "source")?,
        target_oid: fields::opt_text(obj, "target_oid")?,
        source_oid: fields::opt_text(obj, "source_oid")?,
        missing: fields::strings(obj, "missing")?,
        files: match obj.get("files") {
            None => Vec::new(),
            Some(_) => fields::list(obj, "files", churn)?,
        },
        truncated: fields::opt_flag(obj, "truncated")?,
    })
}

/// One changed file, strictly.
fn churn(value: &Value) -> Result<Churn, String> {
    let obj: &Map<String, Value> = value.as_object().ok_or("changed file: not an object")?;
    Ok(Churn {
        path: fields::text(obj, "path")?,
        added: fields::opt_count(obj, "added")?,
        removed: fields::opt_count(obj, "removed")?,
        binary: fields::opt_flag(obj, "binary")?,
    })
}

#[cfg(test)]
mod tests;
