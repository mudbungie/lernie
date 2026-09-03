//! The unmaking pane: what it says, what it offers, and the one control that
//! is on the glass without being live.

use super::{ARM, ARMED, ASKED, CLOSE, CONFIRM, HEADING, NOT_ARMED, REFUSED_IF, render, said};
use crate::paint_probe::frame::Window;
use crate::test_support::window::{click, pane, seated};
use crate::ui::{Model, Unmaking};
use serde_json::json;

/// The seated model with an unmaking standing on its wall.
fn arming() -> Model {
    let mut model = seated();
    model.begin_unmaking();
    model
}

/// The same, armed.
fn armed() -> Model {
    let mut model = arming();
    model.unmaking.as_mut().expect("just opened").typed = "home".to_owned();
    model
}

/// Paint the pane on its own and click the control reading `label`.
fn press(model: &mut Model, label: &str) {
    let window = Window::new();
    click(&window, label, |ctx| {
        egui::CentralPanel::default().show(ctx, |ui| {
            render(ui, model);
        });
    });
}

/// **Nothing is painted where nothing is being unmade**, and the shell reads
/// that answer to know the conversation still stands.
#[test]
fn it_paints_nothing_at_all_with_no_unmaking_open() {
    let mut model = seated();
    let painted = pane(|ui| {
        assert!(!render(ui, &mut model));
    });
    assert_eq!(painted.trim(), "");
}

/// **It names its subject and states the refusal before offering the act** —
/// the wall, the channel, what the engine will decline it for, and the box that
/// arms it.
#[test]
fn it_says_what_would_be_unmade_and_what_would_refuse_it() {
    let mut model = arming();
    let painted = pane(|ui| {
        assert!(render(ui, &mut model));
    });
    for word in [HEADING, "home on (this box's own engine)", REFUSED_IF, ARM] {
        assert!(
            painted.lines().any(|line| line == word),
            "{word:?}:\n{painted}"
        );
    }
}

/// **The three sentences an arming can be in**, read back as a value rather
/// than looked for on a screen.
#[test]
fn the_pane_says_which_of_the_three_states_it_is_in() {
    let aim = seated().aim.expect("the fixture is aimed at a wall");
    let mut held = Unmaking::at(aim);
    assert_eq!(said(&held), NOT_ARMED);
    held.typed = "home".to_owned();
    assert_eq!(said(&held), ARMED);
    held.posted = true;
    assert_eq!(said(&held), ASKED);
}

/// **Unarmed, the act is on the glass and not live** — disabled and not absent,
/// so the control says what would fill it rather than vanishing.
#[test]
fn the_act_is_painted_unarmed_and_fires_nothing() {
    let mut model = arming();
    let painted = pane(|ui| {
        render(ui, &mut model);
    });
    for word in [CONFIRM, CLOSE, NOT_ARMED] {
        assert!(
            painted.lines().any(|line| line == word),
            "{word:?}:\n{painted}"
        );
    }
    press(&mut model, CONFIRM);
    assert!(model.outbox.is_empty(), "a disabled control fires nothing");
}

/// **Armed, it composes the envelope the verb row builds**, and the arming
/// stays where it was typed.
#[test]
fn armed_it_composes_the_unmaking_and_keeps_the_arming() {
    let mut model = armed();
    press(&mut model, CONFIRM);
    assert_eq!(
        model.outbox,
        vec![crate::ui::Posted::act(
            json!({"op": "delete-workspace", "workspace": "home", "typed": "home"})
        )]
    );
    assert_eq!(
        model.unmaking.as_ref().map(|held| held.typed.clone()),
        Some("home".to_owned())
    );
    let painted = pane(|ui| {
        render(ui, &mut model);
    });
    assert!(painted.lines().any(|line| line == ASKED), "{painted}");
}

/// **The way out unmakes nothing**, and it is painted before the act it stands
/// beside — so the control an operator reaches for by reflex is the safe one.
#[test]
fn the_way_out_is_first_and_composes_nothing() {
    let mut model = armed();
    let painted = pane(|ui| {
        render(ui, &mut model);
    });
    let index = |word: &str| {
        painted
            .lines()
            .position(|line| line == word)
            .unwrap_or_else(|| panic!("{word:?}:\n{painted}"))
    };
    assert!(index(CLOSE) < index(CONFIRM), "{painted}");

    press(&mut model, CLOSE);
    assert_eq!(model.unmaking, None);
    assert!(model.outbox.is_empty());
}
