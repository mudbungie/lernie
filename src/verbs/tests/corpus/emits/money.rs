//! **The money doors** (bl-9111), split from [`super`] at the line cap on the
//! seam the corpus draws: these two are the doors whose arguments are JSON
//! NUMBERS, and they round-trip off the frame's own numbers so a whole rate
//! stays whole — `15` must come back `15`, never `15.0`.

use serde_json::Value;

use super::super::super::super::{CEILING, PRICE, ceiling, price};
use super::text;
use crate::envelope;

/// **One money frame, composed**, or `None` for every other op.
pub(super) fn money(frame: &Value) -> Option<Value> {
    let obj = frame.as_object().expect("a gesture envelope");
    let number = |held: &Value| serde_json::from_value(held.clone()).expect("a number");
    match text(obj, envelope::OP).as_str() {
        PRICE => Some(price(
            text(obj, "provider"),
            text(obj, "model"),
            obj.get("rates").map(|held| {
                ["input", "output", "cache_read", "cache_write"].map(|name| number(&held[name]))
            }),
        )),
        CEILING => Some(ceiling(obj.get("usd").map(number))),
        _ => None,
    }
}
