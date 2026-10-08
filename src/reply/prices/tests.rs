//! The table's reading: every row and rate, the three absences, and the two
//! sentences the surfaces say it in.

use serde_json::{Value, json};

use super::{Prices, prices};
use crate::reply::spend::Cost;

fn read(value: &Value) -> Result<Prices, String> {
    prices(value.as_object().expect("an object"))
}

/// A whole receipt: two rows (a wildcard and a subscription), the bound, the
/// spend as a floor, and what the ceiling act woke.
#[test]
fn a_whole_table_reads_with_its_bound_spend_and_release() {
    let table = read(&json!({
        "kind": "prices", "ok": true, "ceiling": 25, "released": 2,
        "rows": [
            { "provider": "anthropic", "model": "*", "input": 3, "output": 15,
              "cache_read": 0.3, "cache_write": 3.75 },
            { "provider": "sub", "model": "m", "input": 0, "output": 0,
              "cache_read": 0, "cache_write": 0 }
        ],
        "spent": { "micro_usd": 4_250_000, "unpriced_tokens": 12, "usd": "$4.25" }
    }))
    .expect("a whole table reads");
    assert_eq!(table.rows.len(), 2);
    assert_eq!(
        table.rows[0].said(),
        "anthropic *  in 3  out 15  cache-read 0.3  cache-write 3.75  per Mtok"
    );
    assert_eq!(
        table.rows[1].said(),
        "sub m  in 0  out 0  cache-read 0  cache-write 0  per Mtok"
    );
    assert_eq!(table.ceiling, Some(25.into()));
    assert_eq!(
        table.spent,
        Some(Cost {
            usd: "$4.25".to_owned(),
            unpriced_tokens: 12
        })
    );
    assert_eq!(table.standing(), "at least $4.25 spent of a $25 ceiling");
    assert_eq!(
        table.woke().as_deref(),
        Some("the ceiling released 2 parked conversation(s)")
    );
}

/// **The three absences are readings**: no bound, no priced spend, and a
/// receipt that woke nothing — each said rather than zeroed.
#[test]
fn an_empty_table_reads_its_absences_as_facts() {
    let table =
        read(&json!({ "kind": "prices", "ok": true, "rows": [] })).expect("an empty table reads");
    assert!(table.rows.is_empty());
    assert_eq!(table.ceiling, None);
    assert_eq!(table.spent, None);
    assert_eq!(table.woke(), None);
    assert_eq!(table.standing(), "nothing priced spent, no ceiling");
    let nulled = read(&json!({ "rows": [], "ceiling": null })).expect("a null bound reads");
    assert_eq!(nulled.ceiling, None);
}

/// Rung 1, and every refusal names its field.
#[test]
fn a_table_that_is_not_one_refuses_naming_what_was_wrong() {
    assert_eq!(
        read(&json!({ "rows": ["x"] })),
        Err("price row: not an object".to_owned())
    );
    assert_eq!(
        read(&json!({ "rows": [{ "provider": "p", "model": "m", "input": "3" }] })),
        Err("missing or non-number field \"input\"".to_owned())
    );
    assert_eq!(
        read(&json!({ "rows": [], "ceiling": "lots" })),
        Err("missing or non-number field \"ceiling\"".to_owned())
    );
}
