//! What the tuning pane says, what it offers, and what one click on it
//! composes.

use super::{
    ASSIGN, CANCEL, CLOSE, HEADING, MODEL_HINT, NO_ROLES, NOT_ANSWERED, OPEN, PRIORITY,
    PROVIDER_HINT, SET, render, unseated,
};
use crate::paint_probe::frame::Window;
use crate::test_support::window::{click, pane, role, seated, tuned};
use crate::ui::{Edit, Model, Tuning};
use serde_json::json;

/// A closed pane paints nothing and says so, which is what lets the shell put
/// the conversation back where it was.
#[test]
fn a_shut_pane_paints_nothing_and_reports_it() {
    let mut model = seated();
    let mut stood = true;
    let painted = pane(|ui| stood = render(ui, &mut model));
    assert!(!stood, "a shut pane reports that it painted nothing");
    assert!(!painted.contains(HEADING), "{painted}");
}

/// **Three states, and the first two are different sentences.** A wall nobody
/// has been answered about is not a wall with no roles — the same doctrine the
/// conversation list keeps one noun over.
#[test]
fn an_unanswered_wall_and_a_wall_with_no_roles_say_different_things() {
    for (roles, expected) in [
        (None, NOT_ANSWERED),
        (Some(Vec::new()), NO_ROLES),
        (Some(vec![role("worker")]), "worker"),
    ] {
        let mut model = Model {
            roles,
            tuning: Some(Tuning::Rows),
            ..seated()
        };
        let painted = pane(|ui| {
            render(ui, &mut model);
        });
        assert!(
            painted.lines().any(|line| line == expected),
            "{expected:?}:\n{painted}"
        );
    }
}

/// One row states what it runs on and offers every level plus the lane.
#[test]
fn a_row_paints_what_it_runs_on_and_every_level_a_control_offers() {
    let mut model = tuned();
    let painted = pane(|ui| {
        render(ui, &mut model);
    });
    for word in [
        "worker",
        "housevendor  house-model-1",
        "low",
        "medium",
        "high",
        "off",
        PRIORITY,
        ASSIGN,
    ] {
        assert!(painted.contains(word), "{word:?}:\n{painted}");
    }
}

/// **A level with no seat is painted as itself** — rung 3 on the glass. Four
/// unselected seats would otherwise say the level is `off`, which is a
/// different claim from the one the config makes.
#[test]
fn a_level_this_pane_has_no_seat_for_is_said_in_its_own_word() {
    let mut row = role("worker");
    row.effort = Some("extreme".to_owned());
    assert!(
        unseated(&row)
            .expect("a word with no seat")
            .contains("extreme"),
        "the word is carried into the sentence"
    );
    for known in [Some("high".to_owned()), None] {
        let mut row = role("worker");
        row.effort = known;
        assert_eq!(unseated(&row), None, "a level with a seat says nothing");
    }
    let mut model = Model {
        roles: Some(vec![{
            let mut row = role("worker");
            row.effort = Some("extreme".to_owned());
            row
        }]),
        ..tuned()
    };
    let painted = pane(|ui| {
        render(ui, &mut model);
    });
    assert!(painted.contains("extreme"), "{painted}");
}

/// Every seat composes the gesture the wire takes, and the click is what proves
/// the tag on it names the op it fires.
#[test]
fn clicking_a_level_and_the_lane_composes_the_two_tuning_writes() {
    for (label, expected) in [
        (
            "high",
            json!({"op": "effort", "workspace": "home", "role": "worker", "level": "high"}),
        ),
        (
            "off",
            json!({"op": "effort", "workspace": "home", "role": "worker", "level": null}),
        ),
        (
            PRIORITY,
            json!({"op": "priority", "workspace": "home", "role": "worker", "on": true}),
        ),
    ] {
        let mut model = Model {
            roles: Some(vec![role("worker")]),
            ..tuned()
        };
        let window = Window::new();
        click(&window, label, |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                render(ui, &mut model);
            });
        });
        assert_eq!(model.outbox, vec![expected], "{label}");
    }
}

/// The close control puts the pane down and nothing else.
#[test]
fn the_done_control_closes_the_pane() {
    let mut model = tuned();
    let window = Window::new();
    click(&window, CLOSE, |ctx| {
        egui::CentralPanel::default().show(ctx, |ui| {
            render(ui, &mut model);
        });
    });
    assert_eq!(model.tuning, None);
}

/// **The editor opens under its own row**, holds two boxes, and both of its
/// controls do what they say.
#[test]
fn the_assignment_editor_opens_spends_and_stands_down() {
    let mut model = tuned();
    let window = Window::new();
    click(&window, ASSIGN, |ctx| {
        egui::CentralPanel::default().show(ctx, |ui| {
            render(ui, &mut model);
        });
    });
    assert!(model.editing("worker").is_some(), "the first row's editor");
    let painted = pane(|ui| {
        render(ui, &mut model);
    });
    // **The boxes open holding what is in force**, which is why the two hints
    // are not on the glass here: a hint is what an empty box says, and these
    // are seeded. The hints' own beat is below, over an empty draft.
    for word in ["housevendor", "house-model-1", SET, CANCEL] {
        assert!(painted.contains(word), "{word:?}:\n{painted}");
    }
    click(&window, CANCEL, |ctx| {
        egui::CentralPanel::default().show(ctx, |ui| {
            render(ui, &mut model);
        });
    });
    assert_eq!(model.tuning, Some(Tuning::Rows), "put down, not sent");
}

/// **`set` is live only when both halves are named**, and what it composes is
/// the row the command line would have spelled.
#[test]
fn set_spends_a_whole_draft_and_a_half_one_composes_nothing() {
    let mut model = Model {
        tuning: Some(Tuning::Editing(Edit {
            role: "worker".to_owned(),
            provider: String::new(),
            model: String::new(),
        })),
        ..tuned()
    };
    let painted = pane(|ui| {
        render(ui, &mut model);
    });
    for hint in [PROVIDER_HINT, MODEL_HINT] {
        assert!(painted.contains(hint), "{hint:?}:\n{painted}");
    }
    let window = Window::new();
    click(&window, SET, |ctx| {
        egui::CentralPanel::default().show(ctx, |ui| {
            render(ui, &mut model);
        });
    });
    assert!(
        model.outbox.is_empty(),
        "an empty draft is not an assignment"
    );
    model.tuning = Some(Tuning::Editing(Edit::of(&role("worker"))));
    click(&window, SET, |ctx| {
        egui::CentralPanel::default().show(ctx, |ui| {
            render(ui, &mut model);
        });
    });
    assert_eq!(
        model.outbox,
        vec![json!({"op": "model", "workspace": "home", "role": "worker",
                    "provider": "housevendor", "model": "house-model-1"})]
    );
}

/// **The pane is what the roster's control opens**, and the whole window is
/// where that is true — the control lives on the aimed row and nowhere else.
#[test]
fn the_roster_control_opens_the_pane_and_the_pane_covers_the_conversation() {
    let mut model = seated();
    let window = Window::new();
    click(&window, OPEN, |ctx| crate::ui::render(ctx, &mut model));
    assert_eq!(model.tuning, Some(Tuning::Rows));
    let painted = window.text(|ctx| crate::ui::render(ctx, &mut model));
    assert!(painted.lines().any(|line| line == HEADING), "{painted}");
    assert!(
        !painted.contains(crate::ui::composer::SEND),
        "the composer stands down under it:\n{painted}"
    );
}
