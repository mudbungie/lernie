//! **The decision queue** — one row per conversation asking for the operator,
//! anywhere this engine can see (yog's `docs/REMOTE.md` §6, §9.10, §9.11;
//! PROTOCOL 3 and 4; bl-f0ef).
//!
//! It is the engine's *what needs you*: a flattened roster across every
//! enumerated workspace, filtered to the rows that are asking, and the answer
//! to two ops rather than one — `attention` asks for the whole of it, and
//! `seen` answers with **the queue that remains** (`crate::verbs::queue`). So
//! one reading serves both, and a seat that took the second for a receipt
//! would throw away the only statement it gets about what is still waiting.
//!
//! # The row addresses itself, and this seat does not address it back
//!
//! `workspace` is the name that workspace answers to **on its host**, which is
//! not always what a gesture from this box must carry (§8.2). So nothing here
//! composes an address out of it: the window resolves a row against the roster
//! it already holds (`crate::ui::model::queue`), which is the one place this
//! seat's leaf↔host mapping is read, and a row no roster row matches is
//! painted as unaddressable rather than aimed by a guess.
//!
//! # Three optional objects, and each absence is a reading
//!
//! [`QueueRow::flag`] is `null` when nobody raised one, [`QueueRow::held`] is
//! `null` when no invocation is parked at the conversation's capability
//! boundary, and [`QueueRow::failure`] is `null` — rather than absent — when
//! the latest model call did not fail, because the encoder that spells `held`
//! that way spells this one the same. All three are `None` here and none is
//! defaulted to an empty string: *nobody flagged this* and *somebody flagged it
//! and left no words* are two claims, and the wire keeps them two.
//!
//! # The signals are a vocabulary, so they ride verbatim
//!
//! `signals` is the same open set the conversation list's tokens are —
//! [`super`]'s rung 3 — and PROTOCOL 4's `flagged` joined `held` and `mail` in
//! it. A new word in an existing vocabulary is not a shape change: the pane
//! paints each one as itself, so a token this build has never seen costs a
//! badge and not a decode.

use serde_json::{Map, Value};

use super::convs::AgentState;
use super::fields;

/// The kind token this reading answers to.
pub(crate) const KIND: &str = "attention";

/// One conversation asking for the operator, wherever it is.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QueueRow {
    /// The workspace it is on, **as its host names it**.
    pub workspace: String,
    /// The conversation's id — the address every act on this row takes.
    pub agent: String,
    /// What the row is labelled with.
    pub display: String,
    /// The badge state, on the conversation list's own vocabulary.
    pub state: AgentState,
    /// Whether the engine could observe that state at all.
    pub uncertain: bool,
    /// The row's first-line preview.
    pub preview: String,
    /// How long it has waited. Signed, because clock skew between two machines
    /// is a fact and not an error.
    pub age_secs: i64,
    /// How many conversations under it are also asking.
    pub pending: u64,
    /// **Why it is asking**, in the engine's own open vocabulary — rung 3, so
    /// a word this build does not know paints as itself.
    pub signals: Vec<String>,
    /// Why its latest model call failed, in one clause (§9.10).
    pub failure: Option<String>,
    /// **The flag somebody raised on it**, which is the whole point of a queue:
    /// a second party asking the operator to look, in their own words (§9.11).
    pub flag: Option<Flag>,
    /// The invocation parked at the conversation's capability boundary.
    pub held: Option<Held>,
}

/// A raised flag: when, and why in the raiser's own words.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Flag {
    pub at: String,
    pub reason: String,
}

/// **An invocation parked at the capability boundary**: the tool, the call it
/// belongs to, and why it stopped there.
///
/// It is read and painted because it is what makes a row *answerable* rather
/// than merely readable — and the control that releases or declines it is
/// `answer`, which belongs to the tool-host surface this seat does not have
/// (`parity.toml`, bl-e53c). The fact is on the glass; the gesture is filed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Held {
    pub tool: String,
    pub tool_use: String,
    pub reason: String,
}

/// One row, strictly ([`super`]'s rung 1: every refusal names its field).
pub(crate) fn row(value: &Value) -> Result<QueueRow, String> {
    let obj: &Map<String, Value> = value.as_object().ok_or("attention row: not an object")?;
    Ok(QueueRow {
        workspace: fields::text(obj, "workspace")?,
        agent: fields::text(obj, "agent")?,
        display: fields::text(obj, "display")?,
        state: AgentState::of(&fields::text(obj, "state")?),
        uncertain: fields::flag(obj, "uncertain")?,
        preview: fields::text(obj, "preview")?,
        age_secs: fields::secs(obj, "age_secs")?,
        pending: fields::count(obj, "pending")?,
        signals: fields::list(obj, "signals", word)?,
        failure: fields::opt_text(obj, "failure")?,
        flag: fields::nested(obj, "flag", flag)?,
        held: fields::nested(obj, "held", held)?,
    })
}

/// One signal token, verbatim — rung 3 in the one place the row spends it.
fn word(value: &Value) -> Result<String, String> {
    value
        .as_str()
        .map(str::to_owned)
        .ok_or_else(|| "signal: not a string".to_owned())
}

/// The raised flag, strictly.
fn flag(obj: &Map<String, Value>) -> Result<Flag, String> {
    Ok(Flag {
        at: fields::text(obj, "at")?,
        reason: fields::text(obj, "reason")?,
    })
}

/// The parked invocation, strictly.
fn held(obj: &Map<String, Value>) -> Result<Held, String> {
    Ok(Held {
        tool: fields::text(obj, "tool")?,
        tool_use: fields::text(obj, "tool_use")?,
        reason: fields::text(obj, "reason")?,
    })
}

#[cfg(test)]
mod tests;
