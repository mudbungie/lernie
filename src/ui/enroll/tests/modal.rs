//! **The two properties that make this pane a modal** (bl-7574): Escape closes
//! it, and nothing live paints under it.
//!
//! Split from [`super`] on a seam of its own — every other beat is about what
//! the pane says and composes, and these two are about what the rest of the
//! window is doing while it stands.

use super::fixtures::{file, opened};
use super::{CLOSE, HEADING, KEPT};
use crate::paint_probe::frame::{Window, press};
use crate::test_support::window::seated;
use crate::ui::composer::start::{GOAL, START};
use crate::ui::{Aim, Model};

/// **Escape closes it, and forgets what it holds.** The pane's stated purpose
/// is *look at this now and close it*, and the only exits were a click on
/// `done — forget it` or a Tab walk to it — reachable, which is not the same as
/// operable.
#[test]
fn escape_closes_the_enrollment_and_forgets_the_material() {
    let mut model = opened();
    file(&mut model);
    let window = Window::new();
    let mut body = |ctx: &egui::Context| crate::ui::render(ctx, &mut model);
    assert!(
        window.text(&mut body).contains(KEPT),
        "the symbol is on the glass to begin with"
    );
    window.frame(vec![press(egui::Key::Escape)], &mut body);
    let after = window.text(&mut body);
    assert_eq!(model.enroll, None, "the material is gone with the pane");
    assert!(!after.contains(KEPT), "{after}");
    assert!(
        !after.lines().any(|line| line == HEADING),
        "the pane itself is off the glass — the roster's own `enroll a box…`          control is not it:
{after}"
    );
}

/// Escape with nothing covering the window is still the notice's × reached
/// without a pointer — one key, and the contexts never overlap.
#[test]
fn escape_with_no_enrollment_is_still_the_notice_s_dismiss() {
    let mut model = Model {
        notice: Some(crate::ui::Notice::Refused("no".to_owned())),
        ..seated()
    };
    let window = Window::new();
    window.frame(vec![press(egui::Key::Escape)], |ctx| {
        crate::ui::render(ctx, &mut model);
    });
    assert_eq!(model.notice, None);
}

/// **A modal owns the arrows.** While the enrollment stands, an arrow walks
/// nothing: the lists behind it are not the subject of anything, so a walk
/// would re-aim the roster beneath the material — and would take the arrows out
/// of the name box the operator is typing into.
#[test]
fn the_arrows_walk_nothing_while_the_enrollment_stands() {
    let mut model = opened();
    let aimed = model.aim.clone();
    let window = Window::new();
    window.frame(vec![press(egui::Key::ArrowDown)], |ctx| {
        crate::ui::render(ctx, &mut model);
    });
    assert_eq!(model.aim, aimed, "the roster was walked under the material");
    assert!(model.enroll.is_some());
}

/// **Nothing live paints under it.** The composer is a bottom panel and so
/// outside what a central panel covers, which left a live `start` control —
/// firing a conversation on the very wall being enrolled into — beneath the
/// symbol.
#[test]
fn the_composer_stands_down_while_an_enrollment_is_open() {
    let mut aimed = Model {
        aim: Some(Aim {
            channel: "(this box's own engine)".to_owned(),
            address: "home".to_owned(),
        }),
        conversation: None,
        ..seated()
    };
    let window = Window::new();
    // **Two frames, because a panel sizes from the last one**: the composer's
    // buttons land on the row below its box, and a bottom panel that has never
    // been painted has not yet grown to hold them.
    let warm = |model: &mut Model| {
        window.text(|ctx| crate::ui::render(ctx, model));
        window.text(|ctx| crate::ui::render(ctx, model))
    };
    let before = warm(&mut aimed);
    for control in [START, GOAL] {
        assert!(
            before.contains(control),
            "{control:?} is there to lose:\n{before}"
        );
    }
    aimed.begin_enrollment();
    let during = warm(&mut aimed);
    for control in [START, GOAL] {
        assert!(
            !during.contains(control),
            "{control:?} is live under the material:\n{during}"
        );
    }
    assert!(
        during.contains(CLOSE),
        "the pane's own controls still stand:\n{during}"
    );
}
