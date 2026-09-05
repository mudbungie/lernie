//! **The grid a decoder reads the matrix off** — the scale rule as arithmetic,
//! and the symbol read back off rendered pixels at two device scales.
//!
//! This is the one suite about this pane that is about pixels, and it is about
//! them for the reason bl-5e0e measured: a module drawn as its own filled
//! rectangle is feathered half a device pixel proud of every edge, so on the
//! old four-point pitch at one pixel per point a light module enclosed by dark
//! ones lost about 44% of its area — contrast the decoder never gets, worst
//! exactly where module pixels are scarcest. Arithmetic alone cannot see that.
//! Pixels can, so the last test here renders and reads every module back.

use super::{FLOOR, QUIET, count, module, paint, pitch, span};
use crate::qr::Symbol;
use crate::ui::Model;
use egui_kittest::HarnessBuilder;
use egui_kittest::wgpu::TestRenderer;

/// Two screen distances are the same one. **A tolerance rather than a bit-for-
/// bit compare** because clippy denies the second outright, and the margin is
/// honest here rather than a hedge: every value in this arithmetic is a small
/// integer times a whole-numbered pitch, so the difference is exactly zero and
/// anything an epsilon would forgive is a real defect.
fn same(got: f32, want: f32) {
    assert!((got - want).abs() < f32::EPSILON, "{got} is not {want}");
}

/// A small whole number as a screen distance, the way the painter converts one.
fn number(n: u32) -> f32 {
    f32::from(u16::try_from(n).expect("every number here is a small one"))
}

/// **The pitch is whole device pixels, and as many as the room buys.** A
/// fraction of a pixel is the defect: it puts a module boundary inside a pixel,
/// so neighbouring modules of one nominal size come out different widths.
#[test]
fn the_pitch_is_a_whole_number_of_device_pixels() {
    // Room for three and a half pixels a module buys three, not three and a
    // half — and the same room at two pixels a point buys seven of them, which
    // is three and a half POINTS. The rule is about the device, not the layout.
    same(pitch(161.0 * 3.5, 161, 1.0), 3.0);
    same(pitch(161.0 * 3.5, 161, 2.0), 3.5);
}

/// **The symbol is as large as the pane allows**, which is the half of the
/// defect that had no fix at all: the pitch was a constant, so the one pane
/// whose whole job is to be photographed offered no way to make its subject
/// bigger.
#[test]
fn more_room_is_a_bigger_symbol() {
    assert!(pitch(1000.0, 161, 1.0) > pitch(500.0, 161, 1.0));
    assert!(pitch(500.0, 161, 1.0) > pitch(250.0, 161, 1.0));
}

/// **One device pixel is the floor**, and a pane narrower than that gets a
/// symbol that overflows it rather than one whose modules share pixels. A
/// negative width and an unmeasured one are the same answer, so a layout that
/// has not settled cannot compose a pitch of nothing.
#[test]
fn a_pane_with_no_room_still_gets_a_module_a_pixel() {
    same(pitch(10.0, 161, 1.0), FLOOR);
    same(pitch(10.0, 161, 2.0), FLOOR / 2.0);
    same(pitch(-1.0, 161, 1.0), FLOOR);
    same(pitch(f32::NAN, 161, 1.0), FLOOR);
}

/// **Where a module goes.** The quiet zone is painted rather than assumed — a
/// decoder uses it to find the symbol's edge, and a pane's own background is
/// not it — so module (0, 0) sits four modules in on both axes.
#[test]
fn a_module_sits_inside_its_own_quiet_zone() {
    let rect = egui::Rect::from_min_size(egui::pos2(10.0, 20.0), egui::vec2(500.0, 500.0));
    let first = module(rect, 0, 0, 4.0);
    same(first.min.x, 10.0 + span(QUIET, 4.0));
    same(first.min.y, 20.0 + span(QUIET, 4.0));
    same(first.width(), 4.0);
    same(first.height(), 4.0);
    let along = module(rect, 3, 5, 4.0);
    same(along.min.x - first.min.x, span(3, 4.0));
    same(along.min.y - first.min.y, span(5, 4.0));
}

/// The conversion is exact by arithmetic rather than by hope: the widest symbol
/// there is measures 185 modules with its quiet zone, so nothing saturates.
#[test]
fn every_symbol_s_span_converts_exactly() {
    same(span(0, 4.0), 0.0);
    same(span(1, 4.0), 4.0);
    let widest = Symbol::encode(&[0; 2331]).expect("the ceiling").side() + 2 * QUIET;
    assert_eq!(widest, 185);
    same(count(widest), 185.0);
    same(span(widest, 4.0), 185.0 * 4.0);
}

/// The square the toy window gives the symbol, in points.
const ROOM: u32 = 240;

/// **The fraction of a point the symbol is pushed off the window's corner.**
///
/// It is the whole reason this suite renders at all. A pane's origin is
/// whatever the layout hands it, never a whole number by promise, and it is at
/// a fractional origin that a feathered module actually bleeds: the feather is
/// a ramp from full to nothing across one device pixel, so on an aligned grid
/// every pixel centre lands at one end of it and the bleed is invisible, while
/// half a pixel over it lands in the middle and the module's own edge comes out
/// grey. A test that photographed the aligned case would be green against the
/// defect it was written for. It is deliberately not a half point, which would
/// put a pixel centre exactly on a module boundary and make the readback a
/// question about tie-breaking.
const NUDGE: f32 = 0.3;

