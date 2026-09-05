//! What the drill-in control does to the model, and the one thing the model
//! deliberately does not remember.

use crate::test_support::window::{drilled, recorded, seated};
use crate::ui::Model;
use serde_json::json;

/// **The read is posted and it names the row it was fired from.** A read
/// rather than an act: asking twice is asking once, so a lost reply is
/// recovered by clicking again.
#[test]
fn asking_for_a_step_posts_a_read_carrying_its_seq() {
    let mut model = recorded();
    model.ask_step("002");
    assert_eq!(
        model.outbox,
        vec![crate::ui::Posted::read(json!({
            "op": "step", "workspace": "home",
            "agent": "20260830T051200Z-a1b2", "seq": "002"
        }))]
    );
}

/// **With no wall or no conversation there is nothing to address**, which are
/// the two states the pane cannot paint the control in.
#[test]
fn asking_with_no_subject_composes_nothing() {
    let mut nowhere = Model::default();
    nowhere.ask_step("001");
    assert!(nowhere.outbox.is_empty(), "nothing is aimed at");

    let mut unselected = Model {
        conversation: None,
        ..seated()
    };
    unselected.ask_step("001");
    assert!(unselected.outbox.is_empty(), "nothing is selected");
}

/// **The answer says which step it is about**, so the model holds no second
/// name for it and a reply cannot paint under the wrong row.
#[test]
fn the_drill_in_belongs_to_the_row_the_answer_names() {
    let mut model = recorded();
    model.records.drilled = Some(drilled("002"));
    assert_eq!(model.drilled_into("002"), Some(drilled("002")));
    assert_eq!(model.drilled_into("001"), None, "another row's records");
    model.records.drilled = None;
    assert_eq!(model.drilled_into("002"), None);
}

/// The deeper three go with the pane's subject, exactly as the four above them
/// do — a header for one conversation left standing over another would name
/// the wrong thing.
#[test]
fn moving_the_selection_takes_all_three_deeper_answers() {
    let mut model = recorded();
    model.records.drilled = Some(drilled("001"));
    model.select("some-other-conversation");
    assert!(model.records.agent.is_none());
    assert!(model.records.mail.is_none());
    assert!(model.records.drilled.is_none());
}

/// All three come in through the one door a reply comes in through.
#[test]
fn the_deeper_three_are_filed_by_the_one_door() {
    let answered = recorded();
    let mut model = seated();
    let channel = crate::test_support::window::own().channel;
    model.absorb(
        &channel,
        crate::reply::Read::Answer(crate::reply::Reply::Agent(Box::new(
            answered
                .records
                .agent
                .clone()
                .expect("the fixture answers a row"),
        ))),
    );
    model.absorb(
        &channel,
        crate::reply::Read::Answer(crate::reply::Reply::Inbox(
            answered.records.mail.clone().expect("and its mail"),
        )),
    );
    model.absorb(
        &channel,
        crate::reply::Read::Answer(crate::reply::Reply::Step(Box::new(drilled("001")))),
    );
    assert_eq!(model.records.agent, answered.records.agent);
    assert_eq!(model.records.mail, answered.records.mail);
    assert_eq!(model.records.drilled, Some(drilled("001")));
}
