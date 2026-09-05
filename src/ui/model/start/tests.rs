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
    assert_eq!(
        model.outbox,
        vec![crate::ui::Posted::act(crate::verbs::prepare(
            "home".to_owned()
        ))]
    );
    assert_eq!(model.draft, "", "what was staged is no longer a draft");
    assert_eq!(
        model.start.as_ref().map(|start| start.phase.clone()),
        Some(Phase::Staging)
    );
    model.absorb(&own().channel, staged("home", ""));
    assert_eq!(model.outbox.len(), 2, "the fire is composed, not posted");
    assert_eq!(model.outbox[1].envelope["op"], json!("prompt"));
    assert_eq!(model.outbox[1].envelope["goal"], json!("do the thing"));
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
    assert_eq!(
        model.outbox[1].envelope["prepared"]["workspace"],
        json!("home")
    );
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
    assert_eq!(
        model.outbox[0].envelope["goal"],
        json!("the rung's own prefill")
    );
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

/// The engine's sentence, refusing a fire (yog's `docs/DESIGN.md` §8.1).
fn refused() -> crate::reply::Read {
    read(&json!({"ok": false, "error": "sign in first: no provider holds a credential"}))
}

/// A start staged and, for the second act, fired — the two states a refusal
/// can find it in.
fn out(op: &str) -> Model {
    let mut model = Model {
        conversation: None,
        draft: "do the thing".to_owned(),
        ..seated()
    };
    model.stage("home");
    if op == crate::verbs::PROMPT {
        model.absorb(&own().channel, staged("home", ""));
    }
    model
}

/// **A refused fire spends nothing** (bl-b180): the engine's refusal of either
/// act retires the start into its own sentence and hands the goal back to the
/// box, and the bar says nothing — the sentence stands where the start's did.
#[test]
fn a_refusal_of_either_act_retires_the_start_and_gives_the_goal_back() {
    for op in [crate::verbs::PREPARE, crate::verbs::PROMPT] {
        let mut model = out(op);
        assert_eq!(model.draft, "", "{op}: the goal is held, not drafted");
        model.receipt(&own().channel, op, refused());
        let start = model.start.as_ref().expect("the start stands, refused");
        assert_eq!(
            start.phase,
            Phase::Refused("sign in first: no provider holds a credential".to_owned()),
            "{op}"
        );
        assert!(!start.outstanding(), "{op}: the box is back");
        assert_eq!(
            start.line(),
            "not started in home: sign in first: no provider holds a credential"
        );
        assert_eq!(
            model.draft, "do the thing",
            "{op}: the goal is the operator's again"
        );
        assert_eq!(
            model.notice, None,
            "{op}: the sentence is the composer's, not the bar's"
        );
    }
}

/// **Only the start's own two acts can refuse it.** Another op's refusal is
/// the bar's sentence and leaves the start exactly where it was — and so does
/// a refusal that arrives once the start has already landed.
#[test]
fn another_op_s_refusal_or_a_late_one_leaves_the_start_alone() {
    let mut model = out(crate::verbs::PREPARE);
    model.receipt(&own().channel, "nudge", refused());
    assert_eq!(
        model.start.as_ref().map(|start| start.phase.clone()),
        Some(Phase::Staging)
    );
    assert_eq!(model.draft, "");
    assert!(
        model.notice.is_some(),
        "somebody else's refusal is the bar's"
    );

    let mut landed = out(crate::verbs::PROMPT);
    landed.absorb(
        &own().channel,
        read(&json!({"ok": true, "kind": "started", "conversation": "brisk-otter"})),
    );
    landed.receipt(&own().channel, crate::verbs::PROMPT, refused());
    assert_eq!(
        landed.start.as_ref().map(|start| start.phase.clone()),
        Some(Phase::Started("brisk-otter".to_owned()))
    );
    assert!(landed.notice.is_some());
}

/// **A start nothing will ever answer is a composer with no box.** A receipt
/// this seat cannot read, and no receipt at all, each take the start back:
/// the bar keeps its own sentence about that, and the goal is the operator's.
#[test]
fn an_unreadable_receipt_or_none_at_all_takes_the_start_back() {
    let mut model = out(crate::verbs::PROMPT);
    model.receipt(
        &own().channel,
        crate::verbs::PROMPT,
        read(&json!({"ok": true, "kind": "a-kind-this-build-cannot-paint"})),
    );
    assert_eq!(model.start, None);
    assert_eq!(model.draft, "do the thing");
    assert!(matches!(
        model.notice,
        Some(crate::ui::Notice::Unreadable(_))
    ));

    let mut model = out(crate::verbs::PREPARE);
    model.acted(
        crate::verbs::PREPARE,
        &crate::channel::Reach::Unsent("no channel".to_owned()),
    );
    assert_eq!(model.start, None);
    assert_eq!(model.draft, "do the thing");
    assert!(matches!(
        model.notice,
        Some(crate::ui::Notice::Unsent { .. })
    ));

    // Another op's lost reply takes nothing back.
    let mut model = out(crate::verbs::PREPARE);
    model.acted(
        "nudge",
        &crate::channel::Reach::Unsent("no channel".to_owned()),
    );
    assert!(model.start.is_some());
    assert_eq!(model.draft, "");
}

/// **The refund never overwrites a draft typed elsewhere.** The composer is one
/// box with two subjects, so a deposit drafted on another wall while the start
/// was out is the operator's newer text and is kept.
#[test]
fn the_refund_never_overwrites_a_draft_typed_elsewhere() {
    let mut model = out(crate::verbs::PROMPT);
    model.draft = "a deposit drafted meanwhile".to_owned();
    model.receipt(&own().channel, crate::verbs::PROMPT, refused());
    assert_eq!(model.draft, "a deposit drafted meanwhile");
    assert!(matches!(
        model.start.as_ref().map(|start| &start.phase),
        Some(Phase::Refused(_))
    ));
}
