//! **The price table and the ceiling** — one read and two acts, all of them
//! window-level (yog's `docs/REMOTE.md` §9.23; DESIGN §4.41; bl-9111).
//!
//! The table and the bound are WORLD facts (yog DESIGN §4.1): none of the
//! three names a workspace, so each one's subject is every channel this box
//! holds, on [`super::window`]'s terms — the CLI fans them, and the window
//! addresses each act down the channel whose section it fired from.
//!
//! # One is a row and two are doors, and the reason is the table's own rule
//!
//! [`PRICES`] takes nothing, so it is a row and `lernie prices` fans exactly
//! as `lernie workspaces` does. `price` carries a nested object of NUMBERS
//! that may be absent, and `ceiling` a number that may be absent — and
//! [`super`]'s table is *a word and its parameters, all of them named
//! strings*. So both are typed doors beside the trail's ([`super::trail`]),
//! spelled for argv in [`super::doors`], with no row in the gesture table.
//!
//! # A rate is the operator's own decimal, sent as written
//!
//! Each is parsed as a JSON [`Number`] and never as an `f64`, so `15` goes
//! out as `15` and not `15.0` — the engine reads rates back exactly as
//! written, and a seat that re-spelled them would be the one place they
//! changed. A word that is not a number is refused HERE, because it is the
//! typist's mistake and costs no connection; a NEGATIVE number is a number,
//! and the engine's in-band refusal is what names it.

use serde_json::{Map, Number, Value, json};

use super::Verb;
use crate::envelope;

/// The `prices` read's row.
pub const PRICES: Verb = Verb {
    word: "prices",
    params: &[],
    flags: &[],
    summary: "the price table, the spend ceiling, and the world's spend against it",
    detail: "One row per (provider, model) the engine prices, with its four \
             rates in USD per million tokens as the operator wrote them — a \
             `*` model means every model that provider serves and the table \
             does not name, and a row of zeros is a subscription, priced at \
             nothing rather than unpriced. Then the ceiling, where one is \
             set, and what the world has spent against it, `at least` where \
             some tokens no row priced. It takes no address, so its subject \
             is EVERY channel this box holds.",
};

/// The word `price` is, and the envelope's `op`. One fact.
pub const PRICE: &str = "price";
/// The word `ceiling` is, likewise.
pub const CEILING: &str = "ceiling";
/// **The word that deletes**: a row's, or the bound.
pub const OFF: &str = "off";

/// The four rates, by the wire's own names, in the order they are typed.
const RATES: [&str; 4] = ["input", "output", "cache_read", "cache_write"];

/// The price table, typed.
pub fn prices() -> Value {
    PRICES.built(Vec::new(), &[])
}

/// **Write one row, or delete it** — `rates` absent is the delete.
pub fn price(provider: String, model: String, rates: Option<[Number; 4]>) -> Value {
    let mut map = Map::new();
    map.insert(envelope::OP.to_owned(), json!(PRICE));
    map.insert("provider".to_owned(), json!(provider));
    map.insert("model".to_owned(), json!(model));
    if let Some(rates) = rates {
        let held: Map<String, Value> = RATES
            .iter()
            .zip(rates)
            .map(|(name, rate)| ((*name).to_owned(), Value::Number(rate)))
            .collect();
        map.insert("rates".to_owned(), Value::Object(held));
    }
    Value::Object(map)
}

/// **Set the bound, or lift it** — `usd` absent is the lift.
pub fn ceiling(usd: Option<Number>) -> Value {
    match usd {
        Some(usd) => json!({ envelope::OP: CEILING, "usd": usd }),
        None => json!({ envelope::OP: CEILING }),
    }
}

/// **The rates as typed**: [`OFF`] alone is the delete, two to four numbers
/// are a row — an omitted rate is zero, the table's own default — and
/// anything else is refused naming the word.
pub fn rates(words: &[&str]) -> Result<Option<[Number; 4]>, String> {
    if words == [OFF] {
        return Ok(None);
    }
    if !(2..=4).contains(&words.len()) {
        return Err(format!(
            "a price is `{OFF}` or two to four rates (input, output, cache read, \
             cache write) and got {} word(s)",
            words.len()
        ));
    }
    let mut held: [Number; 4] = std::array::from_fn(|_| Number::from(0_u8));
    for (slot, word) in held.iter_mut().zip(words) {
        *slot = number(word)?;
    }
    Ok(Some(held))
}

/// **The bound as typed**: [`OFF`] lifts it, a number sets it.
pub fn bound(word: &str) -> Result<Option<Number>, String> {
    if word == OFF {
        return Ok(None);
    }
    number(word).map(Some)
}

/// One decimal, as the operator wrote it.
fn number(word: &str) -> Result<Number, String> {
    word.trim()
        .parse::<Number>()
        .map_err(|_| format!("{word:?} is not a number of US dollars"))
}

#[cfg(test)]
mod tests;
