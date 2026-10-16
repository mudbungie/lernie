//! **The clipped detector's other direction** (`super::super::clipped`), split
//! from [`super`] at the line cap: every synthetic subject that assertion
//! must flag, and the ones it must not.

use super::super::clipped;
use super::toy;
use crate::ui::Model;
use egui_kittest::HarnessBuilder;

/// **A control scrolled past the overlay's clip is not a fault**, and one
/// beside the box or with no box to scroll is judged as before.
#[test]
fn a_control_scrolled_past_the_overlay_s_clip_is_not_a_fault() {
    let clip = Some(egui::Rect::from_min_max(
        egui::pos2(16.0, 100.0),
        egui::pos2(384.0, 790.0),
    ));
    assert!(clipped::scrolled((20.0, 800.0, 200.0, 820.0), clip));
    assert!(clipped::scrolled((20.0, 10.0, 200.0, 90.0), clip));
    assert!(!clipped::scrolled((20.0, 300.0, 200.0, 320.0), clip));
    assert!(!clipped::scrolled((390.0, 800.0, 500.0, 820.0), clip));
    assert!(!clipped::scrolled((20.0, 800.0, 200.0, 820.0), None));
}

#[test]
fn a_control_the_tree_gives_no_rectangle_is_a_fault() {
    let why = clipped::fault(None, 100.0, 100.0).expect("no rectangle is a fault");
    assert!(why.contains("no rectangle"), "{why}");
}

#[test]
fn a_control_with_an_empty_rectangle_is_a_fault() {
    let why = clipped::fault(Some((10.0, 10.0, 10.0, 40.0)), 100.0, 100.0)
        .expect("an empty rectangle is a fault");
    assert!(why.contains("empty"), "{why}");
}

#[test]
fn a_control_wholly_outside_the_window_is_a_fault_on_every_edge() {
    let window = (100.0_f32, 100.0_f32);
    for bounds in [
        (-50.0, 10.0, -1.0, 40.0),
        (10.0, -50.0, 40.0, -1.0),
        (101.0, 10.0, 140.0, 40.0),
        (10.0, 101.0, 40.0, 140.0),
    ] {
        let why = clipped::fault(Some(bounds), window.0, window.1)
            .unwrap_or_else(|| panic!("{bounds:?} is outside the window"));
        assert!(why.contains("wholly outside"), "{why}");
    }
}

/// **The walk itself, over a real tree**, and not just the judgement it makes.
///
/// The matrix exercises the sound answer on every shot; a detector needs the
/// other one too, and in the real window nothing faults — which is the whole
/// point of it. So the window is declared smaller than the control it drew:
/// same tree, same walk, and a node that is now outside it.
#[test]
fn the_walk_names_the_control_it_found_outside_the_window() {
    let harness = toy("a control");
    let said = clipped::complaints("toy", 4.0, 4.0, &harness);
    assert!(
        said.concat().contains("a control"),
        "the complaint names the control: {said:?}"
    );
    assert!(said.concat().contains("wholly outside"), "{said:?}");
}

/// **A label off the window is a layout question, not this assertion's**
/// (bl-d1ae): the same shrunken window that faults a button says nothing
/// about a sentence, because egui counts selectable prose as clickable and a
/// covering pane's prose scrolls.
#[test]
fn a_label_outside_the_window_is_not_a_fault_where_a_control_is() {
    let mut harness = HarnessBuilder::<Model>::default()
        .with_size(egui::Vec2::new(200.0, 100.0))
        .build_state(
            |ctx: &egui::Context, _: &mut Model| {
                egui::CentralPanel::default().show(ctx, |ui| {
                    ui.label("a sentence");
                    let _ = crate::ui::theme::paint::button(ui, "a control", true, "a toy");
                });
            },
            Model::default(),
        );
    harness.run();
    let said = clipped::complaints("toy", 4.0, 4.0, &harness).concat();
    assert!(said.contains("a control"), "{said}");
    assert!(!said.contains("a sentence"), "{said}");
}

#[test]
fn a_control_hanging_over_an_edge_is_not_a_fault() {
    assert!(clipped::fault(Some((-10.0, -10.0, 5.0, 5.0)), 100.0, 100.0).is_none());
    assert!(clipped::fault(Some((95.0, 95.0, 140.0, 140.0)), 100.0, 100.0).is_none());
}
