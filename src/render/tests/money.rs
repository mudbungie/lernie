//! **A token count is never printed without the cost beside it** (yog REMOTE
//! §9.23, DESIGN §3.5; bl-9111) — every surface the command line paints a
//! count on, over upstream's own frames: the money where it rode, the count
//! alone where it did not, and `at least` where some tokens no rate priced.

use serde_json::Value;

use super::super::rendered;
use crate::test_support::corpus;

/// One vendored answer's frame, by its file and its index.
fn frame(name: &str, at: usize) -> Value {
    let path = corpus::root().join("answers").join(format!("{name}.json"));
    corpus::fixture(&path).frames.swap_remove(at)
}

/// A step row: priced exactly, and unpriced on the row that carried no cost.
#[test]
fn a_step_says_its_money_and_an_unpriced_one_says_its_count_alone() {
    let said = rendered(&frame("steps", 0));
    assert!(said.contains("99 tokens — $0.03"), "{said}");
    let unpriced = said
        .lines()
        .find(|line| line.contains("002"))
        .expect("the second step");
    assert!(unpriced.contains("99 tokens"), "{unpriced}");
    assert!(
        !unpriced.contains('$'),
        "no cost rode, no money: {unpriced}"
    );
}

/// A notch whose rollup holds unpriced tokens says its money as a floor.
#[test]
fn a_notch_is_a_floor_where_tokens_went_unpriced() {
    let said = rendered(&frame("rail", 0));
    assert!(said.contains("120 tokens — at least $1.20"), "{said}");
}

/// The conversation's own row, a delivery attempt, a wall's balls and the
/// roster's ledger column — each with the money the engine put there.
#[test]
fn every_other_count_carries_its_money() {
    for (name, said) in [
        ("agent", "spend 120 tokens — at least $4.00"),
        ("science", "11 in / 22 out — at least $7.50"),
        ("workspace-balls", "12 tokens — at least $2.50"),
        ("workspaces", "spent at least $12.50"),
        ("workspaces", "spent $2.50"),
    ] {
        let got = rendered(&frame(name, 0));
        assert!(got.contains(said), "{name}: {said:?} missing from {got}");
    }
    let unpriced = rendered(&frame("agent", 1));
    assert!(unpriced.contains("spend 0 tokens"), "{unpriced}");
    assert!(!unpriced.contains("spend 0 tokens —"), "{unpriced}");
}

/// **The table, the bound against the spend, and what the act woke** — and
/// the empty table saying so rather than printing nothing.
#[test]
fn the_price_table_prints_its_rows_its_bound_and_its_release() {
    let said = rendered(&frame("prices", 0));
    for clause in [
        "at least $4.25 spent of a $25 ceiling",
        "the ceiling released 2 parked conversation(s)",
        "anthropic *  in 3  out 15  cache-read 0.3  cache-write 3.75  per Mtok",
        "claude-session-direct claude-opus-4-1  in 0  out 0",
    ] {
        assert!(said.contains(clause), "{clause:?} missing from {said}");
    }
    let empty = rendered(&frame("prices", 1));
    assert!(
        empty.contains("nothing priced spent, no ceiling"),
        "{empty}"
    );
    assert!(empty.contains("no row is priced"), "{empty}");
}
