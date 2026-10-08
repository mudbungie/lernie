//! The figure's reading: the counters, the money that may not be there, and
//! the attribution's two facts.

use serde_json::json;

use super::{Cost, cost, figure, priced};

/// A whole figure, read field for field.
#[test]
fn a_figure_carries_its_counters_its_money_and_what_it_sums_over() {
    let read = figure(&json!({
        "attribution": { "count": 2, "kind": "conversations", "label": "over 2 conversations" },
        "micro_usd": 2_500_000,
        "tokens": { "cache_read": 3, "cache_write": 4, "input": 1, "output": 2, "total": 10 },
        "unpriced_tokens": 3,
        "usd": "$2.50"
    }))
    .expect("a whole figure reads");
    assert_eq!(read.tokens.input, 1);
    assert_eq!(read.tokens.total, 10);
    assert_eq!(
        read.cost,
        Some(Cost {
            usd: "$2.50".to_owned(),
            unpriced_tokens: 3
        })
    );
    assert_eq!(read.attribution.kind, "conversations");
    assert_eq!(
        read.attribution.label.as_deref(),
        Some("over 2 conversations")
    );
}

/// **No money is a reading, not a zero** — a figure whose tokens no rate
/// priced says so by leaving the string out, and this seat does not compute
/// one to put in its place.
#[test]
fn a_figure_with_no_rate_carries_no_money() {
    let read = figure(&json!({
        "attribution": { "kind": "workspace", "label": "workspace-wide" },
        "tokens": { "cache_read": 0, "cache_write": 0, "input": 0, "output": 0, "total": 0 }
    }))
    .expect("an unpriced figure reads");
    assert!(read.cost.is_none());
    assert_eq!(read.attribution.kind, "workspace");
}

/// **The classification rides even where the clause does not.** A figure over
/// one stamped conversation renders as no clause at all, so the kind is what
/// tells it from workspace-wide.
#[test]
fn an_attribution_with_no_clause_still_says_what_it_sums_over() {
    let read = figure(&json!({
        "attribution": { "count": 1, "kind": "conversations" },
        "tokens": { "cache_read": 0, "cache_write": 0, "input": 0, "output": 0, "total": 0 }
    }))
    .expect("an unlabelled figure reads");
    assert_eq!(read.attribution.kind, "conversations");
    assert!(read.attribution.label.is_none());
}

/// Rung 1, and every refusal names its field.
#[test]
fn a_figure_that_is_not_one_refuses_naming_what_was_wrong() {
    assert_eq!(
        figure(&json!("figure")),
        Err("spend: not an object".to_owned())
    );
    assert_eq!(
        figure(
            &json!({ "tokens": { "cache_read": 0, "cache_write": 0, "input": 0, "output": 0, "total": 0 } })
        ),
        Err("missing or non-object field \"attribution\"".to_owned())
    );
    assert_eq!(
        figure(&json!({ "attribution": { "kind": "workspace" } })),
        Err("missing or non-object field \"tokens\"".to_owned())
    );
    assert_eq!(
        figure(&json!({
            "attribution": { "label": "x" },
            "tokens": { "cache_read": 0, "cache_write": 0, "input": 0, "output": 0, "total": 0 }
        })),
        Err("missing or non-string field \"kind\"".to_owned())
    );
}

/// **A cost under its key, both ways** — present it is the engine's money and
/// its floor count; absent it is a count no table priced, never a zero.
#[test]
fn a_cost_reads_where_it_rode_and_is_none_where_it_did_not() {
    let row = json!({ "cost": { "micro_usd": 30_000, "unpriced_tokens": 0, "usd": "$0.03" } });
    let read = cost(row.as_object().expect("an object"), "cost").expect("a cost reads");
    assert_eq!(
        read,
        Some(Cost {
            usd: "$0.03".to_owned(),
            unpriced_tokens: 0
        })
    );
    let bare = json!({});
    assert_eq!(cost(bare.as_object().expect("an object"), "cost"), Ok(None));
}

/// A cost that rode and says no money is not one the engine writes, and the
/// refusal names the key it sat under.
#[test]
fn a_present_cost_with_no_money_refuses_by_name() {
    let row = json!({ "spend": { "unpriced_tokens": 4 } });
    assert_eq!(
        cost(row.as_object().expect("an object"), "spend"),
        Err("field \"spend\": missing \"usd\"".to_owned())
    );
}

/// **The one sentence**: the count alone, the count and its money, and the
/// money as a floor where some tokens no rate priced.
#[test]
fn a_count_is_said_with_its_money_and_a_floor_says_at_least() {
    let exact = Cost {
        usd: "$0.03".to_owned(),
        unpriced_tokens: 0,
    };
    let floor = Cost {
        unpriced_tokens: 12,
        ..exact.clone()
    };
    assert_eq!(priced("99 tokens".to_owned(), None), "99 tokens");
    assert_eq!(
        priced("99 tokens".to_owned(), Some(&exact)),
        "99 tokens — $0.03"
    );
    assert_eq!(
        priced("99 tokens".to_owned(), Some(&floor)),
        "99 tokens — at least $0.03"
    );
}
