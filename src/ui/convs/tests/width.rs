//! **What the pane's own width does to the panels beside it** (bl-b3b2) — the
//! one assertion in this suite whose subject is the whole window rather than a
//! row, and the reason every line the list paints truncates.

use crate::test_support::window::{conv, seated};

/// **The pane never takes more width than it paints** (bl-b3b2) — the operator
/// sighting of a giant black box in the middle of the window, with the desktop
/// flickering through it.
///
/// A side panel paints a frame sized to its own `max_width` and then reserves
/// `inner_response.response.rect.max` from the layout. Those are two different
/// numbers the moment the pane's content is wider than the cap
/// `crate::ui::shell::widths` hands it — and the strip between them is covered
/// by **no panel at all**, because the central panel begins after the reserved
/// edge rather than after the painted one. Nothing paints that strip, so what
/// shows is the window surface's own clear: black, or the compositor behind it
/// on the frames the alpha path lets through. Paint that chose a wrong colour
/// could never show the desktop; only paint that never happened can.
///
/// The cause was the row headline, laid by a widget that cannot truncate. So
/// the assertion is the invariant rather than the widget: **at every width, the
/// full-height panel grounds tile the window edge to edge with no gap.** 700
/// points is the case the operator hit — the yield policy caps this pane at
/// 149.3 there, well under the headline's natural width — and it left an
/// 81-point hole.
#[test]
fn the_panel_grounds_tile_the_window_with_no_gap_for_the_clear_to_show_through() {
    for width in [1400.0_f32, 1000.0, 900.0, 800.0, 700.0, 600.0] {
        let mut model = seated();
        model.convs = vec![conv(
            "id",
            "a conversation with a name long enough to overflow",
        )];
        let ctx = egui::Context::default();
        let painted = ctx.run(
            crate::paint_probe::frame::screen_sized(width, 600.0),
            |ctx| crate::ui::render(ctx, &mut model),
        );
        let mut grounds: Vec<egui::Rect> = crate::paint_probe::fills_of(&painted)
            .into_iter()
            .filter(|(rect, ink)| rect.height() > 300.0 && ink.a() > 250)
            .map(|(rect, _)| rect)
            .collect();
        grounds.sort_by(|a, b| a.min.x.total_cmp(&b.min.x));
        let mut edge = 0.0_f32;
        for ground in &grounds {
            assert!(
                (ground.min.x - edge).abs() < 0.5,
                "at {width} points a strip of x{edge:.1}..{:.1} is painted by no panel: {grounds:?}",
                ground.min.x
            );
            edge = ground.max.x;
        }
        assert!(
            (edge - width).abs() < 0.5,
            "at {width} points the grounds stop at {edge:.1}: {grounds:?}"
        );
    }
}
