//! The start composer: the case it takes over, what Enter composes, and the
//! start in flight that refuses a second one.

/// What the start mode LOOKS like (DESIGN §4.39): the deposit's own field,
/// its act tags, and the row under it.
mod shape;

use super::{goal, render};
use crate::paint_probe::frame::{Window, locate_in, press};
use crate::test_support::window::{pane, seated};
use crate::ui::model::{Phase, Start};
use crate::ui::theme::glyph;
use crate::ui::{Aim, Model};
use serde_json::json;

/// A model aimed at a workspace with nothing selected on it — the case the start
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

/// **A workspace with no conversation is where one is begun.** The pane that used to
/// refuse now asks the question a start answers.
#[test]
fn a_workspace_with_nothing_selected_offers_a_start() {
    let mut model = unselected();
    let painted = pane(|ui| render(ui, &mut model, &aimed()));
    for expected in [goal("home", false).as_str(), glyph::PROMPT] {
        assert!(
            painted.lines().any(|line| line.starts_with(expected)),
            "{expected:?}:\n{painted}"
        );
    }
}

/// **Enter composes the first act and holds the text.** It composes and does
/// not send: the gesture lands in the outbox like every other.
#[test]
fn starting_composes_the_staging_act_and_posts_nothing() {
    let mut model = unselected();
    model.draft = "do the thing".to_owned();
    let window = Window::new();
    window.frame(Vec::new(), |ctx| {
        egui::CentralPanel::default().show(ctx, |ui| render(ui, &mut model, &aimed()));
    });
    window.focus(egui::Id::new(crate::ui::keys::BOX_ID));
    window.frame(vec![press(egui::Key::Enter)], |ctx| {
        egui::CentralPanel::default().show(ctx, |ui| render(ui, &mut model, &aimed()));
    });
    assert_eq!(
        model.outbox,
        vec![crate::ui::Posted::act(
            json!({"op": "prepare", "workspace": "home", "payload": {"rung": "bare"}})
        )]
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
    let at = crate::paint_probe::frame::led_in(&window, &goal("home", false), &mut body)
        .expect("the box");
    crate::paint_probe::frame::click(&window, at, &mut body);
    window.frame(
        vec![egui::Event::Text("do the thing".to_owned())],
        &mut body,
    );
    window.frame(vec![press(egui::Key::Enter)], &mut body);
    window.frame(Vec::new(), &mut body);
    assert_eq!(model.outbox.len(), 1, "{:?}", model.outbox);
    assert_eq!(model.outbox[0].envelope["op"], json!("prepare"));
}

/// **A slash line is the menu's, not the start's** (`crate::ui::slash`):
/// Enter on `/plan` begins nothing.
#[test]
fn enter_on_a_slash_line_begins_nothing() {
    let mut model = unselected();
    model.draft = "/plan".to_owned();
    let window = Window::new();
    let mut body = |ctx: &egui::Context| {
        egui::CentralPanel::default().show(ctx, |ui| render(ui, &mut model, &aimed()));
    };
    let at = locate_in(&window, "/plan", &mut body).expect("the box");
    crate::paint_probe::frame::click(&window, at, &mut body);
    window.frame(vec![press(egui::Key::Enter)], &mut body);
    assert!(model.outbox.is_empty(), "{:?}", model.outbox);
    assert_eq!(model.start, None);
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
                spread: None,
            }),
            ..unselected()
        };
        let painted = pane(|ui| render(ui, &mut model, &aimed()));
        assert!(
            painted.contains("starting in home: do the thing…"),
            "{painted}"
        );
        assert!(
            !painted.lines().any(|line| line == glyph::PROMPT),
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
            spread: None,
        }),
        ..unselected()
    };
    let painted = pane(|ui| render(ui, &mut model, &aimed()));
    assert!(
        painted.contains("started «brisk-otter» in home"),
        "{painted}"
    );
    assert!(
        painted.lines().any(|line| line == glyph::PROMPT),
        "{painted}"
    );
}

