//! An unmaking between frames: what opens one, what arms it, what it composes,
//! and the two ways it goes away.

use crate::test_support::window::seated;
use crate::ui::{Aim, Model, Unmaking};
use serde_json::json;

/// The wall the [`seated`] fixture is aimed at, spelled once.
fn subject(model: &Model) -> Aim {
    model.aim.clone().expect("the fixture is aimed at a wall")
}

/// **The aim is the gate**, exactly as it is for the enrollment and the tuning
/// pane: the gesture carries a workspace, and a workspace is what an aim is.
#[test]
fn it_opens_on_the_aimed_wall_and_not_at_all_without_one() {
    let mut nowhere = Model::default();
    nowhere.begin_unmaking();
    assert_eq!(nowhere.unmaking, None);

    let mut model = seated();
    model.begin_unmaking();
    assert_eq!(model.unmaking, Some(Unmaking::at(subject(&model))));
}

/// **It opens unarmed**, and only the workspace's own name arms it — which is
/// the engine's own comparison and not a policy this end invented.
#[test]
fn only_the_workspaces_own_name_arms_it() {
    let aim = subject(&seated());
    let mut held = Unmaking::at(aim.clone());
    assert!(!held.armed(), "it opens unarmed");
    for near in ["hom", "home ", " home", "Home", ""] {
        held.typed = near.to_owned();
        assert!(!held.armed(), "{near:?} is not the name");
    }
    held.typed = aim.address.clone();
    assert!(held.armed());
}

/// **What it composes is what the command line composes**, built by the same
/// verb row `lernie delete-workspace` spends — and nothing is sent from here.
#[test]
fn it_composes_the_envelope_its_verb_row_builds() {
    let mut model = seated();
    model.begin_unmaking();
    model.unmaking.as_mut().expect("just opened").typed = "home".to_owned();
    model.post_unmaking();
    assert_eq!(
        model.outbox,
        vec![json!({"op": "delete-workspace", "workspace": "home", "typed": "home"})]
    );
}

/// **Unarmed composes nothing**, so the disabled control is not the only thing
/// standing between a click and an unmaking.
#[test]
fn an_unarmed_pane_composes_nothing_however_it_is_reached() {
    let mut model = seated();
    model.begin_unmaking();
    model.post_unmaking();
    assert!(model.outbox.is_empty());
    assert_eq!(model.unmaking.map(|held| held.posted), Some(false));

    let mut closed = seated();
    closed.post_unmaking();
    assert!(closed.outbox.is_empty());
}

/// **The arming survives the firing**, because a refusal is the common case for
/// this act and clearing the box would charge a retype for the engine's *no*.
#[test]
fn firing_says_so_and_keeps_the_arming() {
    let mut model = seated();
    model.begin_unmaking();
    model.unmaking.as_mut().expect("just opened").typed = "home".to_owned();
    model.post_unmaking();
    model.post_unmaking();
    assert_eq!(
        model.unmaking,
        Some(Unmaking {
            aim: subject(&model),
            typed: "home".to_owned(),
            posted: true,
        })
    );
    assert_eq!(model.outbox.len(), 2, "a refusal may be retried");
}

/// **It holds its own subject.** The roster stays live and clickable under a
/// covering pane, so an aim that moved while an unmaking stood would otherwise
/// re-point an armed destructive act at a wall nobody armed.
#[test]
fn the_aim_moving_underneath_does_not_move_what_would_be_unmade() {
    let mut model = seated();
    model.begin_unmaking();
    model.aim_at("(this box's own engine)", "elsewhere");
    assert_eq!(
        model.unmaking.map(|held| held.aim.address),
        Some("home".to_owned())
    );
}

/// **Two ways out and neither unmakes anything**: the pane's own control, and
/// Escape reaching it without a pointer.
#[test]
fn both_ways_out_leave_it_holding_nothing() {
    for close in [Model::close_unmaking, Model::escape] {
        let mut model = seated();
        model.begin_unmaking();
        close(&mut model);
        assert_eq!(model.unmaking, None);
        assert!(model.outbox.is_empty());
    }
}

/// **It covers the conversation**, which is the one question the shell, the
/// roster's per-wall controls and the keyboard gate all share.
#[test]
fn it_is_a_covering_pane() {
    let mut model = seated();
    assert!(!model.covered());
    model.begin_unmaking();
    assert!(model.covered());
}
