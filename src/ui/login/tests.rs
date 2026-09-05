//! What the login pane says, what it offers, and what one click on it composes.

use super::{
    CLOSE, ELSEWHERE, HEADING, LOOPBACK, NO_PROVIDERS, NOT_ANSWERED, NOT_OFFERED, OFFERS,
    OFFERS_NOTHING, OPEN, SIGN_IN, render,
};
use crate::paint_probe::frame::Window;
use crate::test_support::window::{click, pane, provider, seated, signing};
use crate::ui::{Login, Model};
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
/// has been answered about is not a wall that routes no provider.
#[test]
fn an_unanswered_wall_and_a_wall_with_no_provider_say_different_things() {
    for (providers, expected) in [
        (None, NOT_ANSWERED),
        (Some(Vec::new()), NO_PROVIDERS),
        (Some(vec![provider("housevendor")]), "housevendor"),
    ] {
        let mut model = Model {
            providers,
            login: Some(Login::default()),
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

/// One row states what the engine knows about its credential and what it takes,
/// and offers both of the pane's controls.
#[test]
fn a_row_paints_its_credential_fact_and_offers_both_controls() {
    let mut model = signing();
    let painted = pane(|ui| {
        render(ui, &mut model);
    });
    for word in [
        "housevendor",
        "credential present",
        "takes effort and priority",
        SIGN_IN,
        OFFERS,
    ] {
        assert!(painted.contains(word), "{word:?}:\n{painted}");
    }
}

/// **A blocked row says the engine's own reason and is not offered a sign-in.**
/// A control that fired an act the far end has already refused is a control
/// that only looks actionable.
#[test]
fn a_blocked_row_states_the_reason_and_its_control_is_not_live() {
    let mut model = signing();
    let painted = pane(|ui| {
        render(ui, &mut model);
    });
    assert!(painted.contains("no login flow"), "{painted}");
    // The blocked row is the second; clicking the row that CAN be signed in to
    // is what fires, and the disabled seat below is what proves the other
    // cannot.
    let blocked = model
        .providers
        .clone()
        .expect("the table")
        .into_iter()
        .find(|row| row.name == "otherhouse")
        .expect("the blocked row");
    assert!(!blocked.signable());
}

/// **What a row offers is painted under the row that ASKED**, because the reply
/// carries no name — and the two emptinesses are two sentences.
#[test]
fn an_offering_is_painted_under_the_row_that_asked_for_it() {
    let mut model = signing();
    let painted = pane(|ui| {
        render(ui, &mut model);
    });
    assert!(painted.contains("house-model-1"), "{painted}");
    for (offered, expected) in [(None, NOT_OFFERED), (Some(Vec::new()), OFFERS_NOTHING)] {
        let mut model = Model {
            offered,
            ..signing()
        };
        let painted = pane(|ui| {
            render(ui, &mut model);
        });
        assert!(
            painted.lines().any(|line| line == expected),
            "{expected:?}:\n{painted}"
        );
    }
    let mut unasked = Model {
        login: Some(Login::default()),
        ..signing()
    };
    let painted = pane(|ui| {
        render(ui, &mut unasked);
    });
    assert!(
        !painted.contains("house-model-1"),
        "nothing was asked, so nothing is attributed:\n{painted}"
    );
}

/// **A wall held elsewhere says so and states the loopback remedy**, off the
/// channel stamp and with no address in either sentence.
#[test]
fn a_wall_held_elsewhere_says_where_the_sign_in_runs_and_states_the_remedy() {
    let mut here = signing();
    let painted = pane(|ui| {
        render(ui, &mut here);
    });
    assert!(!painted.contains(ELSEWHERE), "{painted}");
    let mut model = signing();
    model.roster[0].channel.named_there = Some("theirs".to_owned());
    let painted = pane(|ui| {
        render(ui, &mut model);
    });
    for said in [ELSEWHERE, LOOPBACK] {
        assert!(painted.contains(said), "{said:?}:\n{painted}");
    }
}

/// Both controls compose the gesture the wire takes, and the click is what
/// proves the tag on each names the op it fires.
#[test]
fn clicking_the_two_row_controls_composes_the_act_and_the_read() {
    for (label, expected) in [
        (
            SIGN_IN,
            crate::ui::Posted::act(
                json!({"op": "login", "workspace": "home", "provider": "housevendor"}),
            ),
        ),
        (
            OFFERS,
            crate::ui::Posted::read(
                json!({"op": "models", "workspace": "home", "provider": "housevendor"}),
            ),
        ),
    ] {
        let mut model = Model {
            providers: Some(vec![provider("housevendor")]),
            login: Some(Login::default()),
            ..seated()
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
    let mut model = signing();
    let window = Window::new();
    click(&window, CLOSE, |ctx| {
        egui::CentralPanel::default().show(ctx, |ui| {
            render(ui, &mut model);
        });
    });
    assert_eq!(model.login, None);
}

/// **The pane is what the roster's control opens**, and the whole window is
/// where that is true — the control lives on the aimed row and nowhere else.
#[test]
fn the_roster_control_opens_the_pane_and_the_pane_covers_the_conversation() {
    let mut model = seated();
    let window = Window::new();
    click(&window, OPEN, |ctx| crate::ui::render(ctx, &mut model));
    assert_eq!(model.login, Some(Login::default()));
    let painted = window.text(|ctx| crate::ui::render(ctx, &mut model));
    assert!(painted.lines().any(|line| line == HEADING), "{painted}");
    assert!(
        !painted.contains(crate::ui::composer::SEND),
        "the composer stands down under it:\n{painted}"
    );
}
