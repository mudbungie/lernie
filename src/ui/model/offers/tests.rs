//! The offers page's state: when it opens, what its acts compose, and what
//! goes with the aim.

use super::{Grade, Offering};
use crate::reply::offers::{OfferRow, Offered};
use crate::test_support::window::seated;
use crate::ui::{Listing, Model, Page};
use serde_json::json;

fn offered() -> Offered {
    Offered {
        url: "https://engine.invalid/yog/register/ABCD2345".to_owned(),
        expires: 1_700_000_600,
    }
}

/// The page opens on an aim and not without one; Escape puts it down.
#[test]
fn the_page_opens_on_an_aim_and_escape_closes_it() {
    let mut bare = Model::default();
    bare.begin_offers();
    assert!(!bare.showing(Listing::Offers));
    let mut model = seated();
    model.begin_offers();
    assert_eq!(model.page(), Some(Page::Offers));
    model.escape();
    assert_eq!(model.page(), None);
}

/// **The mint composes `offer`** for the aimed workspace, with the name
/// trimmed and the grade's own word — and nothing while no name is typed.
#[test]
fn the_mint_composes_the_offer_for_the_aimed_workspace() {
    let mut model = seated();
    let workspace = model.aim.clone().expect("aimed").address;
    model.post_offer();
    assert!(model.outbox.is_empty(), "no name typed");
    model.offering.name = "  phone-1 ".to_owned();
    model.offering.grade = Grade::Foot;
    model.post_offer();
    let posted = model.outbox.pop().expect("composed");
    assert!(posted.act);
    assert_eq!(
        posted.envelope,
        json!({"op": "offer", "workspace": workspace, "name": "phone-1", "grade": "foot"})
    );
    assert_eq!(Grade::default().word(), "operator");
    let mut unaimed = Model::default();
    unaimed.offering.name = "phone-1".to_owned();
    unaimed.post_offer();
    assert!(unaimed.outbox.is_empty(), "nothing aimed");
}

/// **The minted offer is held with its symbol and the name is spent.** A URL
/// too long for a symbol is still shown, as text alone.
#[test]
fn a_minted_offer_is_held_with_its_symbol() {
    let mut model = seated();
    model.offering.name = "phone-1".to_owned();
    model.absorb(
        &crate::test_support::window::own().channel,
        crate::reply::Read::Answer(crate::reply::Reply::Offered(offered())),
    );
    let minted = model.offering.minted.clone().expect("held");
    assert_eq!(minted.offered, offered());
    assert!(minted.symbol.is_some());
    assert_eq!(model.offering.name, "");
    model.offered(Offered {
        url: "x".repeat(4000),
        expires: 0,
    });
    assert!(model.offering.minted.expect("held").symbol.is_none());
}

/// **Approve and decline each compose their own verb**, naming the row.
#[test]
fn a_verdict_composes_its_own_verb() {
    let mut model = seated();
    let workspace = model.aim.clone().expect("aimed").address;
    for (approve, op) in [(true, "approve"), (false, "decline")] {
        model.post_verdict("tablet", approve);
        let posted = model.outbox.pop().expect("composed");
        assert!(posted.act);
        assert_eq!(
            posted.envelope,
            json!({"op": op, "workspace": workspace, "name": "tablet"})
        );
    }
    let mut unaimed = Model::default();
    unaimed.post_verdict("tablet", true);
    assert!(unaimed.outbox.is_empty());
}

/// **The page, its rows and its minted URL go with the aim**: all three are
/// one workspace's.
#[test]
fn the_page_goes_with_the_workspace_it_was_about() {
    let mut model = seated();
    model.begin_offers();
    model.offered(offered());
    model.pending = Some(vec![OfferRow {
        name: "tablet".to_owned(),
        grade: "operator".to_owned(),
        state: "open".to_owned(),
        expires: 0,
        word: None,
    }]);
    model.retire_offers();
    assert!(!model.showing(Listing::Offers));
    assert_eq!(model.pending, None);
    assert_eq!(model.offering, Offering::default());
}
