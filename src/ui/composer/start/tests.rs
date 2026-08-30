//! The start composer: the case it takes over, what Enter composes, and the
//! start in flight that refuses a second one.

use super::{GOAL, START, render};
use crate::paint_probe::frame::{Window, locate_in, press};
use crate::test_support::window::{click, pane, seated};
use crate::ui::model::{Phase, Start};
use crate::ui::{Aim, Model};
use serde_json::json;

/// A model aimed at a wall with nothing selected on it — the case the start
/// composer takes over.
fn unselected() -> Model {
    Model {
        conversation: None,
        ..seated()
    }
}

/// The aim the fixture is on.
fn aimed() -> Aim {
    unselected().aim.expect("the fixture is aimed")
}

/// **A wall with no conversation is where one is begun.** The pane that used to
/// refuse now asks the question a start answers.
#[test]
fn a_wall_with_nothing_selected_offers_a_start() {
    let mut model = unselected();
    let painted = pane(|ui| render(ui, &mut model, &aimed()));
    for expected in [GOAL, START] {
        assert!(
            painted.lines().any(|line| line == expected),
            "{expected:?}:\n{painted}"
        );
    }
}

/// **The button composes the first act and holds the text.** It composes and
/// does not send: the gesture lands in the outbox like every other.
#[test]
fn starting_composes_the_staging_act_and_posts_nothing() {
    let mut model = unselected();
    model.draft = "do the thing".to_owned();
    let window = Window::new();
    click(&window, START, |ctx| {
        egui::CentralPanel::default().show(ctx, |ui| render(ui, &mut model, &aimed()));
    });
    assert_eq!(
        model.outbox,
        vec![json!({"op": "prepare", "workspace": "home", "payload": {"rung": "bare"}})]
    );
    assert_eq!(
        model.start.as_ref().map(|start| start.goal.clone()),
        Some("do the thing".to_owned())
    );
}

/// **Enter begins it**, exactly as Enter sends a deposit — one box, one Enter.
#[test]
fn enter_begins_what_was_typed() {
    let mut model = unselected();
    let window = Window::new();
    let mut body = |ctx: &egui::Context| {
        egui::CentralPanel::default().show(ctx, |ui| render(ui, &mut model, &aimed()));
    };
    // Aimed at the hint inside the box, not at the button: what Enter needs is
    // the box holding focus.
    let at = locate_in(&window, GOAL, &mut body).expect("the box");
    crate::paint_probe::frame::click(&window, at, &mut body);
    window.frame(
        vec![egui::Event::Text("do the thing".to_owned())],
        &mut body,
    );
    window.frame(vec![press(egui::Key::Enter)], &mut body);
    window.frame(Vec::new(), &mut body);
    assert_eq!(model.outbox.len(), 1, "{:?}", model.outbox);
    assert_eq!(model.outbox[0]["op"], json!("prepare"));
}

/// **A start in flight refuses a second one, by there being no control.** Two
/// starts chained through one composer would spend one goal on two
/// conversations and leave the first unfinished.
#[test]
fn a_start_in_flight_paints_its_sentence_and_offers_no_second_start() {
    for phase in [Phase::Staging, Phase::Firing] {
        let mut model = Model {
            start: Some(Start {
                address: "home".to_owned(),
                goal: "do the thing".to_owned(),
                phase,
            }),
            ..unselected()
        };
        let painted = pane(|ui| render(ui, &mut model, &aimed()));
        assert!(
            painted.contains("starting in home: do the thing…"),
            "{painted}"
        );
        assert!(
            !painted.lines().any(|line| line == START),
            "no control to press:\n{painted}"
        );
    }
}

/// **The receipt stays readable while the next start is composed.** The minted
/// name is what the flow was for, and the box comes back under it because the
/// start is over.
#[test]
fn a_finished_start_paints_its_name_over_the_box_that_begins_the_next() {
    let mut model = Model {
        start: Some(Start {
            address: "home".to_owned(),
            goal: "do the thing".to_owned(),
            phase: Phase::Started("brisk-otter".to_owned()),
        }),
        ..unselected()
    };
    let painted = pane(|ui| render(ui, &mut model, &aimed()));
    assert!(
        painted.contains("started «brisk-otter» in home"),
        "{painted}"
    );
    assert!(painted.lines().any(|line| line == START), "{painted}");
}
