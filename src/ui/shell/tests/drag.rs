//! **Every visible edge is one an operator can drag** (DESIGN §4.39, bl-46e5),
//! asserted on the glass rather than in the policy — `super::super::policy::
//! edges`' suite holds the arithmetic.
//!
//! **There are two of them since the fold** (bl-b9a3): the list's edge against
//! the conversation, and the composer's top. The third — between the two list
//! panes — went with the middle column.
//!
//! The regression these exist for is the SNAP-BACK. egui floors a side panel's
//! content at the width range's minimum and stores the content's rect, so a
//! pane pulled wider than its rows reported the rows' width and shrank to it on
//! the next pass — which is why bl-fef8 concluded the drag could not work and
//! took the handle away. It works; what it needed was one line in the body.

use egui::containers::panel::PanelState;

use super::super::{policy, render};
use crate::paint_probe::frame::Window;
use crate::test_support::window::seated;
use crate::ui::{Model, theme};

/// The window every beat here runs in: wide enough for the broad shape with
/// room to spare, which is the only shape that has an edge to drag.
const WIDE: (f32, f32) = (1440.0, 900.0);

/// **The rect a panel took, as egui itself stores it** — the one number a drag
/// is about, read where the toolkit keeps it rather than inferred from a
/// glyph's position.
///
/// Two settled frames, for `width`'s own reason: egui lays a panel out on the
/// pass after the one that measured it, so a single pass answers about a
/// window mid-layout.
fn panel(window: &Window, name: &str, model: &mut Model) -> egui::Rect {
    let mut rect = egui::Rect::ZERO;
    for _ in 0..2 {
        window.frame(Vec::new(), |ctx| {
            render(ctx, model);
            rect =
                PanelState::load(ctx, egui::Id::new(name)).map_or(egui::Rect::ZERO, |at| at.rect);
        });
    }
    rect
}

/// One pointer-button event, built outside the generic below for
/// `paint_probe::frame::down`'s own reason: a multi-line literal inside a
/// closure-generic function reads back as regions nothing ran.
fn button(pos: egui::Pos2, pressed: bool) -> egui::Event {
    egui::Event::PointerButton {
        pos,
        button: egui::PointerButton::Primary,
        pressed,
        modifiers: egui::Modifiers::NONE,
    }
}

/// **One whole drag of an edge: hover, press, move, move, release.**
///
/// Five frames and none of them spare. egui hit-tests a press against the
/// PREVIOUS frame's widget rects, so the hover has to land first; and a panel
/// reads its resize interaction off the previous frame's response, so the
/// frame that moves the pointer is not the frame that moves the edge.
fn drag(window: &Window, from: egui::Pos2, to: egui::Pos2, mut body: impl FnMut(&egui::Context)) {
    window.frame(vec![egui::Event::PointerMoved(from)], &mut body);
    window.frame(vec![button(from, true)], &mut body);
    for _ in 0..2 {
        window.frame(vec![egui::Event::PointerMoved(to)], &mut body);
    }
    window.frame(vec![button(to, false)], &mut body);
}

/// **A pane dragged WIDER than its rows stays there.** The snap-back, which is
/// the whole reason the handle was taken away: the list's rows are a few short
/// words, so a pane pulled to 520 points reported ~200 on the next frame and
/// shrank to it. Three idle frames after the release, because once was never
/// the failure.
#[test]
fn a_pane_dragged_wider_than_its_rows_does_not_snap_back_to_them() {
    let mut model = seated();
    let window = Window::sized(WIDE.0, WIDE.1);
    let edge = panel(&window, "engines", &mut model).right();
    let at = egui::pos2(edge, 400.0);
    drag(&window, at, egui::pos2(520.0, 400.0), |ctx| {
        render(ctx, &mut model);
    });
    assert!(
        model
            .dragged
            .list
            .is_some_and(|width| (width - 520.0).abs() < 1.0),
        "the seat holds what the operator dragged: {:?}",
        model.dragged.list
    );
    for pass in 0..3 {
        let held = panel(&window, "engines", &mut model).width();
        assert!(
            (held - 520.0).abs() < 1.0,
            "pass {pass}: the pane snapped back to {held}"
        );
    }
}

