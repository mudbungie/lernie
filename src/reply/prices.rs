//! **The price table and the ceiling** — the world's two money facts, read
//! and written from a seat (yog's `docs/REMOTE.md` §9.23; bl-9111).
//!
//! One shape answers three ops: `prices` reads it, and `price` and `ceiling`
//! each answer with it re-derived after the write — so a receipt here is never
//! a sentence about what was asked, it is the table as it now stands.
//!
//! # Rates are the operator's own decimals, read back as written
//!
//! A rate is USD per million tokens, quoted by whoever wrote the row. It rides
//! as a JSON number and is held as one ([`Number`], never an `f64`, so `15`
//! stays `15` and is sent back as `15` rather than `15.0`); nothing here multiplies it by a
//! count, because the box that holds the rates is the only box that may say
//! what a count cost ([`super::spend`]'s own rule). A `"*"` model is the
//! literal string — *every model this row serves that it does not name* — and
//! a row of zeros is a priced row (a subscription), which paints `$0`, never
//! blank.
//!
//! # Three absences, each a fact
//!
//! No `ceiling` is no bound. No `spent` is no priced spend, which is not a
//! zero. No `released` is a receipt from an act that wakes nothing — only a
//! `ceiling` act carries it, counting the conversations the bound had parked
//! and the move released.

use serde_json::{Map, Number, Value};

use super::fields;
use super::spend::Cost;

/// The kind token this reading answers to.
pub(crate) const KIND: &str = "prices";

/// The table whole: its rows, the bound, the world's spend against it, and
/// what a ceiling act woke.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Prices {
    /// The table, flattened, in the engine's own order.
    pub rows: Vec<Rate>,
    /// The bound in USD, where one is set.
    pub ceiling: Option<Number>,
    /// The world's priced spend, where anything was priced.
    pub spent: Option<Cost>,
    /// Conversations a ceiling act woke — present on that receipt alone.
    pub released: Option<u64>,
}

/// **One row of the table**: which provider's which model, and its four rates
/// in USD per million tokens.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Rate {
    pub provider: String,
    /// A model name, or the literal `"*"`.
    pub model: String,
    pub input: Number,
    pub output: Number,
    pub cache_read: Number,
    pub cache_write: Number,
}

/// The whole answer, strictly ([`super`]'s rung 1).
pub(crate) fn prices(obj: &Map<String, Value>) -> Result<Prices, String> {
    Ok(Prices {
        rows: fields::rows(obj, rate)?,
        ceiling: match obj.get("ceiling") {
            None | Some(Value::Null) => None,
            Some(_) => Some(number(obj, "ceiling")?),
        },
        spent: super::spend::cost(obj, "spent")?,
        released: fields::opt_count(obj, "released")?,
    })
}

/// One row, strictly.
fn rate(value: &Value) -> Result<Rate, String> {
    let obj: &Map<String, Value> = value.as_object().ok_or("price row: not an object")?;
    Ok(Rate {
        provider: fields::text(obj, "provider")?,
        model: fields::text(obj, "model")?,
        input: number(obj, "input")?,
        output: number(obj, "output")?,
        cache_read: number(obj, "cache_read")?,
        cache_write: number(obj, "cache_write")?,
    })
}

/// A decimal, which is the one shape on this wire that is neither a count nor
/// text — so it is read here, beside the one answer that carries any.
fn number(obj: &Map<String, Value>, key: &str) -> Result<Number, String> {
    match obj.get(key) {
        Some(Value::Number(held)) => Ok(held.clone()),
        _ => Err(format!("missing or non-number field {key:?}")),
    }
}

impl Rate {
    /// The row as said: the key, then the four rates by name.
    pub fn said(&self) -> String {
        format!(
            "{} {}  in {}  out {}  cache-read {}  cache-write {}  per Mtok",
            self.provider, self.model, self.input, self.output, self.cache_read, self.cache_write
        )
    }
}

impl Prices {
    /// **The bound and the world's spend against it**, in one line.
    pub fn standing(&self) -> String {
        let spent = self.spent.as_ref().map_or_else(
            || "nothing priced spent".to_owned(),
            |cost| format!("{} spent", cost.said()),
        );
        match &self.ceiling {
            Some(usd) => format!("{spent} of a ${usd} ceiling"),
            None => format!("{spent}, no ceiling"),
        }
    }

    /// **What a ceiling act woke**, where it said.
    pub fn woke(&self) -> Option<String> {
        self.released
            .map(|n| format!("the ceiling released {n} parked conversation(s)"))
    }
}

#[cfg(test)]
mod tests;
