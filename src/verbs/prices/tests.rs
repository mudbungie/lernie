//! The money acts' grammar: what the words become, and what they refuse.

use serde_json::{Number, json};

use super::{bound, ceiling, price, prices, rates};

/// **A row, as typed**: two rates fill the cache pair with the table's own
/// zero, four state all of them, and a whole number stays whole.
#[test]
fn rates_are_the_operators_own_numbers_with_zero_for_what_was_left_off() {
    let two = rates(&["3", "15"]).expect("two rates").expect("a row");
    assert_eq!(
        price("anthropic".to_owned(), "*".to_owned(), Some(two)),
        json!({ "op": "price", "provider": "anthropic", "model": "*",
                "rates": { "input": 3, "output": 15, "cache_read": 0, "cache_write": 0 } })
    );
    let four = rates(&["15", "75", "1.5", "18.75"])
        .expect("four rates")
        .expect("a row");
    assert_eq!(four[3], "18.75".parse::<Number>().expect("a number"));
}

/// **`off` is the delete**, on both acts, and absence is absence on the wire.
#[test]
fn off_deletes_a_row_and_lifts_the_bound() {
    assert_eq!(rates(&["off"]), Ok(None));
    assert_eq!(
        price("anthropic".to_owned(), "m".to_owned(), None),
        json!({ "op": "price", "provider": "anthropic", "model": "m" })
    );
    assert_eq!(bound("off"), Ok(None));
    assert_eq!(ceiling(None), json!({ "op": "ceiling" }));
    let set = bound("25").expect("a bound");
    assert_eq!(ceiling(set), json!({ "op": "ceiling", "usd": 25 }));
    assert_eq!(prices(), json!({ "op": "prices" }));
}

/// A word that is not a number, or a count of them that is no row, refuses
/// here and names what it got.
#[test]
fn a_rate_that_is_no_number_refuses_naming_the_word() {
    assert_eq!(
        rates(&["3", "lots"]),
        Err("\"lots\" is not a number of US dollars".to_owned())
    );
    assert!(
        rates(&["3"])
            .expect_err("one rate is no row")
            .contains("got 1 word(s)")
    );
    assert!(rates(&["1", "2", "3", "4", "5"]).is_err());
    assert_eq!(
        bound("much"),
        Err("\"much\" is not a number of US dollars".to_owned())
    );
}