/// **A window too narrow for the drag narrows the pane and forgets nothing.**
/// The shown width is what this window can afford; what the seat holds is what
/// the operator set, so the pane comes back whole when the window does.
#[test]
fn a_narrower_window_clamps_the_drag_and_never_overwrites_it() {
    let mut model = seated();
    let mut window = Window::sized(WIDE.0, WIDE.1);
    let edge = panel(&window, "engines", &mut model).right();
    drag(
        &window,
        egui::pos2(edge, 400.0),
        egui::pos2(520.0, 400.0),
        |ctx| render(ctx, &mut model),
    );
    window.resize(900.0, 900.0);
    let squeezed = panel(&window, "engines", &mut model).width();
    assert!(squeezed < 520.0, "the narrow window shows {squeezed}");
    assert_eq!(
        model.dragged.list.map(f32::round),
        Some(520.0),
        "and the seat still holds the drag"
    );
    window.resize(WIDE.0, WIDE.1);
    let back = panel(&window, "engines", &mut model).width();
    assert!((back - 520.0).abs() < 1.0, "it came back to {back}");
}

/// **A pane nobody dragged follows the policy**, which is bl-fef8's own
/// regression and the half §4.39 did not reverse: the seat holds nothing for
/// that edge, so the width is the yield's at every window size.
#[test]
fn an_edge_nobody_dragged_still_follows_the_policy_when_the_window_moves() {
    let mut model = seated();
    let mut window = Window::sized(800.0, 600.0);
    panel(&window, "engines", &mut model);
    window.resize(WIDE.0, WIDE.1);
    let held = panel(&window, "engines", &mut model).width();
    assert_eq!(model.dragged.list, None, "nobody dragged anything");
    assert!(
        (held - policy::widths(WIDE.0)).abs() < 1.0,
        "{held} against the policy's {}",
        policy::widths(WIDE.0)
    );
}

/// **The composer's top edge sets how many ROWS the field is**, which is
/// §4.38's bound kept: the panel is handed a height computed from a row count
/// and never reads one back off its content.
#[test]
fn the_composers_top_edge_sets_the_rows_and_both_directions_work() {
    let mut model = seated();
    let window = Window::sized(WIDE.0, WIDE.1);
    // **The composer stands beside the list panes, not over them**: it is a
    // bottom panel added after them, so its own middle is where its edge is.
    let edge = panel(&window, "composer", &mut model);
    drag(
        &window,
        egui::pos2(edge.center().x, edge.top()),
        egui::pos2(edge.center().x, edge.top() - 80.0),
        |ctx| render(ctx, &mut model),
    );
    let taller = model.dragged.rows.expect("the seat holds the drag");
    assert!(taller > theme::COMPOSER_ROWS, "{taller} rows is not taller");
    let edge = panel(&window, "composer", &mut model);
    drag(
        &window,
        egui::pos2(edge.center().x, edge.top()),
        egui::pos2(edge.center().x, edge.top() + 200.0),
        |ctx| render(ctx, &mut model),
    );
    let shorter = model.dragged.rows.expect("the seat holds the drag");
    assert!(
        shorter < taller,
        "{shorter} rows is not shorter than {taller}"
    );
}

/// **The narrow shape consults no drag**, because nothing competes for the
/// width there: one column has the whole window, and there is no edge on it.
#[test]
fn the_narrow_shape_has_no_edge_and_reads_no_drag() {
    let mut model = Model {
        dragged: crate::ui::Dragged {
            list: Some(600.0),
            ..crate::ui::Dragged::default()
        },
        ..seated()
    };
    let window = Window::sized(400.0, 800.0);
    assert_eq!(
        panel(&window, "engines", &mut model),
        egui::Rect::ZERO,
        "there is no list panel at all"
    );
    assert_eq!(model.dragged.list, Some(600.0), "and nothing wrote to it");
}
