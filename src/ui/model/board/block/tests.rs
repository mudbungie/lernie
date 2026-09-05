//! The block's four composers: what each carries, and the absence that makes
//! the control that spends it dark.

use serde_json::json;

use super::Authoring;
use crate::test_support::window::boarded;
use crate::ui::Model;

/// The block the fixture's wall opens on its one held ball.
fn amending() -> Model {
    let mut model = boarded();
    model.begin_amending(&crate::test_support::window::panes::board::bound("bl-1"));
    model
}

/// The block open on a ball that does not exist yet.
fn filing() -> Model {
    let mut model = boarded();
    model.begin_filing();
    model
}

/// What the block holds, for a test that types into it.
fn block(model: &mut Model) -> &mut Authoring {
    model.authoring.as_mut().expect("the block is open")
}

/// **A filing needs a project and a title**, and the body is optional — the
/// absence that makes `create` a door rather than a row.
#[test]
fn a_filing_goes_only_with_a_project_and_a_title() {
    let mut model = filing();
    assert!(block(&mut model).filing());
    assert_eq!(model.authoring.as_ref().and_then(Authoring::filed), None);

    block(&mut model).project = "  lernie ".to_owned();
    assert_eq!(model.authoring.as_ref().and_then(Authoring::filed), None);

    block(&mut model).title = " a title ".to_owned();
    assert_eq!(
        model.authoring.as_ref().and_then(Authoring::filed),
        Some(json!({
            "op": "create", "project": "lernie", "name": "home", "title": "a title"
        }))
    );

    block(&mut model).body = "the body".to_owned();
    assert_eq!(
        model.authoring.as_ref().and_then(Authoring::filed),
        Some(json!({
            "op": "create", "project": "lernie", "name": "home",
            "title": "a title", "body": "the body"
        }))
    );
}

/// **A block about an existing ball files nothing**, which is the fold's one
/// rule: one block, two subjects, and each offers only its own acts.
#[test]
fn a_block_about_a_held_ball_composes_no_filing() {
    let model = amending();
    assert_eq!(model.authoring.as_ref().and_then(Authoring::filed), None);
    assert!(!model.authoring.as_ref().is_some_and(Authoring::filing));
}

/// **An amendment carries only what was typed**, and an amendment of nothing
/// is not one: upstream refuses it by name, so this end never sends it.
#[test]
fn an_amendment_of_nothing_composes_nothing() {
    let mut model = amending();
    assert_eq!(model.authoring.as_ref().and_then(Authoring::amended), None);

    block(&mut model).note = "  what happened  ".to_owned();
    assert_eq!(
        model.authoring.as_ref().and_then(Authoring::amended),
        Some(json!({
            "op": "update", "project": "lernie", "id": "bl-1", "name": "home",
            "note": "what happened"
        }))
    );

    block(&mut model).title = "renamed".to_owned();
    block(&mut model).body = "rewritten".to_owned();
    assert_eq!(
        model.authoring.as_ref().and_then(Authoring::amended),
        Some(json!({
            "op": "update", "project": "lernie", "id": "bl-1", "name": "home",
            "title": "renamed", "body": "rewritten", "note": "what happened"
        }))
    );
}

/// **A release is offered whenever there is a ball**, because it is undone by
/// the claim on the board one section up. A filing has none to release.
#[test]
fn a_release_needs_nothing_but_a_ball() {
    assert_eq!(
        amending().authoring.as_ref().and_then(Authoring::released),
        Some(json!({
            "op": "release", "project": "lernie", "id": "bl-1", "name": "home"
        }))
    );
    assert_eq!(
        filing().authoring.as_ref().and_then(Authoring::released),
        None
    );
}

/// **The arming is the subject's own name** (§4.20): a delivery composes only
/// once the box holds this ball's id, and any other id arms nothing.
#[test]
fn a_delivery_waits_for_the_balls_own_id() {
    let mut model = amending();
    assert_eq!(
        model.authoring.as_ref().and_then(Authoring::delivered),
        None
    );

    block(&mut model).arm = "bl-2".to_owned();
    assert_eq!(
        model.authoring.as_ref().and_then(Authoring::delivered),
        None
    );

    block(&mut model).arm = " bl-1 ".to_owned();
    assert_eq!(
        model.authoring.as_ref().and_then(Authoring::delivered),
        Some(json!({
            "op": "close", "project": "lernie", "id": "bl-1", "name": "home"
        }))
    );
    assert_eq!(
        filing().authoring.as_ref().and_then(Authoring::delivered),
        None
    );
}
