//! **The money words** (REMOTE §9.23; bl-9111): what `prices`, `price` and
//! `ceiling` decide, and every way they refuse before costing a connection.

use serde_json::json;

use super::{fanned, said};
use crate::cli::verdict::REFUSED;
use crate::verbs::doors::{CEILING, PRICE};

/// **All three fan**: none names a workspace, so each one's subject is every
/// channel this box holds — `lernie ack`'s shape.
#[test]
fn the_money_words_fan_their_envelopes() {
    assert_eq!(fanned(&["prices"]), json!({ "op": "prices" }));
    assert_eq!(
        fanned(&["price", "anthropic", "*", "3", "15", "0.3"]),
        json!({ "op": "price", "provider": "anthropic", "model": "*",
                "rates": { "input": 3, "output": 15, "cache_read": 0.3, "cache_write": 0 } })
    );
    assert_eq!(
        fanned(&["price", "anthropic", "m", "off"]),
        json!({ "op": "price", "provider": "anthropic", "model": "m" })
    );
    assert_eq!(
        fanned(&["ceiling", "25"]),
        json!({ "op": "ceiling", "usd": 25 })
    );
    assert_eq!(fanned(&["ceiling", "off"]), json!({ "op": "ceiling" }));
}

/// **A word that is not a number is the typist's**, read here and answered
/// with the door's usage — and a trailing `--json` is told where it goes.
#[test]
fn a_rate_or_bound_that_is_no_number_earns_the_door_s_usage() {
    let refusal = said(&["ceiling", "lots"]);
    assert_eq!(refusal.code, REFUSED);
    assert!(refusal.text.contains("\"lots\""), "{}", refusal.text);
    assert!(refusal.text.contains(&CEILING.usage()), "{}", refusal.text);
    let refusal = said(&["price", "anthropic", "*", "3"]);
    assert!(
        refusal.text.contains("two to four rates"),
        "{}",
        refusal.text
    );
    assert!(refusal.text.contains(&PRICE.usage()), "{}", refusal.text);
    let refusal = said(&["price", "anthropic", "*", "--json"]);
    assert!(
        refusal.text.contains("`--json` goes BEFORE the word"),
        "{}",
        refusal.text
    );
}

/// **A wrong arity is told what the door takes**, not that the word is
/// unknown.
#[test]
fn a_wrong_arity_earns_the_door_s_range() {
    for words in [
        vec!["price", "anthropic"],
        vec!["price", "a", "m", "1", "2", "3", "4", "5"],
        vec!["ceiling"],
        vec!["ceiling", "1", "2"],
    ] {
        let refusal = said(&words);
        assert_eq!(refusal.code, REFUSED, "{words:?}");
        assert!(
            refusal.text.contains("argument(s)"),
            "{words:?}: {}",
            refusal.text
        );
    }
}
