//! **The mark at the column's top right** (DESIGN §4.44 item 2; bl-036a):
//! standing at every width with and without tabs, the tabs left of it, a
//! door to the settings pane, told with its chord, washed under the pointer
//! and one Tab stop.

use super::{FIRST, two_open};
use crate::paint_probe::{fills_of, frame::Window};
use crate::test_support::window::seated;
use crate::ui::theme::paint::MARK;
use crate::ui::{Model, keys, tabs, theme};
use egui_kittest::kittest::Queryable;

/// Where the mark's seat stands in the tree, on a seat of `w`×`h`.
fn bounds(model: Model, w: f32, h: f32) -> (f64, f64, f64, f64) {
    let harness = crate::snapshot::seat(model, w, h);
    let node = harness.get_by_label(MARK);
    let at = node.bounding_box().expect("the mark has a seat");
    (at.x0, at.y0, at.x1, at.y1)
}

/// **At every size, with the strip empty and with tabs, the mark stands in
/// the column's top right corner and every tab stands left of it.**
#[test]
fn the_mark_stands_top_right_at_every_width_and_the_tabs_stand_left_of_it() {
    for (size, w, h) in crate::snapshot::SIZES {
        for (world, model) in [("empty", seated()), ("tabbed", two_open())] {
            let at = format!("{world} at {size}");
            let (x0, y0, x1, y1) = bounds(model.clone(), w, h);
            assert!(
                (x1 - x0 - f64::from(theme::ROW)).abs() < 1.0,
                "{at}: ROW wide"
            );
            assert!(
                (y1 - y0 - f64::from(theme::ROW)).abs() < 1.0,
                "{at}: ROW tall"
            );
            assert!(
                x1 > f64::from(w) - 2.0 * f64::from(theme::space::XL),
                "{at}: at the right"
            );
            assert!(y1 < 2.0 * f64::from(theme::ROW), "{at}: at the top");
            if world == "tabbed" {
                let harness = crate::snapshot::seat(model, w, h);
                let tab = harness.get_by_label("tidy the rule");
                let tab = tab.bounding_box().expect("a tab has a seat");
                assert!(tab.x1 <= x0, "{at}: the tab stands left of the mark");
            }
        }
    }
}

/// **A click on the mark opens the settings pane**, as the chord does.
#[test]
fn a_click_on_the_mark_opens_the_settings() {
    let mut harness = crate::snapshot::seat(seated(), 1400.0, 900.0);
    harness.get_by_label(MARK).click();
    harness.run();
    assert_eq!(harness.state().lookup, Some(crate::ui::Lookup::Settings));
    assert!(harness.state().covered());
}

/// **The mark's help cites the chord**, and the strip's tab is still the
/// strip's: opening the settings by the mark keeps the selection.
#[test]
fn the_mark_is_told_with_its_chord_and_keeps_the_selection() {
    assert!(
        keys::cited(tabs::MARK_HELP, &[keys::Act::Settings]).contains("Ctrl+,"),
        "{}",
        keys::cited(tabs::MARK_HELP, &[keys::Act::Settings])
    );
    let mut harness = crate::snapshot::seat(two_open(), 1400.0, 900.0);
    harness.get_by_label(MARK).click();
    harness.run();
    assert_eq!(harness.state().conversation.as_deref(), Some(FIRST));
}

/// **The mark lifts under the pointer and rings while it holds the keyboard**
/// — the door's own dressing, which is what makes it a door.
#[test]
fn the_mark_washes_under_the_pointer_and_rings_with_the_keyboard() {
    let (w, h) = (1400.0, 900.0);
    let (x0, y0, x1, y1) = bounds(seated(), w, h);
    let centre = egui::pos2(
        f32::midpoint(whole(x0), whole(x1)),
        f32::midpoint(whole(y0), whole(y1)),
    );
    let window = Window::sized(w, h);
    let mut model = seated();
    let mut body = |ctx: &egui::Context| crate::ui::render(ctx, &mut model);
    let rested = window.frame(Vec::new(), &mut body);
    assert!(
        !fills_of(&rested)
            .iter()
            .any(|(_, fill)| *fill == theme::hover()),
        "bare at rest"
    );
    let under = window.frame(vec![egui::Event::PointerMoved(centre)], &mut body);
    assert!(
        fills_of(&under)
            .iter()
            .any(|(rect, fill)| *fill == theme::hover() && rect.contains(centre)),
        "the wash under the pointer"
    );
    // The seat egui holds for the mark, read back off the bridge every
    // told control records itself on.
    let harness = crate::snapshot::seat(seated(), w, h);
    let node = harness.get_by_label(MARK).id().0;
    let seat = crate::ui::act::seat_of(&harness.ctx, node).expect("the mark is bridged");
    window.focus(seat);
    let ringed = window.frame(Vec::new(), &mut body);
    assert_eq!(window.focused(), Some(seat), "one Tab stop");
    assert!(
        crate::paint_probe::strokes_of(&ringed)
            .iter()
            .any(|(rect, ink)| *ink == theme::BRAND && rect.contains(centre)),
        "the brand ring"
    );
}

/// The whole point at or before `v`, found rather than cast.
fn whole(v: f64) -> f32 {
    f32::from(
        (0..=4096_u16)
            .rev()
            .find(|&n| f64::from(n) <= v)
            .unwrap_or_default(),
    )
}
