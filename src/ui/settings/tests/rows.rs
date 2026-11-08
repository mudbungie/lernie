//! The rows: a check and the pin write their fields from the control at the
//! row's right end, a slider moves on the arrows, a lying knob paints
//! nothing, the search finds and jumps, and the tree is a select at the
//! matrix size.

use super::open;
use crate::paint_probe::frame::Window;
use crate::settings::Settings;
use crate::test_support::window::pane;
use crate::ui::settings::rows::PINNED;
use crate::ui::settings::search::{NOTHING, OPEN, hits};
use crate::ui::settings::tree;
use crate::ui::{Model, Page};
use egui_kittest::kittest::Queryable;

/// Where the control at the right end of the row labelled `label` stands:
/// the row's line, at the overlay's right margin.
fn control_of(window: &Window, model: &mut Model, label: &str) -> egui::Pos2 {
    let at = crate::paint_probe::frame::locate_in(window, label, |ctx| {
        crate::ui::render(ctx, model);
    })
    .expect("the label is on the glass");
    let inside = window
        .ctx()
        .data(|d| d.get_temp::<egui::Rect>(egui::Id::new(crate::ui::overlay::BOX)))
        .expect("the box");
    egui::pos2(inside.right() - crate::ui::theme::space::L, at.y)
}

/// **A check writes its field and the pin's row writes the bar's**: one
/// fact, two doors — each from the control at the row's right end.
#[test]
fn a_check_at_the_rows_right_end_writes_the_field_and_the_pin_the_bar() {
    let window = Window::new();
    let mut model = open(Page::Seat(crate::settings::Page::Look));
    window.frame(Vec::new(), |ctx| crate::ui::render(ctx, &mut model));
    let at = control_of(&window, &mut model, "bubbles");
    crate::paint_probe::frame::click(&window, at, |ctx| crate::ui::render(ctx, &mut model));
    assert!(
        !model.settings.on(crate::settings::Toggle::Bubbles),
        "the check flipped the field"
    );
    let mut model = open(Page::Seat(crate::settings::Page::Bar));
    window.frame(Vec::new(), |ctx| crate::ui::render(ctx, &mut model));
    let at = control_of(&window, &mut model, PINNED);
    crate::paint_probe::frame::click(&window, at, |ctx| crate::ui::render(ctx, &mut model));
    assert!(model.bar.pinned, "the pin's row is the bar's own fact");
}

/// **A slider writes its field through the row's setter**, and the arrow
/// keys move the one that holds the keyboard (DESIGN §4.46 item 3).
#[test]
fn a_slider_moves_on_the_arrows_and_writes_its_field() {
    let window = Window::new();
    let mut model = open(Page::Seat(crate::settings::Page::Input));
    window.frame(Vec::new(), |ctx| crate::ui::render(ctx, &mut model));
    let shown = format!("{} pt", crate::settings::WHEEL);
    let value = crate::paint_probe::frame::locate_in(&window, &shown, |ctx| {
        crate::ui::render(ctx, &mut model);
    })
    .expect("the wheel's number is on the glass");
    let trough = egui::pos2(value.x - 60.0, value.y);
    crate::paint_probe::frame::click(&window, trough, |ctx| crate::ui::render(ctx, &mut model));
    let pressed = model.settings.wheel;
    assert!(
        (pressed - crate::settings::WHEEL).abs() > f32::EPSILON,
        "{pressed}"
    );
    let right = egui::Event::Key {
        key: egui::Key::ArrowRight,
        physical_key: None,
        pressed: true,
        repeat: false,
        modifiers: egui::Modifiers::NONE,
    };
    window.frame(vec![right], |ctx| crate::ui::render(ctx, &mut model));
    assert!(
        (model.settings.wheel - pressed).abs() > f32::EPSILON,
        "{pressed} -> {}",
        model.settings.wheel
    );
}

