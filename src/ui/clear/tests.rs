//! The place a trail is cut in: the control that opens it, the subject it
//! states, the way out that cuts nothing, and the act.

use super::{CLOSE, CONFIRM, HEADING, OPEN, REACH, WHAT};
use crate::paint_probe::frame::Window;
use crate::test_support::window::{click, own, painted, trailing};
use crate::ui::{Channel, Model, Trail};

/// **It is opened from the trail pane and it stands the trail down**, being
/// the same field: one is the place the other's act lives in, and the two are
/// never on one glass.
#[test]
fn the_trail_s_control_opens_the_place_and_stands_the_trail_down() {
    let mut model = trailing();
    let window = Window::new();
    click(&window, OPEN, |ctx| crate::ui::render(ctx, &mut model));
    assert!(model.clearing(), "the control opened nothing");
    assert!(!model.trailing(), "two panes on one glass");
    let glass = painted(&mut model);
    assert!(glass.contains(HEADING), "{glass}");
    assert!(glass.contains(WHAT), "{glass}");
    assert!(glass.contains(REACH), "{glass}");
}

/// **The subject is stated before the act is offered** — every channel this
/// box holds, named, including one that has answered nothing: the gesture goes
/// down every channel the standing set holds, and naming only the ones that
/// answered would understate what the cut reaches.
#[test]
fn it_names_every_engine_the_cut_would_reach() {
    let mut model = trailing();
    let quiet = Channel {
        name: "elsewhere".to_owned(),
        ..own().channel
    };
    model.roster.push(crate::ui::Chunk::of(quiet.clone()));
    model.trails.push(Trail {
        channel: quiet,
        rows: Vec::new(),
    });
    model.begin_clearing();
    let glass = painted(&mut model);
    assert!(glass.contains("elsewhere"), "{glass}");
}

/// **The way out comes first and cuts nothing**, and it goes back to the trail
/// rather than to no pane at all: that is where the gesture came from, and its
/// read is standing.
#[test]
fn the_way_out_cuts_nothing_and_returns_to_the_trail() {
    let mut model = trailing();
    model.begin_clearing();
    let window = Window::new();
    click(&window, CLOSE, |ctx| crate::ui::render(ctx, &mut model));
    assert!(model.trailing(), "the way out left the operator nowhere");
    assert!(model.outbox.is_empty(), "the way out cut something");
}

/// **The act fans, and the pane stands down onto the trail as it fires** — the
/// place to be standing when the answer lands is the trail, because the trail
/// is what answers.
#[test]
fn the_act_composes_the_bare_gesture_and_goes_back_to_the_trail() {
    let mut model = trailing();
    model.begin_clearing();
    let window = Window::new();
    click(&window, CONFIRM, |ctx| crate::ui::render(ctx, &mut model));
    let posted = model.outbox.first().expect("a gesture");
    assert_eq!(posted.envelope, serde_json::json!({ "op": "clear-trail" }));
    assert!(posted.act, "a truncation is an act");
    assert!(model.trailing());
}

/// It paints nothing while it is not the pane standing.
#[test]
fn it_paints_nothing_while_it_is_down() {
    let mut model = Model::default();
    let glass = painted(&mut model);
    assert!(!glass.contains(HEADING), "{glass}");
}
