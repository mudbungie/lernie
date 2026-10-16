//! **The bar's edge drags, in both states** (DESIGN §4.45 item 2; bl-7578):
//! which width a drag writes, and that a drag changes the width it began on
//! and nothing else — asserted on the glass, the width read back off the
//! painted panel. What bounds the widths is [`super::bounds`]'.

use super::super::drag::{HANDLE, floor};
use super::super::{DESK, EASE, LINGER, SHUT, open_width};
use super::{opened, panel, step, tick};
use crate::test_support::window::seated;

/// Where along the edge the pointer grabs it.
const Y: f32 = 450.0;

/// One pointer-button event, built outside the generic below for
/// `paint_probe::frame::down`'s own reason.
fn button(pos: egui::Pos2, pressed: bool) -> egui::Event {
    egui::Event::PointerButton {
        pos,
        button: egui::PointerButton::Primary,
        pressed,
        modifiers: egui::Modifiers::NONE,
    }
}

/// **One whole drag of the edge grabbed at x `from` and moved `by`**, on a
/// window `width` wide, the frames `beat` apart from `time`: hover, press,
/// move, move, release — the hover first because egui hit-tests a press
/// against the previous frame's rects. Answers the release frame.
pub(super) fn drag(
    ctx: &egui::Context,
    model: &mut crate::ui::Model,
    width: f32,
    (time, beat): (f64, f64),
    from: f32,
    by: f32,
) -> egui::FullOutput {
    let to = from + by;
    let at = |x| egui::pos2(x, Y);
    let frames = [
        egui::Event::PointerMoved(at(from)),
        button(at(from), true),
        egui::Event::PointerMoved(at(to)),
        egui::Event::PointerMoved(at(to)),
        button(at(to), false),
    ];
    let mut last = None;
    for (i, event) in (0u8..).zip(frames) {
        let now = f64::from(i).mul_add(beat, time);
        last = Some(step(ctx, model, width, now, vec![event]));
    }
    last.expect("five frames")
}

/// A number within a hundredth of `want`.
pub(super) fn near(got: Option<f32>, want: f32) -> bool {
    got.is_some_and(|got| (got - want).abs() < 0.01)
}

/// **Dragged while collapsed, the edge sets the collapsed width** and leaves
/// the expanded one the default: the pointer reaching for the edge does not
/// open the bar, the edge wears the resize cursor, the bar stays the dragged
/// column at rest, and the pointer over its rows still opens it to the
/// default expanded width.
#[test]
fn a_collapsed_drag_sets_the_collapsed_width_and_leaves_the_expanded_one() {
    let ctx = egui::Context::default();
    let mut model = seated();
    tick(&ctx, &mut model, 0.0, None);
    let grip = egui::pos2(SHUT - HANDLE / 2.0, Y);
    let hovered = step(
        &ctx,
        &mut model,
        DESK,
        0.5,
        vec![egui::Event::PointerMoved(grip)],
    );
    assert!(!opened(&hovered), "the edge is not the bar");
    assert_eq!(
        hovered.platform_output.cursor_icon,
        egui::CursorIcon::ResizeHorizontal
    );
    let released = drag(&ctx, &mut model, DESK, (1.0, 0.02), grip.x, 40.0);
    assert!(!opened(&released), "shut through the drag");
    assert!(
        near(model.settings.bar_shut, SHUT + 40.0),
        "{:?}",
        model.settings
    );
    assert_eq!(
        model.settings.bar_open, None,
        "the expanded one is untouched"
    );
    assert_eq!(model.bar.drag, None, "put down on release");
    let rest = tick(&ctx, &mut model, 2.0, None);
    assert!(!opened(&rest));
    assert!(
        near(Some(panel(&ctx)), SHUT + 40.0),
        "the glass shows the column"
    );
    let over = Some(egui::pos2(10.0, Y));
    tick(&ctx, &mut model, 3.0, over);
    tick(&ctx, &mut model, 3.0 + f64::from(EASE), over);
    tick(&ctx, &mut model, 3.1 + f64::from(EASE), over);
    assert!(
        near(Some(panel(&ctx)), open_width(DESK)),
        "the default, eased to"
    );
}

/// **Dragged while expanded, the edge sets the expanded width** and leaves
/// the collapsed one the default, which the bar eases back to once unpinned.
#[test]
fn an_expanded_drag_sets_the_expanded_width_and_leaves_the_collapsed_one() {
    let ctx = egui::Context::default();
    let mut model = seated();
    model.bar.pinned = true;
    tick(&ctx, &mut model, 0.0, None);
    let wide = open_width(DESK);
    let released = drag(
        &ctx,
        &mut model,
        DESK,
        (1.0, 0.02),
        wide - HANDLE / 2.0,
        100.0,
    );
    assert!(opened(&released), "open through the drag");
    assert!(
        near(model.settings.bar_open, wide + 100.0),
        "{:?}",
        model.settings
    );
    assert_eq!(
        model.settings.bar_shut, None,
        "the collapsed one is untouched"
    );
    assert!(near(Some(panel(&ctx)), wide + 100.0));
    model.bar.pinned = false;
    model.bar.left_at = None;
    tick(&ctx, &mut model, 2.0, None);
    tick(&ctx, &mut model, 2.0 + f64::from(EASE), None);
    tick(&ctx, &mut model, 2.1 + f64::from(EASE), None);
    assert!(
        near(Some(panel(&ctx)), SHUT),
        "the default column, eased to"
    );
}

/// **A drag writes the width it began on, whatever the pointer does**: an
/// edge grabbed on a bar open by the clock outlasts the linger and the bar
/// stays open under it, writing the expanded width; an edge grabbed on a
/// collapsed bar and dragged into the rows does not open it, and writes the
/// collapsed width — down to the floor and no further.
#[test]
fn a_drag_writes_the_width_it_began_on_whatever_the_pointer_does() {
    let ctx = egui::Context::default();
    let mut model = seated();
    let over = Some(egui::pos2(10.0, Y));
    tick(&ctx, &mut model, 0.0, over);
    tick(&ctx, &mut model, 0.2, over);
    let wide = open_width(DESK);
    let released = drag(
        &ctx,
        &mut model,
        DESK,
        (1.0, f64::from(LINGER)),
        wide - HANDLE / 2.0,
        -100.0,
    );
    assert!(
        opened(&released),
        "open through a drag longer than the linger"
    );
    assert!(
        near(model.settings.bar_open, wide - 100.0),
        "{:?}",
        model.settings
    );
    assert_eq!(model.settings.bar_shut, None);
    let away = Some(egui::pos2(900.0, Y));
    assert!(
        opened(&tick(
            &ctx,
            &mut model,
            4.0f64.mul_add(f64::from(LINGER), 1.1),
            away
        )),
        "lingering from the release"
    );
    assert!(!opened(&tick(&ctx, &mut model, 20.0, away)), "lapsed");
    let released = drag(
        &ctx,
        &mut model,
        DESK,
        (21.0, 0.02),
        SHUT - HANDLE / 2.0,
        -SHUT,
    );
    assert!(
        !opened(&released),
        "shut through a drag that crossed into the rows"
    );
    assert!(
        near(model.settings.bar_shut, floor(&ctx)),
        "{:?}",
        model.settings
    );
    assert!(near(model.settings.bar_open, wide - 100.0), "untouched");
}