/// **The symbol alone on an unframed panel, nudged off the corner.** Alone,
/// because the readback below has to find the symbol and nothing else may be
/// inked; nudged, for [`NUDGE`]'s reason.
fn glass(symbol: &Symbol, per_point: f32) -> image::RgbaImage {
    let drawn = symbol.clone();
    let mut harness = HarnessBuilder::<Model>::default()
        .with_size(egui::Vec2::new(number(ROOM), number(ROOM)))
        .with_pixels_per_point(per_point)
        .build_state(
            move |ctx: &egui::Context, _: &mut Model| {
                egui::CentralPanel::default()
                    .frame(egui::Frame::none().inner_margin(NUDGE))
                    .show(ctx, |ui| paint(ui, &drawn));
            },
            Model::default(),
        );
    harness.run();
    TestRenderer::new().render(&harness)
}

/// **What this pixel reads as: the mark, the ground, or neither** — and the
/// third answer is the defect. A feathered edge is a BLEND of the two, so a
/// threshold halfway between them would file every bled pixel as whichever side
/// it fell on and see nothing. The bands are tight on purpose: a symbol drawn
/// on a whole-pixel grid with no feathering has no pixel between them at all.
///
/// **Opacity is part of the reading**, because the window behind the pane is
/// not: egui composites premultiplied, so a pixel the symbol did not cover
/// carries the ground the harness cleared to and answers *neither* — which is
/// what stops the search below from finding its corner outside the symbol.
fn reads(pixel: image::Rgba<u8>) -> Option<bool> {
    if pixel.0[3] != u8::MAX {
        return None;
    }
    let ink = pixel.0.iter().take(3).all(|channel| *channel <= 8);
    let paper = pixel.0.iter().take(3).all(|channel| *channel >= 247);
    ink.then_some(true).or(paper.then_some(false))
}

/// The pitch as a whole number of device pixels — **and the assertion that it
/// is one**, which is the scale rule itself. Recovered by search rather than by
/// a cast, the house lint set denying the lossy one.
fn whole(pitch: f32, per_point: f32) -> u32 {
    let wide = pitch * per_point;
    (2..=64)
        .find(|n| (number(*n) - wide).abs() < f32::EPSILON)
        .unwrap_or_else(|| panic!("{wide} device pixels a module is not a whole number"))
}

/// **Where module (0, 0) starts, found on the glass rather than computed.**
///
/// The symbol's top-left finder pattern is dark and the quiet zone around it is
/// not, so the first inked pixel on each axis is that module's own first pixel.
/// Asking the image rather than repeating the painter's arithmetic is the
/// point: a grid the test derived would agree with the painter whatever the two
/// of them did.
fn corner(image: &image::RgbaImage) -> (u32, u32) {
    let lit: Vec<(u32, u32)> = image
        .enumerate_pixels()
        .filter(|(_, _, pixel)| reads(**pixel) == Some(true))
        .map(|(x, y, _)| (x, y))
        .collect();
    let least = |axis: fn(&(u32, u32)) -> u32| {
        lit.iter()
            .map(axis)
            .min()
            .expect("the symbol put ink on the glass")
    };
    (least(|at| at.0), least(|at| at.1))
}

/// **Every module reads back off the glass, at one device pixel per point and
/// at two** (bl-5e0e).
///
/// This is the assertion a decoder makes and the one the four-point constant
/// failed: every pixel of every module carries that module's own value, so a
/// light module enclosed by dark ones is light all the way to its edges and a
/// dark one is dark all the way to its own. A symbol painted as one feathered
/// rectangle per module cannot pass it at a fractional origin, which is the
/// only kind a layout offers.
#[test]
fn every_module_reads_back_off_the_glass_at_both_device_scales() {
    let symbol = Symbol::encode(b"a short one").expect("it fits a symbol");
    let across = symbol.side() + 2 * QUIET;
    for scale in [1_u32, 2] {
        let per_point = number(scale);
        let room = number(ROOM) - 2.0 * NUDGE;
        let wide = whole(pitch(room, across, per_point), per_point);
        let image = glass(&symbol, per_point);
        let (x0, y0) = corner(&image);
        for y in 0..symbol.side() {
            for x in 0..symbol.side() {
                let want = symbol.dark(x, y);
                let at = |n: usize, from: u32| {
                    from + u32::try_from(n).expect("a symbol has few modules") * wide
                };
                for down in 0..wide {
                    for along in 0..wide {
                        let (px, py) = (at(x, x0) + along, at(y, y0) + down);
                        let got = *image.get_pixel(px, py);
                        assert_eq!(
                            reads(got),
                            Some(want),
                            "module ({x}, {y}) at {scale}x reads {got:?} at ({px}, {py})"
                        );
                    }
                }
            }
        }
    }
}
