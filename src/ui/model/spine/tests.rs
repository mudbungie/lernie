//! What a fork composes, and the three states in which it composes nothing.

use serde_json::json;

use crate::test_support::window::{recorded, seated};
use crate::ui::{Forking, Model};

/// **The whole gesture, once**: the wall, the parent, the notch's own commit,
/// the trimmed role and the trimmed goal, with the skill list the wire's own
/// way of saying an attempt pins none.
#[test]
fn a_fork_carries_the_notch_it_was_fired_from_and_the_two_typed_words() {
    let mut model = recorded();
    model.forking = Forking {
        role: "  worker  ".to_owned(),
        goal: " try it the other way ".to_owned(),
    };
    model.post_fork("abcdef1234567890".to_owned());
    assert_eq!(
        model.outbox,
        vec![crate::ui::Posted::act(json!({
            "op": "fork", "workspace": "home",
            "parent": "20260830T051200Z-a1b2", "from": "abcdef1234567890",
            "role": "worker", "skills": [], "goal": "try it the other way"
        }))]
    );
}

/// **The goal is spent and the role is kept**, which is the composer's own
/// split between a flag's reason and an unmaking's arming, read here.
#[test]
fn firing_takes_the_goal_and_leaves_the_role_for_the_next_notch() {
    let mut model = recorded();
    model.forking = Forking {
        role: "worker".to_owned(),
        goal: "one".to_owned(),
    };
    model.post_fork("abcdef1234567890".to_owned());
    assert_eq!(model.forking.role, "worker");
    assert!(model.forking.goal.is_empty());
    assert!(!model.forking.ready(), "a spent goal disables the control");
}

/// **A half-typed draft is not a fact about anything**, and neither word may
/// be blank: the wire refuses an empty role and an empty goal alike.
#[test]
fn a_half_typed_draft_composes_nothing() {
    for (role, goal) in [("", "g"), ("r", ""), ("  ", "g"), ("r", "   ")] {
        let mut model = recorded();
        model.forking = Forking {
            role: role.to_owned(),
            goal: goal.to_owned(),
        };
        assert!(!model.forking.ready(), "{role:?}/{goal:?}");
        model.post_fork("abcdef1234567890".to_owned());
        assert!(model.outbox.is_empty(), "{role:?}/{goal:?}");
    }
}

/// **A fork with no wall and a fork with no conversation compose nothing** —
/// the two states the pane cannot paint a control in, made unreachable rather
/// than merely unlikely.
#[test]
fn a_fork_with_no_subject_composes_nothing() {
    let whole = Forking {
        role: "worker".to_owned(),
        goal: "g".to_owned(),
    };
    let mut nowhere = Model {
        forking: whole.clone(),
        ..Model::default()
    };
    nowhere.post_fork("abcdef1".to_owned());
    assert!(nowhere.outbox.is_empty(), "nothing is aimed at");

    let mut unselected = Model {
        forking: whole,
        conversation: None,
        ..seated()
    };
    unselected.post_fork("abcdef1".to_owned());
    assert!(unselected.outbox.is_empty(), "nothing is selected");
}

/// **The draft goes with the pane's subject.** A goal typed for one
/// conversation is a sentence about that one, and selecting another must not
/// leave it standing over a control that would fire it elsewhere.
#[test]
fn moving_the_selection_takes_the_draft_and_both_spine_answers_with_it() {
    let mut model = recorded();
    model.forking = Forking {
        role: "worker".to_owned(),
        goal: "one".to_owned(),
    };
    model.select("some-other-conversation");
    assert_eq!(model.forking, Forking::default());
    assert!(model.rail.is_none());
    assert!(model.governing.is_none());
}

/// **Both answers come in through the one door and are filed**, whether or not
/// the pane is open: the reads stand only while it is, so a frame that arrives
/// after it closed is the last one in flight rather than a thing to drop.
#[test]
fn the_spine_pair_is_filed_by_the_one_door_a_reply_comes_in_through() {
    let answered = recorded();
    let mut model = seated();
    model.absorb(
        &crate::test_support::window::own().channel,
        crate::reply::Read::Answer(crate::reply::Reply::Rail(
            answered.rail.clone().expect("the fixture answers a spine"),
        )),
    );
    model.absorb(
        &crate::test_support::window::own().channel,
        crate::reply::Read::Answer(crate::reply::Reply::Governing(
            answered.governing.clone().expect("and a governing commit"),
        )),
    );
    assert_eq!(model.rail, answered.rail);
    assert_eq!(model.governing, answered.governing);
}
