//! What the tuning pane's controls do to the model, whichever control did it.

use super::{Edit, Tuning};
use crate::test_support::window::{role, seated, tuned};
use crate::ui::Model;
use serde_json::json;

/// **The aim is the gate**, because every gesture the pane composes carries a
/// workspace and a workspace is what an aim is.
#[test]
fn the_pane_opens_on_an_aimed_wall_and_on_nothing_else() {
    let mut nowhere = Model::default();
    nowhere.begin_tuning();
    assert_eq!(nowhere.tuning, None, "nothing to configure");

    let mut model = seated();
    model.begin_tuning();
    assert_eq!(model.tuning, Some(Tuning::Rows));
    model.close_tuning();
    assert_eq!(model.tuning, None);
}

/// **Aiming elsewhere retires the pane and the rows with it.** The pane holds
/// no aim of its own, so leaving it open would leave one wall's rows over
/// another wall's controls.
#[test]
fn aiming_at_another_wall_closes_the_pane_and_forgets_what_it_showed() {
    let mut model = tuned();
    model.aim_at("(this box's own engine)", "elsewhere");
    assert_eq!(model.tuning, None);
    assert_eq!(model.roles, None, "the old wall's answer is not this one's");
}

/// **The editor opens seeded from what is in force**, which is the whole point
/// of the pane's first question being the read.
#[test]
fn the_assignment_editor_opens_holding_the_row_it_was_opened_on() {
    let row = role("worker");
    let mut model = tuned();
    model.edit_assignment(&row);
    assert_eq!(
        model.editing("worker"),
        Some(Edit {
            role: "worker".to_owned(),
            provider: "housevendor".to_owned(),
            model: "house-model-1".to_owned(),
        })
    );
    assert_eq!(model.editing("compactor"), None, "one row at a time");
    model.cancel_assignment();
    assert_eq!(model.tuning, Some(Tuning::Rows), "the pane is still open");
    assert_eq!(model.editing("worker"), None);
}

/// **Neither editing act reaches a closed pane.** There is no state that means
/// *editing while closed*, and these are the two doors that could make one.
#[test]
fn nothing_edits_an_assignment_while_the_pane_is_shut() {
    let mut model = seated();
    model.edit_assignment(&role("worker"));
    assert_eq!(model.tuning, None);
    model.cancel_assignment();
    assert_eq!(model.tuning, None);
    assert!(model.draft_assignment().is_none());
}

/// The two seat-shaped writes, composed as the wire spells them.
#[test]
fn effort_and_priority_compose_the_gestures_the_wire_takes() {
    let mut model = tuned();
    model.post_effort("worker", Some("low".to_owned()));
    model.post_effort("worker", None);
    model.post_priority("compactor", true);
    assert_eq!(
        model.outbox,
        vec![
            json!({"op": "effort", "workspace": "home", "role": "worker", "level": "low"}),
            json!({"op": "effort", "workspace": "home", "role": "worker", "level": null}),
            json!({"op": "priority", "workspace": "home", "role": "compactor", "on": true}),
        ]
    );
}

/// **A draft is not a fact until both halves of it are named**, and the trim is
/// where the words become the assignment.
#[test]
fn a_half_named_assignment_composes_nothing_and_a_whole_one_composes_once() {
    let mut model = tuned();
    model.edit_assignment(&role("worker"));
    for half in ["", "   "] {
        let draft = model.draft_assignment().expect("the editor is open");
        draft.model = half.to_owned();
        assert!(!draft.ready());
        model.post_assignment();
        assert!(model.outbox.is_empty(), "{half:?} is half an assignment");
    }
    let draft = model.draft_assignment().expect("the editor is open");
    draft.provider = "  othervendor  ".to_owned();
    draft.model = " house-model-2 ".to_owned();
    model.post_assignment();
    assert_eq!(
        model.outbox,
        vec![json!({"op": "model", "workspace": "home", "role": "worker",
                    "provider": "othervendor", "model": "house-model-2"})]
    );
    assert_eq!(
        model.tuning,
        Some(Tuning::Rows),
        "spending the draft puts the editor away"
    );
}

/// **A pane with no editor open spends nothing**, which is the arm the control
/// cannot reach and the keyboard could.
#[test]
fn posting_an_assignment_with_no_draft_open_composes_nothing() {
    let mut model = tuned();
    model.post_assignment();
    assert!(model.outbox.is_empty());
}

/// **The aim is read at the moment of composing**, so a pane left open over no
/// wall composes nothing rather than a gesture addressed at nobody. It is
/// unreachable through the controls and is the reading that makes it so — for
/// each of the three writes, because the door they share is generic and each
/// spends its own copy of it.
#[test]
fn no_tuning_gesture_is_composed_with_no_wall_under_the_pane() {
    let mut model = Model {
        tuning: Some(Tuning::Rows),
        ..Model::default()
    };
    model.post_effort("worker", None);
    model.post_priority("worker", true);
    model.tuning = Some(Tuning::Editing(Edit::of(&role("worker"))));
    model.post_assignment();
    assert!(model.outbox.is_empty());
}

/// **A roles answer is filed whether or not the pane is open.** The read is
/// standing only while it is, so a frame arriving after it closed is the last
/// one in flight — and dropping it would leave the next open blank for a beat
/// over a wall this seat had already been told about.
#[test]
fn a_roles_answer_is_filed_through_the_one_door_open_pane_or_not() {
    let channel = crate::test_support::window::own().channel;
    let rows = vec![role("worker")];
    for pane in [Some(Tuning::Rows), None] {
        let mut model = Model {
            tuning: pane,
            ..seated()
        };
        model.absorb(
            &channel,
            crate::reply::Read::Answer(crate::reply::Reply::Roles(rows.clone())),
        );
        assert_eq!(model.roles, Some(rows.clone()));
    }
}

/// Escape is a ladder from the innermost thing outwards: the draft, then the
/// pane, then the notice.
#[test]
fn escape_puts_the_draft_down_before_it_puts_the_pane_down() {
    let mut model = tuned();
    model.edit_assignment(&role("worker"));
    model.escape();
    assert_eq!(model.tuning, Some(Tuning::Rows), "the draft went first");
    model.escape();
    assert_eq!(model.tuning, None, "then the pane");
    model.notice = Some(crate::ui::Notice::Refused("no".to_owned()));
    model.escape();
    assert_eq!(model.notice, None, "then the notice");
}
