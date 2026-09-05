//! **What a ball has cost, and what the sum is over** (yog's `docs/REMOTE.md`
//! §9.7; bl-d2af).
//!
//! One figure, read by the two listings that carry one — the board's rows
//! ([`super::board`]) and one wall's bound balls ([`super::balls`]) — because
//! upstream writes it with one encoder and a second reading of it here would
//! be a second protocol.
//!
//! # The money is upstream's own rendering, and this seat does not compute one
//!
//! `usd` is a string the engine derived from a price table this seat does not
//! have. So it rides verbatim and is painted as it arrived: a seat that
//! multiplied tokens by a rate of its own would be quietly disagreeing with
//! the box that holds the rates, which is the failure mode REMOTE §9.17 names
//! for the trail and which is no different here. Its absence is a fact and not
//! a zero — a figure with no money is one whose tokens no rate priced.
//!
//! # The attribution says what the figure sums over, and it says it twice
//!
//! `kind` is the classification and `label` the clause upstream wrote about
//! it. Both ride, for the reason upstream carries both: a figure over one
//! stamped conversation renders as no clause at all, so the clause alone
//! cannot tell *one conversation* from *workspace-wide*. The kind is carried
//! verbatim ([`super`]'s rung 3) — a classification this build has never seen
//! paints as itself.

use serde_json::{Map, Value};

use super::fields;
use super::steps::Spend;

/// **One figure**: the four counters and their total, what money the engine
/// put on them, and what the sum is over.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Figure {
    /// The counters, in the shape every spend on this wire has
    /// ([`super::steps::Spend`], read from there rather than restated).
    pub tokens: Spend,
    /// The money, as the engine rendered it, or none where no rate priced it.
    pub usd: Option<String>,
    /// What the figure sums over.
    pub attribution: Attribution,
}

/// **What a figure sums over** — the classification, and the clause upstream
/// wrote about it where it wrote one.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Attribution {
    /// `conversations` or `workspace` today, carried verbatim.
    pub kind: String,
    /// The engine's own clause, absent where the classification says it all.
    pub label: Option<String>,
}

/// One figure, strictly ([`super`]'s rung 1: every refusal names its field).
pub(crate) fn figure(value: &Value) -> Result<Figure, String> {
    let obj: &Map<String, Value> = value.as_object().ok_or("spend: not an object")?;
    Ok(Figure {
        tokens: super::steps::spend(obj)?,
        usd: fields::opt_text(obj, "usd")?,
        attribution: attribution(obj)?,
    })
}

/// The nested attribution, read where the figure holds it.
fn attribution(obj: &Map<String, Value>) -> Result<Attribution, String> {
    let held = obj
        .get("attribution")
        .and_then(Value::as_object)
        .ok_or("missing or non-object field \"attribution\"")?;
    Ok(Attribution {
        kind: fields::text(held, "kind")?,
        label: fields::opt_text(held, "label")?,
    })
}

#[cfg(test)]
mod tests;
