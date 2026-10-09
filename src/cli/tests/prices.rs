//! **The money words** (REMOTE §9.23; bl-9111): what `prices`, `price` and
//! `ceiling` decide, and every way they refuse before costing a connection.

use serde_json::json;

use super::{fanned, said};
use crate::cli::Decided;
use crate::cli::verdict::REFUSED;
use crate::verbs::doors::{CEILING, PRICE};

/// **The read fans and the two acts are aimed** (bl-1bb9): `prices` is every
/// engine's table, and a row or a bound is one engine's — so the acts carry
/// the channel `--on` names, or none for the seat to choose.
#[test]
fn the_read_fans_and_the_acts_are_aimed() {
    assert_eq!(fanned(&["prices"]), json!({ "op": "prices" }));
    assert_eq!(
        aimed(&["price", "anthropic", "*", "3", "15", "0.3"]),
        (
            None,
            json!({ "op": "price", "provider": "anthropic", "model": "*",
                "rates": { "input": 3, "output": 15, "cache_read": 0.3, "cache_write": 0 } })
        )
    );
    assert_eq!(
        aimed(&["price", "anthropic", "m", "off", "--on", "alpha"]),
        (
            Some("alpha".to_owned()),
            json!({ "op": "price", "provider": "anthropic", "model": "m" })
        )
    );
    assert_eq!(
        aimed(&["ceiling", "25"]),
        (None, json!({ "op": "ceiling", "usd": 25 }))
    );
    assert_eq!(
        aimed(&["ceiling", "off", "--on", "alpha"]),
        (Some("alpha".to_owned()), json!({ "op": "ceiling" }))
    );
}

/// The channel and envelope one money act decided.
fn aimed(words: &[&str]) -> (Option<String>, serde_json::Value) {
    match crate::cli::run(words.iter().map(|w| (*w).to_owned()).collect()) {
        Decided::Aimed { on, envelope, .. } => (on, envelope),
        other => panic!("{words:?} decided {other:?}"),
    }
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
/// unknown — and a shape inside the door's range is told what it got.
#[test]
fn a_wrong_arity_earns_the_door_s_usage() {
    for (words, says) in [
        (vec!["price", "anthropic"], "argument(s)"),
        (
            vec!["price", "a", "m", "1", "2", "3", "4", "5"],
            "got 5 word(s)",
        ),
        (vec!["ceiling"], "got 0 word(s)"),
        (vec!["ceiling", "1", "2"], "got 2 word(s)"),
        (vec!["ceiling", "1", "--on"], "got 2 word(s)"),
    ] {
        let refusal = said(&words);
        assert_eq!(refusal.code, REFUSED, "{words:?}");
        assert!(refusal.text.contains(says), "{words:?}: {}", refusal.text);
    }
}
