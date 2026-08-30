//! The chain: staged, fired, received — and what each receipt does to the model.

use super::{Phase, Start};
use crate::reply::read;
use crate::test_support::window::{own, seated};
use crate::ui::{Aim, Model};
use serde_json::json;

/// A staging receipt as an engine answers one.
fn staged(workspace: &str, goal: &str) -> crate::reply::Read {
    read(&json!({"ok": true, "kind": "prepared",
                 "prepared": {"workspace": workspace, "goal": goal}}))
}

/// **The whole chain in the model.** A goal staged composes the first act and
/// holds the text; the first act's receipt composes the second from it; the
/// second's receipt is the minted name.
#[test]
fn a_start_is_two_acts_and_the_second_is_composed_from_the_first_s_answer() {
    let mut model = Model {
        conversation: None,
        draft: "do the thing".to_owned(),
        ..seated()
    };
    model.stage("home");
    assert_eq!(model.outbox, vec![crate::verbs::prepare("home".to_owned())]);
    assert_eq!(model.draft, "", "what was staged is no longer a draft");
    assert_eq!(
        model.start.as_ref().map(|start| start.phase.clone()),
        Some(Phase::Staging)
    );
    model.absorb(&own().channel, staged("home", ""));
    assert_eq!(model.outbox.len(), 2, "the fire is composed, not posted");
    assert_eq!(model.outbox[1]["op"], json!("prompt"));
    assert_eq!(model.outbox[1]["goal"], json!("do the thing"));
    model.absorb(
        &own().channel,
        read(&json!({"ok": true, "kind": "started", "conversation": "brisk-otter"})),
    );
    let start = model.start.as_ref().expect("the start");
    assert_eq!(start.phase, Phase::Started("brisk-otter".to_owned()));
    assert!(!start.outstanding());
    assert_eq!(start.line(), "started «brisk-otter» in home");
}

/// **The address is held, not re-read at fire time.** An operator who aims
/// somewhere else while the staging act is in flight must not have their fire
/// prompt a workspace nothing staged.
#[test]
fn the_fire_goes_where_the_start_was_staged_even_if_the_aim_moved() {
    let mut model = Model {
        conversation: None,
        draft: "do it".to_owned(),
        ..seated()
    };
    model.stage("home");
    model.aim = Some(Aim {
        channel: "elsewhere".to_owned(),
        address: "other".to_owned(),
    });
    model.absorb(&own().channel, staged("home", ""));
    assert_eq!(model.outbox[1]["prepared"]["workspace"], json!("home"));
}

/// **A blank goal composes nothing and costs nothing** — the deposit's own rule,
/// and a driver launched for a conversation nobody said anything to is worse
/// than a press that did not take.
#[test]
fn a_blank_goal_stages_nothing_and_keeps_what_was_typed() {
    let mut model = Model {
        draft: "   ".to_owned(),
        ..Model::default()
    };
    model.stage("home");
    assert!(model.outbox.is_empty());
    assert_eq!(model.start, None);
    assert_eq!(model.draft, "   ", "the draft is still the operator's");
}

/// **A staged body this window did not stage is not dropped.** It is fired on
/// its own terms — the name it came back under, and the goal its rung composed
/// — and the bare rung's empty goal is exactly what never fires.
#[test]
fn a_body_nothing_staged_fires_on_its_own_terms_or_not_at_all() {
    let mut model = Model::default();
    model.absorb(&own().channel, staged("home", "the rung's own prefill"));
    assert_eq!(model.outbox.len(), 1);
    assert_eq!(model.outbox[0]["goal"], json!("the rung's own prefill"));
    assert_eq!(
        model.start.as_ref().map(|start| start.phase.clone()),
        Some(Phase::Firing)
    );

    let mut bare = Model::default();
    bare.absorb(&own().channel, staged("home", "  "));
    assert!(bare.outbox.is_empty(), "a blank goal never fires");
    assert_eq!(bare.start, None);
}

/// A receipt for a start this window did not fire still paints its name: it is
/// the one fact the engine added, and a seat that dropped it would have started
/// a conversation and said nothing about it.
#[test]
fn a_receipt_with_nothing_held_still_carries_the_minted_name() {
    for (model, address) in [(Model::default(), ""), (seated(), "home")] {
        let mut model = model;
        model.absorb(
            &own().channel,
            read(&json!({"ok": true, "kind": "started", "conversation": "brisk-otter"})),
        );
        assert_eq!(
            model.start,
            Some(Start {
                address: address.to_owned(),
                goal: String::new(),
                phase: Phase::Started("brisk-otter".to_owned()),
            })
        );
    }
}

/// **Both acts read the same to an operator**, deliberately: which of the two
/// envelopes is out is this seat's business, not a fact about the conversation.
#[test]
fn a_start_in_flight_reads_the_same_whichever_act_is_out() {
    let staging = Start {
        address: "home".to_owned(),
        goal: "do it".to_owned(),
        phase: Phase::Staging,
    };
    let firing = Start {
        phase: Phase::Firing,
        ..staging.clone()
    };
    assert_eq!(staging.line(), "starting in home: do it…");
    assert_eq!(firing.line(), staging.line());
    assert!(staging.outstanding() && firing.outstanding());
}
