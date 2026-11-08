//! The rails on a real window: laid docked and not floating, the pane whole
//! on each, the allowance and the room, the two together under the ceiling,
//! and nothing under the enrollment.

use super::{COLUMN, FLOOR, allowance, clip, room};
use crate::settings::Mode;
use crate::test_support::window::{configured, recorded, trailing};
use crate::ui::{Model, Rail};
use egui_kittest::kittest::Queryable;

/// A rendered window at `width` in the advanced mode over `model`.
fn docked(model: Model, width: f32) -> egui_kittest::Harness<'static, Model> {
    let mut model = model;
    model.settings.mode = Mode::Advanced;
    crate::snapshot::seat(model, width, 700.0)
}

/// The rail's box, if the rail was laid.
fn laid(harness: &egui_kittest::Harness<'_, Model>, rail: Rail) -> Option<egui::Rect> {
    harness
        .ctx
        .data(|d| d.get_temp::<egui::Rect>(egui::Id::new(clip(rail))))
}

/// **Docked, the trail is a rail beside the box; floating, it is the
/// overlay and the box stands down** — the same pane, one query.
#[test]
fn the_trail_is_a_rail_docked_and_an_overlay_floating() {
    let harness = docked(trailing(), 1400.0);
    let rail = laid(&harness, Rail::Branch).expect("the branch rail was laid");
    assert!(harness.state().railed(Rail::Branch));
    assert!(rail.right() <= 1400.0 && rail.left() > 1400.0 * 3.0 / 5.0);
    assert!(
        harness.query_by_label(crate::ui::trail::ACK).is_some(),
        "the pane's control is on the rail"
    );
    assert!(
        harness
            .query_by_label(crate::ui::theme::paint::SEND)
            .is_some(),
        "the box stands beside a rail"
    );
    let floating = crate::snapshot::seat(trailing(), 1400.0, 700.0);
    assert!(laid(&floating, Rail::Branch).is_none());
    assert!(
        floating
            .query_by_label(crate::ui::theme::paint::SEND)
            .is_none()
    );
}

/// **The context rail holds the records pane or the config pane**, the
/// config under the rail's own name line.
#[test]
fn the_context_rail_holds_the_records_or_the_config_pane() {
    let harness = docked(recorded(), 1400.0);
    assert!(laid(&harness, Rail::Context).is_some());
    assert!(laid(&harness, Rail::Branch).is_none());
    assert!(
        harness
            .query_by_label(crate::ui::records::HEADING)
            .is_some()
    );
    let harness = docked(configured(), 1400.0);
    assert!(laid(&harness, Rail::Context).is_some());
    assert!(harness.query_by_label(crate::ui::config::WORD).is_some());
    assert!(
        harness
            .query_by_label(crate::ui::settings::HEADING)
            .is_none(),
        "no settings breadcrumb: the pane is the rail's"
    );
}

/// **Both rails together stay under the allowance**, each at least its
/// floor, the context at the edge and the branch beside the column.
#[test]
fn two_rails_share_the_allowance_with_the_context_at_the_edge() {
    let mut model = trailing();
    model.begin_records();
    let harness = docked(model, 1400.0);
    let branch = laid(&harness, Rail::Branch).expect("branch");
    let context = laid(&harness, Rail::Context).expect("context");
    assert!(context.left() >= branch.right(), "{branch:?} {context:?}");
    assert!(branch.width() >= FLOOR - 2.0 * crate::ui::theme::space::S);
    let together = context.right() - branch.left();
    assert!(together <= allowance(1400.0, crate::settings::BAR_SHUT) + 1.0);
}

/// **The allowance and the room**: two fifths, never into the column's
/// floor; the matrix's phone size has no room and its narrow size has it.
#[test]
fn the_allowance_is_the_bars_ceiling_short_of_the_columns_floor() {
    assert!((allowance(1400.0, 80.0) - 560.0).abs() < f32::EPSILON);
    assert!((allowance(900.0, 80.0) - 360.0).abs() < f32::EPSILON);
    assert!((allowance(900.0, 420.0) - (900.0 - 420.0 - COLUMN)).abs() < f32::EPSILON);
    assert!(allowance(400.0, 80.0) <= 0.0);
    let phone = crate::snapshot::seat(trailing(), 400.0, 800.0);
    assert!(!room(&phone.ctx, phone.state()));
    let narrow = docked(trailing(), 900.0);
    assert!(room(&narrow.ctx, narrow.state()));
    assert!(narrow.state().docked());
    assert!(laid(&narrow, Rail::Branch).is_some());
    // **A pinned bar counts open**: at the narrow size that leaves no room,
    // and the pane floats.
    let mut pinned = trailing();
    pinned.bar.pinned = true;
    let pinned = docked(pinned, 900.0);
    assert!(!pinned.state().docked());
    assert!(laid(&pinned, Rail::Branch).is_none());
    assert!(
        pinned
            .query_by_label(crate::ui::theme::paint::SEND)
            .is_none()
    );
}

/// Nothing is laid under the enrollment.
#[test]
fn nothing_is_laid_under_the_enrollment() {
    let mut model = trailing();
    model.begin_enrollment();
    let harness = docked(model, 1400.0);
    assert!(laid(&harness, Rail::Branch).is_none());
}
