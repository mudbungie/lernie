//! **Every count the window paints carries its money** (yog REMOTE §9.23,
//! DESIGN §4.18, §4.29, §4.36, §4.41; bl-9111): the records pane's step row,
//! the spine's notch, a candidate's attempt, the roster's ledger column and a
//! ball's figure — each with the money where it rode, the count alone where it
//! did not, and `at least` where some tokens no rate priced.

use crate::reply::spend::Cost;
use crate::test_support::window::{attempt, figure, notch, step, wall};

/// The engine's money, exact or a floor.
fn cost(usd: &str, unpriced_tokens: u64) -> Cost {
    Cost {
        usd: usd.to_owned(),
        unpriced_tokens,
    }
}

/// The records pane's step row (§4.18).
#[test]
fn a_step_row_says_its_money() {
    let mut row = step("001");
    assert_eq!(
        crate::ui::records::headline(&row),
        "001  complete — 99 tokens"
    );
    row.cost = Some(cost("$0.03", 0));
    assert_eq!(
        crate::ui::records::headline(&row),
        "001  complete — 99 tokens — $0.03"
    );
    row.cost = Some(cost("$0.03", 5));
    assert_eq!(
        crate::ui::records::headline(&row),
        "001  complete — 99 tokens — at least $0.03"
    );
}

/// The spine's notch (§4.29).
#[test]
fn a_notch_says_its_money() {
    let mut row = notch("001");
    let headline = crate::ui::records::spine::headline;
    assert_eq!(headline(&row), "001  abcdef1 — 120 tokens");
    row.cost = Some(cost("$1.20", 0));
    assert_eq!(headline(&row), "001  abcdef1 — 120 tokens — $1.20");
    row.cost = Some(cost("$1.20", 20));
    assert_eq!(headline(&row), "001  abcdef1 — 120 tokens — at least $1.20");
}

/// A candidate's attempt on the fleet pane (§4.36).
#[test]
fn a_candidate_s_attempt_says_its_money() {
    let counts = "4 steps  90s  11 in  22 out  33 cache-read  44 cache-write";
    let mut row = attempt("bl-1", "pending");
    assert!(crate::ui::fleet::attempt(&row).contains(&counts.to_owned()));
    row.cost = Some(cost("$7.50", 0));
    assert!(crate::ui::fleet::attempt(&row).contains(&format!("{counts} — $7.50")));
    row.cost = Some(cost("$7.50", 3));
    assert!(crate::ui::fleet::attempt(&row).contains(&format!("{counts} — at least $7.50")));
}

/// **The roster's ledger column** (§4.41): each wall's whole spend, after
/// its state and before the weak facts — and nothing where it is unpriced.
#[test]
fn the_roster_is_a_ledger() {
    let mut row = wall("home");
    assert_eq!(
        crate::ui::roster::line(&row),
        "home  2 conversations  (named)"
    );
    row.spend = Some(cost("$12.50", 0));
    assert_eq!(
        crate::ui::roster::line(&row),
        "home  $12.50  2 conversations  (named)"
    );
    row.spend = Some(cost("$12.50", 2));
    assert_eq!(
        crate::ui::roster::line(&row),
        "home  at least $12.50  2 conversations  (named)"
    );
}

/// A ball's figure on the board says a floor as one.
#[test]
fn a_ball_s_figure_says_a_floor_as_one() {
    let mut held = figure(Some("$1.50"), None);
    held.cost = Some(cost("$1.50", 4));
    assert_eq!(
        crate::ui::board::cost(&held),
        "at least $1.50  99 tokens  conversations"
    );
}
