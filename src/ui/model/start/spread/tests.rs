//! A start spread over n: what it stages, what its two receipts compose, and
//! the gate it shares with the single start.

use super::Spread;
use crate::reply::start::Prepared;
use crate::test_support::window::seated;
use crate::ui::Model;
use crate::ui::model::{Phase, Start};
use serde_json::json;

fn over() -> Spread {
    Spread {
        ball: "bl-1".to_owned(),
        project: "p".to_owned(),
        n: 3,
    }
}

fn staged_body() -> Prepared {
    Prepared {
        workspace: "there".to_owned(),
        goal: String::new(),
        body: json!({"binding": null, "goal": "", "lineage": null,
                     "origin": "world", "workspace": "there"}),
    }
}

/// **The whole chain**: the spread stages the ordinary `prepare`, its receipt
/// composes the `fan` carrying the obligation, and the fan's own answer
/// composes one `prompt` per candidate — all of it by the frame that took the
/// previous receipt, which is §4.26's argument read over n.
#[test]
fn a_spread_is_a_prepare_then_a_fan_then_one_prompt_per_candidate() {
    let mut model = seated();
    model.stage_spread("home", over(), "try three ways");
    assert_eq!(model.outbox.len(), 1);
    assert_eq!(model.outbox[0].envelope["op"], "prepare");
    model.outbox.clear();

    model.staged(&staged_body());
    let fanned = model.outbox.first().expect("the fan");
    assert_eq!(fanned.envelope["op"], "fan");
    assert_eq!(fanned.envelope["ball"], "bl-1");
    assert_eq!(fanned.envelope["n"], 3);
    assert_eq!(
        fanned.envelope["prepared"]["workspace"], "home",
        "re-addressed into this box's spelling"
    );
    model.outbox.clear();

    model.fanned(vec![staged_body(), staged_body()]);
    assert_eq!(model.outbox.len(), 2, "one fire per candidate");
    for posted in &model.outbox {
        assert_eq!(posted.envelope["op"], "prompt");
        assert_eq!(posted.envelope["goal"], "try three ways");
        assert_eq!(posted.envelope["prepared"]["workspace"], "home");
    }
}

/// **A staged receipt with no spread held is the ordinary single start**, which
/// is what makes this one value rather than two: the reply is the same either
/// way and the intent is what tells them apart.
#[test]
fn a_staged_receipt_with_no_spread_fires_the_single_start() {
    let mut model = seated();
    model.draft = "do the thing".to_owned();
    model.stage("home");
    model.outbox.clear();
    model.staged(&staged_body());
    assert_eq!(
        model.outbox.first().map(|p| p.envelope["op"].clone()),
        Some(json!("prompt"))
    );
}

/// **One is outstanding at a time**, and the gate is [`Start`]'s existing one:
/// two `prepare` acts in flight cannot be told apart, because the receipt
/// carries no correlation with the gesture that earned it.
#[test]
fn a_spread_refuses_while_a_start_is_outstanding() {
    let mut model = Model {
        start: Some(Start {
            address: "home".to_owned(),
            goal: "already going".to_owned(),
            phase: Phase::Staging,
            spread: None,
        }),
        ..seated()
    };
    model.stage_spread("home", over(), "and another");
    assert!(model.outbox.is_empty());
    assert_eq!(model.start.and_then(|start| start.spread), None);
}

/// **An empty goal stages nothing**, for the single start's reason over n.
#[test]
fn a_spread_with_no_goal_stages_nothing() {
    let mut model = seated();
    model.stage_spread("home", over(), "   ");
    assert!(model.outbox.is_empty());
    assert_eq!(model.start, None);
}

/// **A fanned answer this window did not ask for still fires**, on the
/// candidate's own terms: nothing that arrives is dropped, and each row is
/// addressed by the name it came back under and fired with its own goal.
#[test]
fn a_fanned_answer_nobody_here_asked_for_fires_on_its_own_terms() {
    let mut model = seated();
    let mut row = staged_body();
    row.goal = "the rung's own prefill".to_owned();
    model.fanned(vec![row]);
    let posted = model.outbox.first().expect("one fire");
    assert_eq!(posted.envelope["goal"], "the rung's own prefill");
    assert_eq!(posted.envelope["prepared"]["workspace"], "there");
}

/// And a candidate that prefills nothing is not fired at all — the bare rung's
/// empty goal, which is the same predicate the single start spends.
#[test]
fn a_candidate_with_no_goal_at_all_is_not_fired() {
    let mut model = seated();
    model.fanned(vec![staged_body()]);
    assert!(model.outbox.is_empty());
}

/// **A spread's receipts select nothing.** *A start focuses what it started*
/// is a sentence about one conversation; n receipts land one after another and
/// the focus would be whichever arrived last, which is a fact about the
/// network rather than about the operator.
#[test]
fn a_spread_s_receipts_focus_no_conversation() {
    let mut model = Model {
        conversation: None,
        ..seated()
    };
    model.stage_spread("home", over(), "try three ways");
    model.started("20260905T000000Z-c1".to_owned());
    assert_eq!(model.conversation, None);
    assert!(
        model
            .start
            .as_ref()
            .is_some_and(|start| start.line().contains("one candidate of")),
        "{:?}",
        model.start
    );
}
