//! The doctor pane between frames: what opening it asks, bare and aimed, and
//! what an answer replaces.

use crate::test_support::window::{checked, own, seated};
use crate::ui::{Aim, Channel, Model, Posted};

/// **Unaimed, the ask is the bare form** — it names no workspace, so it fans
/// and every channel answers for its own box.
#[test]
fn opening_it_unaimed_asks_the_bare_form() {
    let mut model = Model {
        aim: None,
        ..seated()
    };
    model.begin_doctor();
    assert!(model.doctoring());
    assert_eq!(model.outbox, vec![Posted::read(crate::verbs::doctor(None))]);
    model.close_lookup();
    assert!(!model.doctoring());
}

/// **Aimed, the ask carries that wall's address**, so it is routed down that
/// wall's channel and adds the wall's own checks.
#[test]
fn opening_it_aimed_asks_about_that_wall() {
    let mut model = Model {
        aim: Some(Aim {
            channel: own().channel.name,
            address: "home".to_owned(),
        }),
        ..seated()
    };
    model.begin_doctor();
    assert_eq!(
        model.outbox,
        vec![Posted::read(crate::verbs::doctor(Some("home".to_owned())))]
    );
}

/// **An answer replaces its own channel's section and no other**, and asking
/// again clears them all — two asks may have two subjects.
#[test]
fn an_answer_replaces_its_own_section_and_asking_again_clears_them() {
    let mut model = seated();
    let other = Channel {
        name: "elsewhere".to_owned(),
        named_there: None,
        dials: None,
    };
    model.diagnosed(&own().channel, vec![checked("listener", true)]);
    model.diagnosed(&other, vec![checked("address", false)]);
    model.diagnosed(&own().channel, vec![checked("identity", true)]);
    assert_eq!(model.diagnoses.len(), 2);
    assert_eq!(model.diagnoses[0].rows[0].check, "identity");
    assert_eq!(model.diagnoses[1].rows[0].check, "address");
    model.begin_doctor();
    assert!(model.diagnoses.is_empty(), "a new ask is a new subject");
}

/// **The answer comes in through the one door**, as every reply does.
#[test]
fn a_doctor_frame_is_filed_under_the_channel_it_came_down() {
    let mut model = seated();
    model.absorb(
        &own().channel,
        crate::reply::Read::Answer(crate::reply::Reply::Doctor(vec![checked("listener", true)])),
    );
    assert_eq!(model.diagnoses.len(), 1);
    assert_eq!(model.diagnoses[0].channel, own().channel);
}
