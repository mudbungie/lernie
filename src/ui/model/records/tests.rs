//! The records pane's acts: the gate on opening, the retirement with its
//! subject, and the cover question three panes share.

use crate::test_support::window::{recorded, seated};
use crate::ui::{Enrolling, Model, Tuning};

/// **The selection is the gate**: no conversation, no pane — the same reading
/// tuning gives the aim.
#[test]
fn the_pane_opens_only_on_a_selected_conversation() {
    let mut adrift = Model {
        conversation: None,
        ..seated()
    };
    adrift.begin_records();
    assert!(!adrift.records, "no subject, no pane");
    let mut model = seated();
    model.begin_records();
    assert!(model.records);
    model.close_records();
    assert!(!model.records);
}

/// **The records go with the conversation they answer.** Selecting another
/// closes the pane and retires both answers; the close alone keeps them, for
/// the same reason the roles stay — the standing read replaces them anyway.
#[test]
fn selecting_another_conversation_retires_the_pane_and_its_answers() {
    let mut model = recorded();
    model.select("20260830T051200Z-zzzz");
    assert!(!model.records);
    assert_eq!(model.steps, None);
    assert_eq!(model.files, None);
    let mut model = recorded();
    model.close_records();
    assert!(model.steps.is_some(), "a close keeps the answers");
}

/// Aiming at another wall retires everything under the old one, the records
/// included — they hang off a conversation the new wall does not hold.
#[test]
fn aiming_elsewhere_retires_the_records_too() {
    let mut model = recorded();
    model.aim_at("(this box's own engine)", "elsewhere");
    assert!(!model.records);
    assert_eq!(model.steps, None);
    assert_eq!(model.files, None);
}

/// **Escape closes it**, on the ladder's own rung: after the panes that hold
/// more, before the notice.
#[test]
fn escape_closes_the_records_pane_before_reaching_the_notice() {
    let mut model = recorded();
    model.notice = Some(crate::ui::Notice::Refused("no".to_owned()));
    model.escape();
    assert!(!model.records, "the pane went down");
    assert!(model.notice.is_some(), "the notice did not");
    model.escape();
    assert_eq!(model.notice, None, "the next escape reaches it");
}

/// **One cover at a time is a question, and this is it**: each pane answers
/// it, and the empty window answers no.
#[test]
fn covered_answers_for_all_three_panes_and_for_none() {
    assert!(!seated().covered());
    assert!(recorded().covered());
    let tuning = Model {
        tuning: Some(Tuning::Rows),
        ..seated()
    };
    assert!(tuning.covered());
    let mut enrolling = seated();
    let aim = enrolling.aim.clone().expect("the seated fixture is aimed");
    enrolling.enroll = Some(Enrolling::at(aim));
    assert!(enrolling.covered());
}
