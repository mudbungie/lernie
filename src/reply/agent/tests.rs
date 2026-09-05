//! The conversation's own row: every optional field read as an absence, the
//! two shapes shared with other readings, and the strictness that names one.

use serde_json::json;

use super::agent;
use crate::reply::convs::AgentState;

/// A row with every optional fact stated — the busy conversation.
fn busy() -> serde_json::Value {
    json!({
        "agent": "r-0-c-1", "root": "r-0", "ancestors": ["r-0"],
        "display": "pennant", "display_only": true, "tip": "aaaa",
        "state": "in-flight", "refused": false,
        "marks": ["notified", "held"], "flight": "tools",
        "held": {"tool": "Bash", "tool_use_id": "toolu_1", "reason": "unconfined"},
        "present": true, "nudgeable": false, "stoppable": true, "stop_children": true,
        "seats": [{"name": "pennant", "doing": "waiting"}],
        "strip": {"class": "tools", "facts": "Bash · 5s"},
        "spend": {
            "tokens": {"input": 120, "output": 0, "cache_read": 0,
                       "cache_write": 0, "total": 120},
            "micro_usd": 4_000_000, "usd": "$4.00", "unpriced_tokens": 1,
            "attribution": {"kind": "conversations", "count": 3,
                            "label": "over 3 conversations"}
        },
        "context": {"model": "claude-x", "prompt_tokens": 4000,
                    "window": 200_000, "percent": 2}
    })
}

/// **The busy row reads whole**, including the two shapes shared with other
/// readings and the id spelling that is this encoder's own.
#[test]
fn a_busy_conversation_carries_every_fact_the_engine_holds() {
    let read = agent(busy().as_object().expect("an object")).expect("a whole row reads");
    assert_eq!(read.state, AgentState::InFlight);
    assert_eq!(read.ancestors, ["r-0"]);
    assert_eq!(read.marks, ["notified", "held"]);
    assert_eq!(read.flight.as_deref(), Some("tools"));
    let held = read.held.expect("a parked invocation");
    assert_eq!(held.tool_use, "toolu_1", "litany's own key spelling");
    assert_eq!(read.seats[0].doing, "waiting");
    assert_eq!(
        read.offers,
        [
            crate::reply::agent::Offer::Stop,
            crate::reply::agent::Offer::Children
        ],
        "the gates read as the set they describe, in the engine's own order"
    );
    assert_eq!(read.strip.expect("a strip").facts, "Bash · 5s");
    assert_eq!(read.spend.tokens.total, 120);
    assert_eq!(read.spend.usd.as_deref(), Some("$4.00"));
    assert_eq!(
        read.spend.attribution.label.as_deref(),
        Some("over 3 conversations")
    );
    let full = read.context.expect("a measured context");
    assert_eq!(full.percent, 2, "the engine's own rounding, carried");
    assert_eq!(full.window, 200_000);
}

/// **The row at rest states none of them, and each absence is a reading.** An
/// empty list is not a fact the encoder declined to state, so the encoder
/// declines by leaving the key out and this reads it as the same thing.
#[test]
fn a_resting_conversation_states_none_of_the_optional_facts() {
    let read = agent(
        json!({
            "agent": "r-0", "root": "r-0", "display": "r-0", "display_only": false,
            "tip": "", "state": "stopped", "refused": true,
            "failure": "no credential for provider row \"work\"",
            "present": false, "nudgeable": false, "stoppable": false,
            "stop_children": false,
            "spend": {
                "tokens": {"input": 0, "output": 0, "cache_read": 0,
                           "cache_write": 0, "total": 0},
                "attribution": {"kind": "workspace", "label": "workspace-wide"}
            }
        })
        .as_object()
        .expect("an object"),
    )
    .expect("a resting row reads");
    assert!(read.ancestors.is_empty());
    assert!(read.marks.is_empty());
    assert!(read.flight.is_none());
    assert!(read.held.is_none());
    assert!(read.seats.is_empty());
    assert!(read.strip.is_none());
    assert!(read.context.is_none());
    assert!(read.spend.usd.is_none(), "no price table, no money");
    assert!(
        read.offers.is_empty(),
        "a stopped conversation is offered nothing"
    );
    assert_eq!(
        read.failure.as_deref(),
        Some("no credential for provider row \"work\"")
    );
    assert!(read.refused);
}

/// Rung 3: a state, a flight class, a doing word and an attribution kind this
/// build has never seen each paint as themselves.
#[test]
fn vocabulary_this_build_does_not_know_rides_verbatim() {
    let mut frame = busy();
    frame["state"] = json!("sideways");
    frame["flight"] = json!("dowsing");
    frame["seats"] = json!([{"name": "n", "doing": "dowsing"}]);
    frame["spend"]["attribution"] = json!({"kind": "galaxies"});
    let read =
        agent(frame.as_object().expect("an object")).expect("unknown words are not refusals");
    assert_eq!(read.state, AgentState::Unknown("sideways".to_owned()));
    assert_eq!(read.flight.as_deref(), Some("dowsing"));
    assert_eq!(read.seats[0].doing, "dowsing");
    assert_eq!(read.spend.attribution.kind, "galaxies");
    assert!(read.spend.attribution.label.is_none());
}

/// Rung 1: a missing or mistyped field refuses, and the refusal names it.
#[test]
fn a_malformed_row_refuses_and_names_the_field() {
    let why = agent(json!({"agent": "c-1"}).as_object().expect("an object"))
        .expect_err("a row with no root refuses");
    assert!(why.contains("root"), "{why}");

    let mut frame = busy();
    frame.as_object_mut().expect("an object").remove("spend");
    let why =
        agent(frame.as_object().expect("an object")).expect_err("a row with no spend refuses");
    assert!(why.contains("spend"), "{why}");

    let mut frame = busy();
    frame["marks"] = json!([7]);
    let why = agent(frame.as_object().expect("an object")).expect_err("a non-string mark refuses");
    assert!(why.contains("non-string"), "{why}");

    let mut frame = busy();
    frame["seats"] = json!(["a string"]);
    let why = agent(frame.as_object().expect("an object"))
        .expect_err("a seat that is not an object refuses");
    assert!(why.contains("not an object"), "{why}");

    let mut frame = busy();
    frame
        .as_object_mut()
        .expect("an object")
        .remove("stoppable");
    let why = agent(frame.as_object().expect("an object"))
        .expect_err("a row missing one gate refuses rather than offering less");
    assert!(why.contains("stoppable"), "{why}");

    let mut frame = busy();
    frame["held"] = json!("a string");
    let why = agent(frame.as_object().expect("an object")).expect_err("a non-object held refuses");
    assert!(why.contains("held"), "{why}");

    let mut frame = busy();
    frame["spend"]["attribution"] = json!("a string");
    let why = agent(frame.as_object().expect("an object"))
        .expect_err("a figure with no attribution object refuses");
    assert!(why.contains("attribution"), "{why}");
}
