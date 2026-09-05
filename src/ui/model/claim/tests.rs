//! The claim: taken on the wall it was made on, asked nothing about, painted as
//! a row nothing observed, and spent exactly once and only where it was made.

use crate::reply::convs::{AgentState, Tone};
use crate::reply::read;
use crate::state::Standing;
use crate::test_support::window::{conv, own, seated};
use crate::ui::model::{Phase, Start};
use crate::ui::{Aim, Model};
use serde_json::json;

/// A minted-name receipt as an engine answers one.
fn minted(name: &str) -> crate::reply::Read {
    read(&json!({"ok": true, "kind": "started", "conversation": name}))
}

/// A conversations listing carrying one row under `name`.
fn listing(root: &str, name: &str) -> crate::reply::Read {
    read(&json!({"ok": true, "kind": "conversations", "rows": [
        {"root_id": root, "display": name, "name": name, "state": "quiescent",
         "uncertain": false, "preview": "", "age_secs": 0, "attention": 0,
         "members": 1, "depth": 0, "tone": "plain"}]}))
}

/// A model whose start is out on the aimed wall, with the operator's goal held.
fn firing() -> Model {
    Model {
        conversation: None,
        start: Some(Start {
            address: "home".to_owned(),
            goal: "port the paint probe".to_owned(),
            phase: Phase::Firing,
            spread: None,
        }),
        ..seated()
    }
}

/// **A start focuses what it started**, and the load-bearing half is what
/// follows: the standing set publishes no question against a name the engine
/// resolves nowhere. A claim that painted a row while the asker kept refusing
/// against it would be worse than no claim at all.
#[test]
fn the_minted_name_is_selected_and_nothing_is_asked_about_it() {
    let mut model = firing();
    model.absorb(&own().channel, minted("brisk-otter"));
    assert_eq!(model.conversation.as_deref(), Some("brisk-otter"));
    assert_eq!(model.asked(), None, "a claimed name is asked nothing");
    assert_eq!(Standing::of(&model).conversation, None);
    assert_eq!(
        Standing::of(&model).aim,
        model.aim.clone(),
        "the wall it was started on is still asked for its conversations"
    );
}

/// **The row is what nothing observed**: no lock, no completed step, flagged
/// uncertain — which is what the engine's own classifier answers for a
/// conversation it cannot probe. It must not read `live`, which would claim a
/// driver this seat has never seen, and it carries the operator's own text so
/// the goal has a painted representation for the whole round trip.
#[test]
fn the_pending_row_is_uncertain_faded_and_carries_the_goal() {
    let mut model = firing();
    model.absorb(&own().channel, minted("brisk-otter"));
    let row = model.pending().expect("a pending row");
    assert_eq!(row.state, AgentState::Quiescent);
    assert_ne!(row.state, AgentState::Live);
    assert!(row.uncertain);
    assert_eq!(row.tone, Tone::Weak);
    assert_eq!(row.preview, "port the paint probe");
    assert_eq!(row.name, None, "nothing the engine will answer to yet");
    assert_eq!(row.root_id, "brisk-otter");
    let rows = model.rows();
    assert_eq!(rows.first(), Some(&row));
    assert_eq!(rows.len(), model.convs.len() + 1);
}

/// **Spent by the answer that makes the conversation addressable**: the engine
/// hands a row its `name` exactly when a stored fact backs it, and the
/// selection moves from the minted name to the id every gesture addresses.
#[test]
fn a_row_under_the_minted_name_spends_the_claim_and_migrates_the_selection() {
    let mut model = firing();
    model.absorb(&own().channel, minted("brisk-otter"));
    model.absorb(
        &own().channel,
        listing("20260830T060000Z-e5f6", "brisk-otter"),
    );
    assert_eq!(model.conversation.as_deref(), Some("20260830T060000Z-e5f6"));
    assert_eq!(model.start, None, "no claim survives being spent");
    assert_eq!(model.pending(), None);
    assert_eq!(
        model.asked().as_deref(),
        Some("20260830T060000Z-e5f6"),
        "and the transcript question resumes against the id"
    );
}

/// **A claim is spent where it was made.** A start can take a minute to write
/// its branch, and an operator who read something else in that minute must not
/// be yanked back by their own conversation arriving — but the claim still
/// retires, because no claim survives being spent.
#[test]
fn an_operator_who_read_something_else_is_not_yarded_back() {
    let mut model = firing();
    model.absorb(&own().channel, minted("brisk-otter"));
    model.select("20260830T051200Z-a1b2");
    assert_eq!(
        model.pending(),
        None,
        "the claim is no longer the selection"
    );
    model.absorb(
        &own().channel,
        listing("20260830T060000Z-e5f6", "brisk-otter"),
    );
    assert_eq!(
        model.conversation.as_deref(),
        Some("20260830T051200Z-a1b2"),
        "what they read stands"
    );
    assert_eq!(model.start, None);
}

/// **A receipt for a wall the window is no longer looking at claims nothing.**
/// The selection is painted in the aimed wall's list and nowhere else, so a
/// selection on another wall is one the operator can neither see nor leave —
/// the name is still painted, which is the fact the engine added.
#[test]
fn a_receipt_from_a_wall_the_window_left_paints_its_name_and_takes_no_selection() {
    let mut model = firing();
    model.aim = Some(Aim {
        channel: "(this box's own engine)".to_owned(),
        address: "elsewhere".to_owned(),
    });
    model.absorb(&own().channel, minted("brisk-otter"));
    assert_eq!(model.conversation, None);
    assert_eq!(model.pending(), None);
    assert_eq!(
        model.start.as_ref().map(Start::line),
        Some("started «brisk-otter» in home".to_owned())
    );
}

/// **A claim whose row never arrives is inert**, which is what a start whose
/// driver died honestly is: a listing that carries other conversations, and
/// none of them this one, leaves the claim standing.
#[test]
fn a_listing_without_the_name_leaves_the_claim_standing() {
    let mut model = firing();
    model.absorb(&own().channel, minted("brisk-otter"));
    model.convs = vec![conv("20260830T051200Z-a1b2", "port the paint probe")];
    model.resolve();
    assert_eq!(model.conversation.as_deref(), Some("brisk-otter"));
    assert!(model.pending().is_some());
}

/// The two states that are not a claim at all: nothing started, and a start
/// still in flight. Neither may withhold the standing set's third question.
#[test]
fn a_start_that_has_not_been_named_claims_nothing() {
    let mut model = seated();
    assert_eq!(model.pending(), None);
    assert_eq!(model.asked(), model.conversation);
    model.resolve();
    assert_eq!(model.conversation.as_deref(), Some("20260830T051200Z-a1b2"));

    let mut staging = firing();
    staging.conversation = Some("20260830T051200Z-a1b2".to_owned());
    assert_eq!(staging.pending(), None);
    assert_eq!(staging.asked(), staging.conversation);
    staging.resolve();
    assert!(staging.start.is_some(), "an unnamed start is not spendable");
}
