//! The three acts one obligation's attempts earn: which row earns which, the
//! word each stands down without, and what a click composes.

use super::{ACCEPT, FEWER, GOAL_HINT, MORE, RELEASE, SPREAD, SUMMARY_HINT};
use crate::paint_probe::frame::Window;
use crate::test_support::window::{click, fleeting, pane};
use crate::ui::Model;

/// **Both kinds of row are on the glass with the acts each earns**, and the
/// two boxes the three acts spend are above them.
#[test]
fn a_candidate_and_a_claim_each_wear_the_acts_they_earn() {
    let mut model = fleeting();
    let painted = pane(|ui| {
        super::super::render(ui, &mut model);
    });
    for word in [ACCEPT, RELEASE, GOAL_HINT, SUMMARY_HINT, "spread it over 3"] {
        assert!(painted.contains(word), "{word:?}:\n{painted}");
    }
}

/// **The count is a stepper and it never goes below two**: upstream reads one
/// and zero as *materialize nothing and hand back the ordinary claim binding*,
/// which is a start rather than a fan.
#[test]
fn the_count_steps_and_floors_at_two() {
    let window = Window::new();
    let mut model = fleeting();
    for _ in 0..4 {
        click(&window, FEWER, |ctx| crate::ui::render(ctx, &mut model));
    }
    assert_eq!(model.fleet.as_ref().map(|held| held.spread), Some(2));
    click(&window, MORE, |ctx| crate::ui::render(ctx, &mut model));
    assert_eq!(model.fleet.as_ref().map(|held| held.spread), Some(3));
}

/// **Accepting one composes the delivery off the ROW** — the obligation and
/// the handle both — and it is addressed down the channel the pane stands on,
/// because the envelope names no workspace at all (§4.30's ruling).
#[test]
fn accepting_a_candidate_composes_the_delivery_addressed_to_one_channel() {
    let window = Window::new();
    let mut model = fleeting();
    if let Some(fleet) = model.fleet.as_mut() {
        fleet.summary = "take the winner".to_owned();
    }
    click(&window, ACCEPT, |ctx| crate::ui::render(ctx, &mut model));
    let posted = model.outbox.first().expect("one act");
    assert_eq!(posted.envelope["op"], "deliver");
    assert_eq!(posted.envelope["ball"], "bl-3");
    assert_eq!(posted.envelope["handle"], "at-0badcafe");
    assert_eq!(posted.envelope["summary"], "take the winner");
    assert_eq!(
        posted.channel.as_ref().map(|held| held.name.clone()),
        model.aim.as_ref().map(|aim| aim.channel.clone())
    );
}

/// **An acceptance with no subject composes nothing**, which is §4.20's
/// enablement: the parameter is missing, not the subject.
#[test]
fn an_acceptance_with_no_subject_composes_nothing() {
    let window = Window::new();
    let mut model = fleeting();
    click(&window, ACCEPT, |ctx| crate::ui::render(ctx, &mut model));
    assert!(model.outbox.is_empty());
}

/// **Releasing a worktree needs no word and is not armed**: it changes no
/// delivery target, and whether the source ref went with it is the project's
/// own declared retention, which the engine answers and this seat paints.
#[test]
fn releasing_a_worktree_composes_the_retirement() {
    let window = Window::new();
    let mut model = fleeting();
    click(&window, RELEASE, |ctx| crate::ui::render(ctx, &mut model));
    let posted = model.outbox.first().expect("one act");
    assert_eq!(posted.envelope["op"], "retire");
    assert_eq!(posted.envelope["handle"], "at-0badcafe");
    assert!(posted.channel.is_some(), "addressed, never fanned");
}

/// **The spread is staged, not sent** — its first act is the ordinary
/// `prepare`, and the `fan` is composed by the frame that takes its receipt.
#[test]
fn spreading_a_claim_stages_the_start_the_fan_will_carry() {
    let window = Window::new();
    let mut model = fleeting();
    if let Some(fleet) = model.fleet.as_mut() {
        fleet.goal = "try three ways".to_owned();
    }
    click(&window, &format!("{SPREAD} 3"), |ctx| {
        crate::ui::render(ctx, &mut model);
    });
    let posted = model.outbox.first().expect("one act");
    assert_eq!(posted.envelope["op"], "prepare");
    assert_eq!(posted.envelope["workspace"], "home");
    let spread = model
        .start
        .as_ref()
        .and_then(|start| start.spread.clone())
        .expect("the obligation is held");
    assert_eq!(spread.ball, "bl-2", "off the claim row, never a box");
    assert_eq!(spread.n, 3);
}

/// **A spread with no goal composes nothing**: n conversations nobody said
/// anything to is n drivers launched for nothing.
#[test]
fn a_spread_with_no_goal_composes_nothing() {
    let window = Window::new();
    let mut model = fleeting();
    click(&window, &format!("{SPREAD} 3"), |ctx| {
        crate::ui::render(ctx, &mut model);
    });
    assert!(model.outbox.is_empty());
    assert_eq!(model.start, None);
}

/// A pane that is not open paints no listing, which is the state the shell
/// never reaches made unreachable rather than merely unlikely.
#[test]
fn a_shut_pane_paints_no_listing() {
    let mut model = Model {
        fleet: None,
        ..fleeting()
    };
    let painted = pane(|ui| {
        super::render(ui, &mut model, &[]);
    });
    assert!(!painted.contains(ACCEPT), "{painted}");
}