/// **A choice's seat writes its word** (DESIGN §4.48 item 8): the mode page
/// paints a seat per mode over its sentence, and a click on the other one
/// sets it.
#[test]
fn a_choice_s_seat_writes_its_word() {
    let window = Window::new();
    let mut model = open(Page::Seat(crate::settings::Page::Mode));
    window.frame(Vec::new(), |ctx| crate::ui::render(ctx, &mut model));
    let painted = crate::paint_probe::text_of(
        &window.frame(Vec::new(), |ctx| crate::ui::render(ctx, &mut model)),
    );
    for word in [
        crate::settings::Mode::Simple.says(),
        crate::settings::Mode::Advanced.says(),
        "changes what is shown, not what the seat can do",
    ] {
        assert!(painted.contains(word), "{word:?}:\n{painted}");
    }
    crate::test_support::window::click(&window, "advanced", |ctx| {
        crate::ui::render(ctx, &mut model);
    });
    assert_eq!(model.settings.mode, crate::settings::Mode::Advanced);
    crate::test_support::window::click(&window, "simple", |ctx| {
        crate::ui::render(ctx, &mut model);
    });
    assert_eq!(model.settings.mode, crate::settings::Mode::Simple);
}

/// **A row whose kind and value disagree paints nothing**: the table is
/// judged before any page paints it (`crate::settings::tests`), so the arm
/// exists to refuse a control that lies rather than to be reached.
#[test]
fn a_row_whose_kind_and_value_disagree_paints_nothing() {
    let lying = crate::settings::Knob {
        key: "lying",
        page: crate::settings::Page::Look,
        label: "a knob that lies",
        help: "its kind says a toggle and its value is a number",
        kind: crate::settings::Kind::Toggle,
        get: |_| crate::settings::Knobbed::At(1.0),
        set: |_, _| {},
    };
    let mut settings = Settings::default();
    let mut jump = None;
    let painted = pane(|ui| crate::ui::settings::rows::knob(ui, &mut settings, &mut jump, &lying));
    assert!(painted.trim().is_empty(), "{painted}");
}

/// **The search reads the knobs, the pin and the pages by label and help**,
/// lower-cased, and says so when nothing matches.
#[test]
fn the_search_finds_rows_across_every_page() {
    let found = hits("wheel");
    assert_eq!(found.len(), 2, "{found:?}");
    assert_eq!(found[0].key, Some("wheel"));
    assert_eq!(
        found[1].page,
        Page::Seat(crate::settings::Page::Input),
        "the page by its help"
    );
    let found = hits("pinned");
    assert_eq!(found.len(), 1);
    assert_eq!(found[0].label, PINNED);
    assert!(hits("roles").iter().any(|hit| hit.page == Page::Roles));
    assert!(hits("nothing says this").is_empty());
}

/// **Typing in the box stands the hits where the page was, and picking one
/// turns the overlay to its page and jumps to the row** — the jump spent on
/// the frame the row paints.
#[test]
fn a_hit_turns_the_overlay_to_its_page_and_jumps_to_the_row() {
    let mut model = open(Page::Seat(crate::settings::Page::Look));
    "bar linger".clone_into(&mut model.seeking.text);
    let mut harness = crate::snapshot::seat(model, 1400.0, 900.0);
    assert!(harness.query_all_by_label("bar linger").next().is_some());
    assert!(
        harness.query_all_by_label("scale").next().is_none(),
        "the page's rows stand down under the hits"
    );
    harness.get_by_label(OPEN).click();
    harness.run();
    assert_eq!(
        harness.state().page(),
        Some(Page::Seat(crate::settings::Page::Bar))
    );
    assert!(harness.state().seeking.text.is_empty());
    assert_eq!(
        harness.state().seeking.jump,
        None,
        "spent on the row's frame"
    );
    "no such row".clone_into(&mut harness.state_mut().seeking.text);
    harness.run();
    assert!(harness.query_all_by_label(NOTHING).next().is_some());
}

/// **At the matrix size the tree is a select above the page and the rows
/// stack**: the path that stands on the select, every page's path in its
/// list, a pick turning the overlay — and a knob page's rows all on the
/// glass under the select, each control under its sentence.
#[test]
fn at_the_matrix_size_the_tree_is_a_select_and_the_rows_stack() {
    let look = Page::default();
    let mut harness = crate::snapshot::seat(open(look), 400.0, 700.0);
    assert!(
        harness
            .query_all_by_label(Page::Roles.label())
            .next()
            .is_none(),
        "no column"
    );
    for knob in crate::settings::KNOBS
        .iter()
        .filter(|knob| Page::Seat(knob.page) == look)
    {
        assert!(
            harness.query_all_by_label(knob.label).next().is_some(),
            "{} stacked on the page",
            knob.key
        );
    }
    harness.get_by_label(&tree::path(look)).click();
    harness.run();
    harness.get_by_label(&tree::path(Page::Clients)).click();
    harness.run();
    assert_eq!(harness.state().page(), Some(Page::Clients));
}
