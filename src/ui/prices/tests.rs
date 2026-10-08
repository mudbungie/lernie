//! The prices pane: the control that opens it, every sentence a section can
//! say, and each act's click reaching the model.

/// Every other surface that paints a token count, and its money.
mod surfaces;

use serde_json::json;

use super::{HEADING, LIFT, NO_ROWS, NOT_ANSWERED, OPEN, PRICE, SET, UNPRICE};
use crate::paint_probe::frame::Window;
use crate::test_support::window::{click, own, painted, priced};
use crate::ui::{Model, Posted};

/// **The roster's control opens the pane**, from an unaimed seat.
#[test]
fn the_roster_s_control_opens_the_pane() {
    let mut model = Model::default();
    let window = Window::new();
    click(&window, OPEN, |ctx| crate::ui::render(ctx, &mut model));
    assert!(model.pricing(), "the control opened nothing");
    assert!(painted(&mut model).contains(HEADING));
}

/// **Every sentence a section says**: the bound and the spend against it, a
/// floor said as one, what the act woke, each row with its rates — and the
/// two disabled controls' reasons beside them.
#[test]
fn a_section_says_its_table_its_bound_and_why_an_act_is_not_live() {
    let mut model = priced();
    let glass = painted(&mut model);
    for said in [
        "at least $4.25 spent of a $25 ceiling",
        "the ceiling released 2 parked conversation(s)",
        "housevendor *  in 3  out 15  cache-read 0  cache-write 0  per Mtok",
        "subscription *  in 0  out 0",
        crate::ui::model::pricing::NEEDS_ROW,
        crate::ui::model::pricing::NEEDS_BOUND,
        LIFT,
    ] {
        assert!(glass.contains(said), "{said:?} missing from {glass}");
    }
}

/// **The two emptinesses**: nobody has answered, and an engine that prices
/// nothing — which is a fact about its counts, not a wait.
#[test]
fn each_emptiness_says_which_one_it_is() {
    let mut unheard = Model {
        pricing: crate::ui::Pricing::default(),
        ..priced()
    };
    assert!(painted(&mut unheard).contains(NOT_ANSWERED));
    let mut bare = priced();
    bare.pricing.tables[0].prices = crate::reply::prices::Prices {
        rows: Vec::new(),
        ceiling: None,
        spent: None,
        released: None,
    };
    let glass = painted(&mut bare);
    assert!(glass.contains(NO_ROWS), "{glass}");
    assert!(
        glass.contains("nothing priced spent, no ceiling"),
        "{glass}"
    );
    assert!(!glass.contains(LIFT), "no bound, nothing to lift: {glass}");
}

/// **Each control reaches its act**, down the section's channel.
#[test]
fn each_control_fires_its_act_down_the_section_s_channel() {
    let channel = own().channel;
    let fired = |label: &str, model: &mut Model| {
        let window = Window::new();
        click(&window, label, |ctx| crate::ui::render(ctx, model));
        model.outbox.pop()
    };
    let mut model = priced();
    model.pricing.provider = "housevendor".to_owned();
    model.pricing.model = "m".to_owned();
    model.pricing.rates = "3 15".to_owned();
    model.pricing.bound = "30".to_owned();
    model.providers = None;
    let row = json!({ "op": "price", "provider": "housevendor", "model": "m",
        "rates": { "input": 3, "output": 15, "cache_read": 0, "cache_write": 0 } });
    assert_eq!(
        fired(PRICE, &mut model),
        Some(Posted::act(row).down(channel.clone()))
    );
    assert_eq!(
        fired(UNPRICE, &mut model),
        Some(
            Posted::act(json!({ "op": "price", "provider": "housevendor", "model": "*" }))
                .down(channel.clone())
        )
    );
    assert_eq!(
        fired(SET, &mut model),
        Some(Posted::act(json!({ "op": "ceiling", "usd": 30 })).down(channel.clone()))
    );
    assert_eq!(
        fired(LIFT, &mut model),
        Some(Posted::act(json!({ "op": "ceiling" })).down(channel))
    );
}
