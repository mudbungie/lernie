//! **What bounds the bar's two widths** (DESIGN §4.45 item 2; bl-7578): the
//! defaults a seat that dragged nothing gets, the floor, the ceiling and each
//! width against the other — as a value, and on the glass, where a drag past
//! a bound stops at it and a narrower window clamps and overwrites nothing.

use super::super::drag::{HANDLE, ceiling, shown};
use super::super::{DESK, SHUT, open_width};
use super::drag::{drag, near};
use super::{bar, panel, step, tick};
use crate::settings::Settings;
use crate::test_support::window::seated;
use crate::ui::theme;

/// **The default widths**: a quarter of the desk, never more than two fifths of the
/// window — so at the matrix's phone the column keeps three fifths — and the
/// collapsed one holds a glyph and the name elided at the column's width,
/// **and one dragged to twice that shows more of the name** (bl-773e).
#[test]
fn the_widths_are_a_quarter_of_the_desk_and_a_glyph_column() {
    let sizes = crate::snapshot::SIZES;
    let near = |a: f32, b: f32| (a - b).abs() < 0.01;
    assert!(near(sizes[2].1, DESK), "the desk is the matrix's");
    assert!(near(open_width(sizes[2].1), 350.0));
    assert!(near(open_width(sizes[1].1), 350.0));
    assert!(
        near(open_width(sizes[0].1), 160.0),
        "two fifths of the phone"
    );
    let head = format!("{} p", theme::glyph::AT_REST);
    let collapsed = |model: &mut crate::ui::Model| {
        let ctx = egui::Context::default();
        bar(&tick(&ctx, model, 0.0, None))
            .into_iter()
            .find(|run| run.text.starts_with(&head))
            .expect("the collapsed row")
    };
    let row = collapsed(&mut seated());
    assert!(row.text.ends_with(theme::glyph::ELIDED), "{:?}", row.text);
    assert!(row.text.len() < super::ROW.len(), "cut: {:?}", row.text);
    assert_eq!(row.shown, row.laid, "the cut is on the glass, not the clip");
    assert!(row.laid.max.x <= SHUT, "inside the glyph column");
    let mut wider = seated();
    wider.settings.bar_shut = Some(SHUT * 2.0);
    let more = collapsed(&mut wider);
    assert!(
        more.text.chars().count() > row.text.chars().count(),
        "dragged wider shows more: {:?} > {:?}",
        more.text,
        row.text
    );
    assert!(more.laid.max.x <= SHUT * 2.0);
}

/// **The bounds clamp, and overwrite nothing**: as a value, the expanded
/// width stands between the floor and two fifths of the window and the
/// collapsed one between the floor and the expanded one, the defaults where
/// nothing was dragged; on the glass, a drag past a bound stops at it, and a
/// window narrower than what was set shows the ceiling and keeps the number.
#[test]
fn the_bounds_clamp_a_drag_and_a_window_and_overwrite_nothing() {
    let f = 30.0;
    let wide = open_width(DESK);
    assert_eq!(shown(&Settings::default(), DESK, f), (SHUT, wide));
    let set = Settings {
        bar_shut: Some(300.0),
        bar_open: Some(500.0),
        ..Settings::default()
    };
    assert_eq!(shown(&set, DESK, f), (300.0, 500.0));
    assert_eq!(
        shown(&set, 900.0, f),
        (300.0, 360.0),
        "two fifths of the window"
    );
    assert_eq!(
        shown(&set, 500.0, f),
        (200.0, 200.0),
        "the column follows down"
    );
    assert_eq!(shown(&set, 50.0, f), (f, f), "neither under the floor");
    let narrow = Settings {
        bar_shut: Some(10.0),
        ..Settings::default()
    };
    assert_eq!(shown(&narrow, DESK, f), (f, wide));
    let ctx = egui::Context::default();
    let mut model = seated();
    tick(&ctx, &mut model, 0.0, None);
    drag(
        &ctx,
        &mut model,
        DESK,
        (1.0, 0.02),
        SHUT - HANDLE / 2.0,
        DESK,
    );
    assert!(
        near(model.settings.bar_shut, wide),
        "no wider than the expanded width"
    );
    model.bar.pinned = true;
    tick(&ctx, &mut model, 2.0, None);
    drag(
        &ctx,
        &mut model,
        DESK,
        (3.0, 0.02),
        wide - HANDLE / 2.0,
        DESK,
    );
    let cap = ceiling(DESK);
    assert!(
        near(model.settings.bar_open, cap),
        "no wider than the ceiling"
    );
    drag(
        &ctx,
        &mut model,
        DESK,
        (4.0, 0.02),
        cap - HANDLE / 2.0,
        -cap,
    );
    assert!(
        near(model.settings.bar_open, wide),
        "no narrower than the collapsed width"
    );
    for i in 0..2 {
        step(
            &ctx,
            &mut model,
            400.0,
            5.0 + f64::from(i),
            vec![egui::Event::PointerGone],
        );
    }
    assert!(
        near(Some(panel(&ctx)), ceiling(400.0)),
        "the phone's ceiling"
    );
    assert_eq!(
        (model.settings.bar_shut, model.settings.bar_open),
        (Some(wide), Some(wide)),
        "and the numbers are kept"
    );
}
