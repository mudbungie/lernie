//! **The steps a conversation's loop has taken** — the cheap per-step summary
//! the records pane opens on (yog's `docs/REMOTE.md` §8.5; bl-2cf7).
//!
//! One row per step, in sequence order, and the view-level orphaned-tail state
//! at the top because it is not any one step's fact. The drill-in under a row —
//! one step's record files, its tool calls, its capture logs — is the `step`
//! reply, which is bl-3257's and not decoded here.
//!
//! # The class tokens are carried verbatim, and `"none"` is one of them
//!
//! `framing`, `wound` and `orphan` are each a closed set on the engine's side
//! and an open one here, which is [`super`]'s rung 3: a word this seat does not
//! know paints as itself rather than as a neighbour it is not. The one word the
//! pane interprets is [`NONE`] — the engine's own spelling for *nothing is
//! wounded, nothing is orphaned* — because painting a badge that says `none`
//! would state an absence twice.
//!
//! # Absence is a reading, never a zero
//!
//! The timestamps and the read-state commit are absent keys when the step's
//! own record did not carry them, and upstream is explicit that nothing stands
//! in for a fact nobody recorded. `auth_row` absent under `auth_failed` means
//! the affordance is offered with nothing derivable to point it at; a reason
//! key absent beside a wound or an orphan is a class that left no words.

use serde_json::{Map, Value};

use super::fields;

/// The kind token this reading answers to.
pub(crate) const KIND: &str = "steps";

/// The engine's word for a wound or an orphan that is not there. The one
/// token the pane reads rather than paints — an absence stated once.
pub const NONE: &str = "none";

/// The steps listing whole: the rows, and the orphaned-tail state that is the
/// view's own fact rather than any row's.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Steps {
    /// One row per step, in the engine's own sequence order.
    pub rows: Vec<StepRow>,
    /// Which tail is orphaned — [`NONE`], or a class token carried verbatim.
    pub orphan: String,
    /// The orphan's own words, where the class left any.
    pub orphan_reason: Option<String>,
}

/// One step's summary: how it ended, what it cost, and what went wrong.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StepRow {
    /// The step's sequence number — the address the `step` drill-in takes.
    pub seq: String,
    /// The terminal classification, verbatim.
    pub framing: String,
    /// How many attempts the step took.
    pub attempts: u64,
    /// What the step cost, in the four counters and their total.
    pub tokens: Spend,
    /// The read-state commit, where the step's record carried one.
    pub commit: Option<String>,
    pub started_at: Option<String>,
    pub ended_at: Option<String>,
    /// Whether the sign-in affordance is offered on this step.
    pub auth_failed: bool,
    /// The provider row it points at, where one was derivable.
    pub auth_row: Option<String>,
    /// The wound's class — [`NONE`], or a token carried verbatim.
    pub wound: String,
    /// The adapter's own last words, where the class left any.
    pub wound_reason: Option<String>,
}

/// The four counters and their total, exactly as the wire carries them. The
/// total rides rather than being summed here because it is the engine's
/// derivation — cache counters do not add the way a reader would guess.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Spend {
    pub input: u64,
    pub output: u64,
    pub cache_read: u64,
    pub cache_write: u64,
    pub total: u64,
}

/// The whole listing, strictly ([`super`]'s rung 1: every refusal names the
/// field it refused on).
pub(crate) fn steps(obj: &Map<String, Value>) -> Result<Steps, String> {
    Ok(Steps {
        rows: fields::rows(obj, row)?,
        orphan: fields::text(obj, "orphan")?,
        orphan_reason: fields::opt_text(obj, "orphan_reason")?,
    })
}

/// One row, strictly. The optional fields are readings, not tolerances: each
/// absence is upstream's own spelling of a fact nobody recorded.
fn row(value: &Value) -> Result<StepRow, String> {
    let obj: &Map<String, Value> = value.as_object().ok_or("step row: not an object")?;
    Ok(StepRow {
        seq: fields::text(obj, "seq")?,
        framing: fields::text(obj, "framing")?,
        attempts: fields::count(obj, "attempts")?,
        tokens: spend(obj)?,
        commit: fields::opt_text(obj, "commit")?,
        started_at: fields::opt_text(obj, "started_at")?,
        ended_at: fields::opt_text(obj, "ended_at")?,
        auth_failed: fields::flag(obj, "auth_failed")?,
        auth_row: fields::opt_text(obj, "auth_row")?,
        wound: fields::text(obj, "wound")?,
        wound_reason: fields::opt_text(obj, "wound_reason")?,
    })
}

/// The nested spend object, read where the row holds it.
fn spend(row: &Map<String, Value>) -> Result<Spend, String> {
    let obj = row
        .get("tokens")
        .and_then(Value::as_object)
        .ok_or("missing or non-object field \"tokens\"")?;
    Ok(Spend {
        input: fields::count(obj, "input")?,
        output: fields::count(obj, "output")?,
        cache_read: fields::count(obj, "cache_read")?,
        cache_write: fields::count(obj, "cache_write")?,
        total: fields::count(obj, "total")?,
    })
}

#[cfg(test)]
mod tests;
