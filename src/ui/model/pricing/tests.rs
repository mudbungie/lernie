//! The prices pane's model: what opening asks, how an answer files, and when
//! each act is live and where it goes.

use serde_json::json;

use super::{NEEDS_BOUND, NEEDS_ROW};
use crate::test_support::window::panes::prices::table;
use crate::test_support::window::{own, priced, provider, seated};
use crate::ui::{Channel, Model, Posted};

/// The seated model, drafting a row for `housevendor`.
fn drafting(rates: &str) -> Model {
    let mut model = priced();
    model.pricing.provider = "housevendor".to_owned();
    model.pricing.model = "*".to_owned();
    model.pricing.rates = rates.to_owned();
    model
}

/// **Opening asks every channel's table, and the aimed workspace's roster** — the
/// one the price control checks a row against. An unaimed seat asks the table
/// alone.
#[test]
fn opening_asks_the_table_and_the_aimed_workspace_s_providers() {
    let mut model = seated();
    model.begin_prices();
    assert!(model.pricing());
    assert_eq!(
        model.outbox,
        vec![
            Posted::read(json!({ "op": "prices" })),
            Posted::read(json!({ "op": "providers", "workspace": "home" })),
        ]
    );
    let mut unaimed = Model::default();
    unaimed.begin_prices();
    assert_eq!(
        unaimed.outbox,
        vec![Posted::read(json!({ "op": "prices" }))]
    );
}

/// **An answer replaces its own channel's section** and leaves another
/// standing — the receipt of an act included.
#[test]
fn an_answer_replaces_its_own_channel_only() {
    let mut model = seated();
    model.absorb(
        &own().channel,
        crate::reply::Read::Answer(crate::reply::Reply::Prices(table())),
    );
    assert_eq!(model.pricing.tables.len(), 1, "filed through the one door");
    let other = Channel {
        name: "lab".to_owned(),
        ..own().channel
    };
    model.priced(&other, table());
    let mut emptied = table();
    emptied.rows.clear();
    model.priced(&own().channel, emptied);
    assert_eq!(model.pricing.tables.len(), 2);
    assert!(model.pricing.tables[0].prices.rows.is_empty());
    assert_eq!(model.pricing.tables[1].prices.rows.len(), 2);
}

/// **The price control is live only on a whole row**, and says why not.
#[test]
fn the_price_control_says_what_would_make_it_live() {
    let channel = own().channel;
    assert_eq!(priced().unpriceable(&channel).as_deref(), Some(NEEDS_ROW));
    assert!(
        drafting("3")
            .unpriceable(&channel)
            .expect("one rate is no row")
            .contains("two to four rates")
    );
    assert_eq!(drafting("3 15").unpriceable(&channel), None);
}

/// **The roster check runs where this seat holds the roster**: on the aimed
/// workspace's channel, against what that workspace answered — and nowhere else, where
/// the server's own refusal names it.
#[test]
fn a_provider_the_aimed_workspace_does_not_name_disables_the_control() {
    let channel = own().channel;
    let mut model = drafting("3 15");
    model.providers = Some(vec![provider("otherhouse")]);
    assert_eq!(
        model.unpriceable(&channel).as_deref(),
        Some("home has no provider row \"housevendor\" — the server would refuse it")
    );
    model.providers = Some(vec![provider("housevendor")]);
    assert_eq!(model.unpriceable(&channel), None);
    model.providers = Some(vec![provider("otherhouse")]);
    let elsewhere = Channel {
        name: "lab".to_owned(),
        ..own().channel
    };
    assert_eq!(model.unpriceable(&elsewhere), None);
}

/// **A row on another machine is priced by its qualified name** (yog's
/// `docs/REMOTE.md` §5.7 — the price table keys on the name a step records):
/// the roster check reads the table's own spelling, so the bare row beside it
/// does not stand in, and the act carries the name whole.
#[test]
fn a_row_on_another_machine_is_priced_by_its_qualified_name() {
    let channel = own().channel;
    let mut model = drafting("3 15");
    model.pricing.provider = "housevendor@attic".to_owned();
    model.providers = Some(vec![provider("housevendor")]);
    assert!(
        model.unpriceable(&channel).is_some(),
        "the bare row is another row"
    );
    model.providers = Some(vec![provider("housevendor"), provider("housevendor@attic")]);
    assert_eq!(model.unpriceable(&channel), None);
    model.post_price(&channel);
    model.post_unprice(&channel, "housevendor@attic", "*");
    let sent: Vec<&str> = model
        .outbox
        .iter()
        .filter_map(|posted| posted.envelope["provider"].as_str())
        .collect();
    assert_eq!(sent, vec!["housevendor@attic", "housevendor@attic"]);
}

/// **Each act goes down the channel its section is**, and a control that is
/// not live posts nothing.
#[test]
fn the_acts_go_down_one_channel_and_a_dead_one_posts_nothing() {
    let channel = own().channel;
    let mut idle = priced();
    idle.post_price(&channel);
    idle.post_ceiling(&channel);
    assert!(idle.outbox.is_empty());

    let mut model = drafting("3 15");
    model.pricing.bound = "25".to_owned();
    model.post_price(&channel);
    model.post_unprice(&channel, "subscription", "*");
    model.post_ceiling(&channel);
    model.post_lift(&channel);
    let row = json!({ "op": "price", "provider": "housevendor", "model": "*",
        "rates": { "input": 3, "output": 15, "cache_read": 0, "cache_write": 0 } });
    assert_eq!(
        model.outbox,
        vec![
            Posted::act(row).down(channel.clone()),
            Posted::act(json!({ "op": "price", "provider": "subscription", "model": "*" }))
                .down(channel.clone()),
            Posted::act(json!({ "op": "ceiling", "usd": 25 })).down(channel.clone()),
            Posted::act(json!({ "op": "ceiling" })).down(channel),
        ]
    );
}

/// **The ceiling box says what it needs**: a number or `off`, and nothing is
/// no bound at all.
#[test]
fn the_ceiling_control_says_what_would_make_it_live() {
    let mut model = priced();
    assert_eq!(model.unbounded().as_deref(), Some(NEEDS_BOUND));
    model.pricing.bound = "lots".to_owned();
    assert_eq!(
        model.unbounded().as_deref(),
        Some("\"lots\" is not a number of US dollars")
    );
    model.pricing.bound = "off".to_owned();
    assert_eq!(model.unbounded(), None);
    model.post_ceiling(&own().channel);
    assert_eq!(
        model.outbox,
        vec![Posted::act(json!({ "op": "ceiling" })).down(own().channel)]
    );
}
