//! The offers page: the form, the minted URL and its symbol, the pending
//! rows, and which of a row's acts is live.

use super::{
    APPROVE, DECLINE, MINT, MINT_HEADING, MINTED, NO_WORD, NONE_PENDING, NOT_ANSWERED, PENDING,
    UNTIL, WORD, render,
};
use crate::reply::offers::{OfferRow, Offered};
use crate::test_support::window::{pane, seated};
use crate::ui::{Listing, Model};
use egui_kittest::kittest::Queryable;
use serde_json::json;

fn row(name: &str, state: &str, word: Option<&str>) -> OfferRow {
    OfferRow {
        name: name.to_owned(),
        grade: "operator".to_owned(),
        state: state.to_owned(),
        expires: 1_700_000_600,
        word: word.map(str::to_owned),
    }
}

/// A seat with the page open and three rows pending, one of each state.
pub(crate) fn offering() -> Model {
    let mut model = seated();
    model.begin_offers();
    model.pending = Some(vec![
        row("phone-1", "open", None),
        row("tablet", "claimed", Some("K7M2QX")),
        row("builder", "proved", Some("2LFTUR")),
    ]);
    model.offered(Offered {
        url: "https://engine.invalid/yog/register/ABCD2345".to_owned(),
        expires: 1_700_000_600,
    });
    model
}

fn painted(model: &mut Model) -> String {
    pane(|ui| {
        render(ui, model);
    })
}

/// A shut page paints nothing and says so.
#[test]
fn a_shut_page_paints_nothing_and_reports_it() {
    let mut model = seated();
    let mut stood = true;
    let said = pane(|ui| stood = render(ui, &mut model));
    assert!(!stood);
    assert!(!said.contains(MINT_HEADING), "{said}");
}

/// **Before the read answers the list says it is waiting, and an empty
/// answer says nothing is pending** — two facts, two sentences.
#[test]
fn the_list_says_whether_it_is_waiting_or_empty() {
    let mut model = seated();
    model.begin_offers();
    let said = painted(&mut model);
    for words in [MINT_HEADING, MINT, PENDING, NOT_ANSWERED] {
        assert!(said.contains(words), "{words:?}:\n{said}");
    }
    assert!(!said.contains(MINTED), "nothing minted yet:\n{said}");
    model.pending = Some(Vec::new());
    assert!(painted(&mut model).contains(NONE_PENDING));
}

/// **The minted URL is on the glass as text, with its expiry in words**, and
/// every pending row says its name, grade, state and word.
#[test]
fn the_minted_url_and_every_pending_row_are_said() {
    let said = painted(&mut offering());
    for words in [
        MINTED,
        "https://engine.invalid/yog/register/ABCD2345",
        &format!("{UNTIL} 22:23 UTC"),
        "phone-1  operator  open",
        NO_WORD,
        "tablet  operator  claimed",
        "K7M2QX",
        "builder  operator  proved",
        "2LFTUR",
    ] {
        assert!(said.contains(words), "{words:?}:\n{said}");
    }
}

/// **The form mints**: a grade's seat chooses it, and the button composes the
/// offer for what is typed.
#[test]
fn the_form_chooses_a_grade_and_mints() {
    let mut model = offering();
    model.offering.name = "desk".to_owned();
    let mut harness = crate::snapshot::seat(model, 1400.0, 900.0);
    harness.get_by_label("foot").click();
    harness.run();
    harness.get_by_label(MINT).click();
    harness.run();
    let posted = harness.state().outbox.last().expect("composed").clone();
    assert_eq!(posted.envelope["op"], json!("offer"));
    assert_eq!(posted.envelope["name"], json!("desk"));
    assert_eq!(posted.envelope["grade"], json!("foot"));
}

/// **Approve is live on the proved row and on no other; decline is live on
/// every row.** A click on a dark approve composes nothing.
#[test]
fn approve_is_live_only_on_a_proved_row() {
    let mut harness = crate::snapshot::seat(offering(), 1400.0, 900.0);
    assert!(harness.state().showing(Listing::Offers));
    let approves: Vec<_> = harness.query_all_by_label(APPROVE).collect();
    assert_eq!(approves.len(), 3);
    for dark in &approves[..2] {
        dark.click();
    }
    harness.run();
    assert!(harness.state().outbox.is_empty(), "neither row is proved");
    let approves: Vec<_> = harness.query_all_by_label(APPROVE).collect();
    approves[2].click();
    harness.run();
    let posted = harness.state().outbox.last().expect("composed").clone();
    assert_eq!(
        (
            posted.envelope["op"].clone(),
            posted.envelope["name"].clone()
        ),
        (json!("approve"), json!("builder"))
    );
    let declines: Vec<_> = harness.query_all_by_label(DECLINE).collect();
    declines[0].click();
    harness.run();
    let posted = harness.state().outbox.last().expect("composed").clone();
    assert_eq!(
        (
            posted.envelope["op"].clone(),
            posted.envelope["name"].clone()
        ),
        (json!("decline"), json!("phone-1"))
    );
}

/// The word that opens it is the word the menu offers.
#[test]
fn the_word_opens_the_page() {
    let mut model = seated();
    assert!(crate::ui::slash::fire(&mut model, WORD, ""));
    assert!(model.showing(Listing::Offers));
}
