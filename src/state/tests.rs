//! The link: what a settle files, what it hands over, and what it publishes.

use std::time::Duration;

use super::{Link, Said, Standing};
use crate::test_support::window::{own, seated};
use crate::ui::{Aim, Model};
use serde_json::json;

fn link() -> Link {
    Link::new(Duration::from_millis(1))
}

/// **The standing set is a query over the model**, so there is nothing to
/// invalidate: a click changes the model, and what is asked next follows.
#[test]
fn what_to_ask_is_derived_from_the_model_and_never_stored() {
    let model = seated();
    let standing = Standing::of(&model);
    assert_eq!(standing.channels, vec![own().channel]);
    assert_eq!(standing.aim, model.aim);
    assert_eq!(standing.conversation, model.conversation);
    assert_eq!(Standing::of(&Model::default()), Standing::default());
}

/// A focus on a channel this seat no longer holds is **not a question**. The
/// two can disagree for a frame — a roster answer may drop the channel the aim
/// was on — and asking down a channel that is gone is asking nothing.
#[test]
fn an_aim_on_a_channel_that_is_gone_is_not_asked() {
    let mut model = seated();
    assert!(Standing::of(&model).aimed().is_some());
    model.roster.clear();
    assert!(Standing::of(&model).aimed().is_none());
    model = Model::default();
    assert!(Standing::of(&model).aimed().is_none());
}

/// **The frame's whole side, in one call**: what landed is filed, what was
/// composed is handed over, and what to ask next is published.
#[test]
fn a_settle_files_hands_over_and_publishes() {
    let link = link();
    let channel = own().channel;
    link.heard(
        &channel,
        Said::Frame(json!({"ok": true, "kind": "workspaces", "rows": []})),
    );
    let mut model = Model {
        outbox: vec![crate::ui::Posted::act(json!({"op": "nudge"}))],
        aim: Some(Aim {
            channel: channel.name.clone(),
            address: "home".to_owned(),
        }),
        ..Model::default()
    };
    link.settle(&mut model);
    assert_eq!(model.roster.len(), 1, "the answer was filed");
    assert!(model.outbox.is_empty(), "the frame handed it over");
    assert_eq!(
        link.compose(),
        vec![crate::ui::Posted::act(json!({"op": "nudge"}))]
    );
    assert_eq!(link.standing().aim, model.aim);
    assert!(link.compose().is_empty(), "a drain takes it once");
}

/// **A leg that never reached an engine reads as neither of the other two.**
/// The three remedies are three different acts: type something else, upgrade
/// the seat, or look at this box's files.
#[test]
fn an_unreachable_leg_is_its_own_sentence() {
    let seated = link();
    seated.heard(
        &own().channel,
        Said::Unreachable("connect 127.0.0.1:1: refused".to_owned()),
    );
    // **It lands on that channel's own section** (bl-e620), which is where a
    // fact about a relationship belongs — the bar is for what an engine said.
    let mut model = Model {
        roster: vec![own()],
        ..Model::default()
    };
    seated.settle(&mut model);
    assert_eq!(
        model.roster[0].held,
        crate::ui::Held::Unheld("connect 127.0.0.1:1: refused".to_owned())
    );
    assert_eq!(model.notice, None, "the shell-wide bar is not its home");

    // With no section for it there is nowhere else, so the bar takes it — and
    // names the channel, because a fact with no home still has a subject.
    let orphan = link();
    orphan.heard(
        &own().channel,
        Said::Unreachable("connect 127.0.0.1:1: refused".to_owned()),
    );
    let mut homeless = Model::default();
    orphan.settle(&mut homeless);
    let Some(notice) = &homeless.notice else {
        panic!("a notice");
    };
    assert!(
        notice.line().contains("could not reach"),
        "{}",
        notice.line()
    );
    assert!(
        notice.line().contains(&own().channel.name),
        "it names its subject: {}",
        notice.line()
    );
}

/// A stop is a fact the workers read between passes, and the beat is theirs to
/// wait.
#[test]
fn a_link_carries_the_cadence_and_the_stop() {
    let link = link();
    assert_eq!(link.beat(), Duration::from_millis(1));
    assert!(!link.stopped());
    link.stop();
    assert!(link.stopped());
    assert!(link.clone().stopped(), "every handle is the same link");
}

/// **An act's receipt is filed knowing which act it answered** (bl-b180): the
/// poster stamps the op, and the one thing held across two acts — a start —
/// is retired by its own refusal and by no other.
#[test]
fn a_receipt_is_filed_with_the_act_it_answers() {
    let link = link();
    let channel = own().channel;
    let mut model = Model {
        conversation: None,
        draft: "do the thing".to_owned(),
        ..seated()
    };
    model.stage("home");
    link.heard(
        &channel,
        Said::Receipt {
            op: "prepare".to_owned(),
            frame: json!({"ok": false, "error": "sign in first"}),
        },
    );
    link.settle(&mut model);
    assert_eq!(
        model.start.as_ref().map(|start| start.phase.clone()),
        Some(crate::ui::model::Phase::Refused("sign in first".to_owned()))
    );
    assert_eq!(model.draft, "do the thing", "the goal came back");
    assert_eq!(model.notice, None);
}
