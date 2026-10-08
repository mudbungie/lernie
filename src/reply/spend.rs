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
    pub cost: Option<Cost>,
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
        cost: money(obj)?,
        attribution: attribution(obj)?,
    })
}

/// **What the engine put on a count**: its own rendering of the money, and
/// how many of the tokens under it no rate priced.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Cost {
    /// The money, verbatim — never computed here.
    pub usd: String,
    /// Tokens no row of the table priced. Above zero, `usd` is a floor.
    pub unpriced_tokens: u64,
}

impl Cost {
    /// The money as said: `$x`, or `at least $x` where it is a floor.
    pub fn said(&self) -> String {
        if self.unpriced_tokens > 0 {
            format!("at least {}", self.usd)
        } else {
            self.usd.clone()
        }
    }
}

/// **A count and the cost beside it, as one clause** — `N tokens — $x`, or
/// the count alone where the engine put no money on it. The one sentence
/// every surface that paints a token count says it in (yog DESIGN §3.5).
pub fn priced(count: String, cost: Option<&Cost>) -> String {
    match cost {
        Some(cost) => format!("{count} — {}", cost.said()),
        None => count,
    }
}

/// **The money half, read where any shape holds it** — `None` where no `usd`
/// rode, which is a count no rate priced and never a zero.
pub(crate) fn money(obj: &Map<String, Value>) -> Result<Option<Cost>, String> {
    let Some(usd) = fields::opt_text(obj, "usd")? else {
        return Ok(None);
    };
    Ok(Some(Cost {
        usd,
        unpriced_tokens: fields::opt_count(obj, "unpriced_tokens")?.unwrap_or(0),
    }))
}

/// **A cost under its own key**, absent where the engine has no price table.
/// Present, it is a priced figure and must say its money.
pub(crate) fn cost(obj: &Map<String, Value>, key: &str) -> Result<Option<Cost>, String> {
    fields::nested(obj, key, |held| {
        money(held)?.ok_or_else(|| format!("field {key:?}: missing \"usd\""))
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