/// **A refused start paints its sentence over the box that carries the goal
/// back** (bl-b180): the server's *sign in first* stands where the start's own
/// sentence stood, and under it the box is live again with what was typed.
#[test]
fn a_refused_start_paints_its_sentence_over_the_box_that_carries_the_goal_back() {
    let mut model = Model {
        start: Some(Start {
            address: "home".to_owned(),
            goal: "do the thing".to_owned(),
            phase: Phase::Refused("sign in first: no provider holds a credential".to_owned()),
            spread: None,
        }),
        draft: "do the thing".to_owned(),
        ..unselected()
    };
    let painted = pane(|ui| render(ui, &mut model, &aimed()));
    assert!(
        painted.contains("not started in home: sign in first: no provider holds a credential"),
        "{painted}"
    );
    assert!(
        painted.lines().any(|line| line == glyph::PROMPT),
        "{painted}"
    );
    assert!(
        painted.contains("do the thing"),
        "the goal is in the box:\n{painted}"
    );
}

/// **A refused start carries the doors its sentence names** (bl-7a77): the
/// server's *… /config brazen* stands with `/config` after it as a link, and
/// a click on it opens what the menu's row opens; a sentence naming no row
/// carries none.
#[test]
fn a_refused_start_carries_the_doors_it_names() {
    let refused = |why: &str| Model {
        start: Some(Start {
            address: "home".to_owned(),
            goal: "do the thing".to_owned(),
            phase: Phase::Refused(why.to_owned()),
            spread: None,
        }),
        ..unselected()
    };
    let mut model =
        refused("sign in first — write its key into this workspace with /config brazen");
    let mut body = |ctx: &egui::Context| {
        egui::CentralPanel::default().show(ctx, |ui| render(ui, &mut model, &aimed()));
    };
    let window = Window::new();
    crate::test_support::window::click(&window, "/config", &mut body);
    assert!(model.covered(), "the config pane stands");
    let mut model = refused("sign in first: no provider holds a credential");
    let painted = pane(|ui| render(ui, &mut model, &aimed()));
    assert!(!painted.lines().any(|line| line == "/config"), "{painted}");
}

/// **The hint names the workspace the Enter begins on, and says when it begins a
/// plan** (bl-9031).
#[test]
fn the_hint_names_the_workspace_and_the_plan() {
    let mut model = unselected();
    let painted = pane(|ui| render(ui, &mut model, &aimed()));
    assert!(painted.contains("on home is for"), "{painted}");
    model.plan = true;
    let planned = pane(|ui| render(ui, &mut model, &aimed()));
    assert!(planned.contains(&goal("home", true)), "{planned}");
}

/// **A workspace whose worker reaches no model says so before a goal is typed**
/// (bl-7a77): the worker's provider row is bare, so the hint names the doors
/// and the pick; a credentialed row, or a table not yet read, asks the goal.
#[test]
fn a_workspace_whose_worker_holds_no_credential_names_the_doors() {
    let worker = crate::reply::roles::RoleRow {
        role: "worker".to_owned(),
        provider: "anthropic".to_owned(),
        model: "m".to_owned(),
        priority: false,
        effort: None,
    };
    let row = |fact: &str| crate::reply::providers::ProviderRow {
        name: "anthropic".to_owned(),
        fact: fact.to_owned(),
        blocked: None,
        effort: true,
        priority: true,
    };
    for (providers, unsigned) in [
        (Some(vec![row("auth api_key · no credential stored")]), true),
        (Some(vec![row("auth api_key · credential stored")]), false),
        (None, false),
    ] {
        let mut model = Model {
            roles: Some(vec![worker.clone()]),
            providers,
            ..unselected()
        };
        assert_eq!(super::unsigned(&model), unsigned);
        let painted = pane(|ui| render(ui, &mut model, &aimed()));
        assert_eq!(painted.contains(super::UNSIGNED), unsigned, "{painted}");
        assert_eq!(
            painted.contains(&goal("home", false)),
            !unsigned,
            "{painted}"
        );
    }
    assert!(
        !super::unsigned(&unselected()),
        "no roles read, nothing claimed"
    );
}
