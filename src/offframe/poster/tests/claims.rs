//! A registration on the poster's lane (DESIGN §4.50): taken once, claimed
//! off the frame, and its outcome filed by the next settle.

use std::time::Duration;

use super::super::tick;
use crate::state::Link;
use crate::test_support::Scratch;
use crate::ui::{Claim, Model};

/// **The fired URL is claimed on this lane and its outcome reaches the
/// page.** What is typed here is no registration URL, so the claim refuses
/// before a socket opens — the whole act against a door is
/// `crate::seat::register`'s own suite — and a second pass claims nothing.
#[test]
fn a_fired_claim_is_made_off_the_frame_and_its_outcome_comes_back() {
    let scratch = Scratch::new();
    let link = Link::new(Duration::from_millis(1));
    let mut model = Model::default();
    model.registering.url = "engine.example".to_owned();
    assert!(model.register());
    link.settle(&mut model);
    tick(&link, scratch.path());
    link.settle(&mut model);
    let Claim::Done(Err(refusal)) = &model.registering.phase else {
        panic!("the claim came back refused: {:?}", model.registering.phase);
    };
    assert!(refusal.contains("is not a registration URL"), "{refusal}");
    assert!(model.registering.ready(), "and may be fired again");
    tick(&link, scratch.path());
    link.settle(&mut model);
    assert!(matches!(model.registering.phase, Claim::Done(Err(_))));
}
