//! **Every delivery attempt of one workspace** — what each was fired with,
//! what it cost, what it said and how it ended (yog's `docs/REMOTE.md` §9.7;
//! bl-a43a).
//!
//! One row per attempt, the ordinary claim and each fan candidate alike. It is
//! **derived when you ask**: nothing behind it is stored, so the same row a
//! minute later is a statement about the world a minute later — which is why
//! its read stands on the pane rather than being posted once.
//!
//! # The diff column is the work diff's own row
//!
//! [`super::diff`] is the one spelling of an attempt's identity and churn, and
//! this composes it rather than restating it — upstream's own arrangement,
//! kept here for upstream's own reason.
//!
//! # The outcome is a token with whatever that token can say
//!
//! `accepted` carries the delivery commit, `rejected` may carry the sibling
//! that landed instead, and `reworked` and `pending` say nothing else because
//! there is nothing else to say. The token rides verbatim ([`super`]'s rung 3)
//! and the two facts beside it are options, which is the same shape and the
//! same reason [`super::diff`] has.
//!
//! # The counters are the attempt's own four and there is no total
//!
//! They arrive under their own names — `input_tokens` and its three — rather
//! than under the figure's (`super::spend`), and with no total beside them. So
//! this is its own small type: a reader that reached for the figure's would be
//! reading a shape this answer does not have.

use serde_json::{Map, Value};

use super::diff::Diff;
use super::{diff, fields};

/// The kind token this reading answers to.
pub(crate) const KIND: &str = "science";

/// **One delivery attempt**, whole.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Attempt {
    /// What it changed, in the work diff's own spelling.
    pub diff: Diff,
    /// The commit both ends of its diff departed from.
    pub base: Option<String>,
    /// The conversation that ran it.
    pub conversation: Option<String>,
    /// The goal it was fired with, frozen at dispatch.
    pub goal: Option<String>,
    /// The config commit it is governed by.
    pub governing: Option<String>,
    /// What it last said.
    pub response: Option<String>,
    /// The instruction documents frozen onto its dispatch commit.
    pub pins: Vec<String>,
    pub usage: Usage,
    pub wall_secs: u64,
    pub steps: u64,
    /// Every message delivered into it.
    pub verdicts: Vec<Verdict>,
    /// How many entries compaction deleted out from under it, absent on an
    /// intact record.
    pub compacted: Option<u64>,
    pub outcome: Ending,
}

/// The attempt's four counters, by their own names and with no total.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Usage {
    pub input: u64,
    pub output: u64,
    pub cache_read: u64,
    pub cache_write: u64,
}

/// One message delivered into an attempt.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Verdict {
    pub sender: String,
    pub body: String,
}

/// **How it ended**: the token, and whatever that token can say.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ending {
    /// `accepted`, `rejected`, `reworked` or `pending`, carried verbatim.
    pub state: String,
    /// The delivery commit, on an acceptance.
    pub commit: Option<String>,
    /// The sibling that landed instead, where a rejection names one.
    pub by: Option<String>,
}

/// One attempt, strictly ([`super`]'s rung 1: every refusal names its field).
pub(crate) fn row(value: &Value) -> Result<Attempt, String> {
    let obj: &Map<String, Value> = value.as_object().ok_or("science row: not an object")?;
    Ok(Attempt {
        diff: diff::diff(obj.get("diff").ok_or("science row: missing diff")?)?,
        base: fields::opt_text(obj, "base")?,
        conversation: fields::opt_text(obj, "conversation")?,
        goal: fields::opt_text(obj, "goal")?,
        governing: fields::opt_text(obj, "governing")?,
        response: fields::opt_text(obj, "response")?,
        pins: fields::strings(obj, "pins")?,
        usage: usage(obj)?,
        wall_secs: fields::count(obj, "wall_secs")?,
        steps: fields::count(obj, "steps")?,
        verdicts: fields::list(obj, "verdicts", verdict)?,
        compacted: fields::opt_count(obj, "compacted")?,
        outcome: ending(obj)?,
    })
}

/// The nested counters.
fn usage(obj: &Map<String, Value>) -> Result<Usage, String> {
    let held = obj
        .get("usage")
        .and_then(Value::as_object)
        .ok_or("missing or non-object field \"usage\"")?;
    Ok(Usage {
        input: fields::count(held, "input_tokens")?,
        output: fields::count(held, "output_tokens")?,
        cache_read: fields::count(held, "cache_read_tokens")?,
        cache_write: fields::count(held, "cache_write_tokens")?,
    })
}

/// One delivered message.
fn verdict(value: &Value) -> Result<Verdict, String> {
    let obj: &Map<String, Value> = value.as_object().ok_or("verdict: not an object")?;
    Ok(Verdict {
        sender: fields::text(obj, "sender")?,
        body: fields::text(obj, "body")?,
    })
}

/// The nested outcome.
fn ending(obj: &Map<String, Value>) -> Result<Ending, String> {
    let held = obj
        .get("outcome")
        .and_then(Value::as_object)
        .ok_or("missing or non-object field \"outcome\"")?;
    Ok(Ending {
        state: fields::text(held, "state")?,
        commit: fields::opt_text(held, "commit")?,
        by: fields::opt_text(held, "by")?,
    })
}

#[cfg(test)]
mod tests;
