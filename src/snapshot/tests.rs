//! **The matrix, and the three assertions that gate.**
//!
//! Two tests do the standing work. One renders every (world, size) in the
//! matrix, writes the PNG an agent looks at, and judges the frame it wrote —
//! for content the layout claimed and the glass does not carry, and for
//! controls laid out off the window. The other walks to the seat's one covered
//! pane and back at every size.
//!
//! Everything else here is the detectors' **other direction**: a synthetic
//! subject each one must flag. A detector is not shown to work by a green
//! suite — a detector that matches nothing is green forever, which is the way
//! this kind of check actually dies.

use super::{SIZES, blank, clipped, name, reach, seat, shots, worlds};
use crate::test_support::window::seated;
use crate::ui::Model;
use egui_kittest::HarnessBuilder;
use egui_kittest::wgpu::TestRenderer;
use image::{Rgba, RgbaImage};

/// **The matrix**: every named world at every size, written out and judged.
///
/// The image the assertions read is the one that was WRITTEN, not a second
/// render of the same frame — so the evidence an agent opens is the evidence
/// the gate judged.
#[test]
fn the_matrix_renders_every_world_at_every_size_and_the_frame_holds() {
    let renderer = TestRenderer::new();
    let into = shots();
    std::fs::create_dir_all(&into).expect("the shot directory is creatable");
    let mut complaints = Vec::new();
    for world in worlds::all() {
        for (size, width, height) in SIZES {
            let harness = seat(world.model.clone(), width, height);
            let image = renderer.render(&harness);
            let file = into.join(name(world.name, size));
            image.save(&file).expect("the shot is writable");
            let at = format!("{} at {size}", world.name);
            println!(
                "{at}: {}x{} -> {} ({:?})",
                image.width(),
                image.height(),
                file.display(),
                crate::ui::shell::shape(width)
            );
            complaints.extend(blank::complaints(&at, &image, &harness));
            complaints.extend(clipped::complaints(&at, width, height, &harness));
        }
    }
    assert!(
        complaints.is_empty(),
        "the rendered window is not sound:\n{}",
        complaints.join("\n")
    );
}

/// **Assertion (a)**, at every size the matrix renders.
///
/// The main screen is the seated one: an unprovisioned window is aimed at no
/// wall, so it offers no enrollment to reach and says so in words instead.
///
/// **The walk is the shape's** (bl-dfda): the narrow shape puts one column on
/// the glass, so reaching a pane there costs the gesture that goes to its
/// column as well as the two that open and close it.
#[test]
fn the_covered_pane_is_one_gesture_away_at_every_size() {
    let mut complaints = Vec::new();
    for (size, width, height) in SIZES {
        let mut harness = seat(seated(), width, height);
        complaints.extend(reach::complaints(
            &format!("seated at {size}"),
            &mut harness,
            &reach::walk(crate::ui::shell::shape(width)),
        ));
    }
    assert!(
        complaints.is_empty(),
        "the pane behind a control is not behind it any more:\n{}",
        complaints.join("\n")
    );
}

/// A window painting one control and nothing else, for the walk's own suite.
fn toy(label: &'static str) -> egui_kittest::Harness<'static, Model> {
    let mut harness = HarnessBuilder::<Model>::default()
        .with_size(egui::Vec2::new(200.0, 100.0))
        .build_state(
            move |ctx: &egui::Context, _: &mut Model| {
                egui::CentralPanel::default().show(ctx, |ui| {
                    let _ = ui.button(label);
                });
            },
            Model::default(),
        );
    harness.run();
    harness
}

#[test]
fn a_walk_whose_first_control_is_not_there_says_so() {
    let mut harness = toy("a control");
    let walk = [reach::Step {
        gesture: "not on this window",
        then: "never asked",
    }];
    let said = reach::complaints("toy", &mut harness, &walk);
    assert_eq!(said.len(), 1, "{said:?}");
    assert!(said.concat().contains("cannot be reached"), "{said:?}");
}

#[test]
fn a_walk_whose_gesture_brings_nothing_says_so() {
    let mut harness = toy("a control");
    let walk = [reach::Step {
        gesture: "a control",
        then: "a pane that never opens",
    }];
    let said = reach::complaints("toy", &mut harness, &walk);
    assert_eq!(said.len(), 1, "{said:?}");
    assert!(said.concat().contains("did not bring"), "{said:?}");
}

/// A window painting one legible label, and — when asked for — a slab over it.
///
/// This is the defect the detector exists for, reproduced: the label is laid
/// out and correct, and something is drawn on top of it afterwards.
fn covered(slab: bool) -> egui_kittest::Harness<'static, Model> {
    let mut harness = HarnessBuilder::<Model>::default()
        .with_size(egui::Vec2::new(240.0, 120.0))
        .build_state(
            move |ctx: &egui::Context, _: &mut Model| {
                egui::CentralPanel::default().show(ctx, |ui| {
                    ui.label(LEGIBLE);
                    if slab {
                        ui.painter()
                            .rect_filled(ctx.screen_rect(), 0.0, egui::Color32::BLACK);
                    }
                });
            },
            Model::default(),
        );
    harness.run();
    harness
}

/// The one word the covering fixture puts on the glass.
const LEGIBLE: &str = "this has to stay legible";

#[test]
fn a_label_that_reached_the_glass_is_quiet_and_one_painted_over_is_not() {
    let renderer = TestRenderer::new();
    let clear = covered(false);
    let said = blank::complaints("clear", &renderer.render(&clear), &clear);
    assert!(
        said.is_empty(),
        "a legible label is not a complaint: {said:?}"
    );

    let slabbed = covered(true);
    let said = blank::complaints("slabbed", &renderer.render(&slabbed), &slabbed);
    assert!(
        said.concat().contains(LEGIBLE),
        "a label under a slab is the whole point: {said:?}"
    );
    assert!(said.concat().contains("the glass is blank"), "{said:?}");
}

#[test]
fn a_rectangle_off_the_image_or_too_small_to_read_is_not_judged() {
    assert_eq!(visible_at((10.0, 10.0, 60.0, 40.0)), Some((10, 10, 60, 40)));
    // clamped to the image rather than dropped
    assert_eq!(visible_at((-20.0, -20.0, 60.0, 40.0)), Some((0, 0, 60, 40)));
    // wholly off it, and so nothing to read
    assert_eq!(visible_at((-90.0, 10.0, -20.0, 40.0)), None);
    // a hairline, which is still by construction
    assert_eq!(visible_at((10.0, 10.0, 60.0, 12.0)), None);
}

/// [`blank::visible`] against a fixed 100x100 image.
fn visible_at(bounds: (f64, f64, f64, f64)) -> Option<(u32, u32, u32, u32)> {
    blank::visible(bounds, 100, 100)
}

#[test]
fn flatness_is_measured_over_the_rectangle_and_not_the_image() {
    let mut image = RgbaImage::from_pixel(100, 100, Rgba([20, 20, 20, u8::MAX]));
    image.put_pixel(50, 50, Rgba([200, 200, 200, u8::MAX]));
    assert!(blank::flat(&image, (0, 0, 40, 40)), "quiet ground is flat");
    assert!(
        !blank::flat(&image, (40, 40, 60, 60)),
        "one lit pixel is not"
    );
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

#[test]
fn a_control_hanging_over_an_edge_is_not_a_fault() {
    assert!(clipped::fault(Some((-10.0, -10.0, 5.0, 5.0)), 100.0, 100.0).is_none());
    assert!(clipped::fault(Some((95.0, 95.0, 140.0, 140.0)), 100.0, 100.0).is_none());
}
