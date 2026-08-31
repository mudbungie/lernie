//! **The enrollment pane, after the engine answers**: the picture, the
//! forgetting, and where a module goes.
//!
//! **Nothing here asserts a pixel.** A QR symbol drawn at the wrong scale
//! carries the same bytes and a symbol with one wrong module does not, so the
//! thing that is right or wrong is the matrix — which `crate::qr`'s own suite
//! pins against an independent encoder — and what is left for this pane is
//! *where* a module goes and *what words* stand around it.

use super::super::{CLOSE, KEPT, MODULE, QUIET, SEND, module, points};
use super::fixtures::{file, material, opened};
use crate::paint_probe::frame::Window;
use crate::qr::Symbol;
use crate::reply::enrolled::Enrolled;
use crate::test_support::window::{click, painted, seated};
use crate::ui::Shown;

/// **The answer becomes a picture, and the picture is the envelope.** The
/// matrix is what `crate::qr` pins; what this asserts is that the pane drew
/// *these* bytes and not some other reading of them.
#[test]
fn the_material_is_drawn_as_the_envelope_and_nothing_else() {
    let held = material();
    let shown = Shown::of(&held).expect("the measured envelope fits one symbol");
    let straight = Symbol::encode(held.envelope().as_bytes()).expect("it fits");
    assert_eq!(shown.symbol, straight);
    assert_eq!(shown.caption, held.caption());
}

/// **The symbol and its words reach the glass**, and the material does not.
#[test]
fn the_shown_pane_paints_the_caption_the_warning_and_no_material() {
    let mut model = opened();
    file(&mut model);
    let glass = painted(&mut model);
    assert!(glass.contains("phone-1"), "{glass}");
    assert!(glass.contains(KEPT), "{glass}");
    for secret in ["notreal-key", "notreal-leaf", "BEGIN"] {
        assert!(
            !glass.contains(secret),
            "{secret} is on the glass:\n{glass}"
        );
    }
    assert!(
        !glass.contains(SEND),
        "the form is still offering to mint:\n{glass}"
    );
}

/// **Closing it forgets the material**, which is the whole product of that
/// control: after it there is no copy of that key on this box.
#[test]
fn closing_the_pane_drops_the_material() {
    let mut model = opened();
    file(&mut model);
    assert!(model.enroll.as_ref().is_some_and(|e| e.shown.is_some()));
    model.close_enrollment();
    assert_eq!(model.enroll, None);
}

/// **Material that arrives for a closed pane is dropped and said so.** The
/// enrollment did happen on the engine, and an operator who is not told would
/// go looking for a box that has a registration and no material. A silent drop
/// is the one thing this model's door does not do.
#[test]
fn material_arriving_for_a_closed_pane_is_reported_rather_than_dropped() {
    let mut model = seated();
    file(&mut model);
    assert!(model.enroll.is_none());
    let said = model
        .notice
        .as_ref()
        .map(crate::ui::Notice::line)
        .unwrap_or_default();
    assert!(said.contains("phone-1"), "{said}");
    assert!(
        !said.contains("BEGIN"),
        "the notice quoted the material: {said}"
    );
}

/// Material too big for any symbol: an anchor chain nothing would mint, which
/// is what the refusal exists for. REMOTE §8.4 measures the real envelope at
/// about 1567 bytes against a 2331-byte ceiling, so this is a recipe that moved
/// rather than a picture that could have been drawn smaller.
fn oversized() -> Enrolled {
    Enrolled {
        ca: "notreal".repeat(400),
        ..material()
    }
}

/// **A picture that will not fit is said, and the material is dropped with the
/// pane.** Holding a secret nobody can act on is the one outcome worse than
/// refusing.
#[test]
fn material_too_big_for_a_symbol_closes_the_pane_and_says_why() {
    assert!(Shown::of(&oversized()).is_err());
    let mut model = opened();
    model.absorb(
        &crate::ui::Channel::default(),
        crate::reply::Read::Answer(crate::reply::Reply::Enrolled(oversized())),
    );
    assert_eq!(model.enroll, None, "the material outlived the refusal");
    let said = model
        .notice
        .as_ref()
        .map(crate::ui::Notice::line)
        .unwrap_or_default();
    assert!(said.contains("will not fit"), "{said}");
    assert!(!said.contains("notreal"), "the notice quoted it: {said}");
}

/// And the shown pane's close is the one that forgets the material.
#[test]
fn the_shown_pane_s_close_forgets_the_material() {
    let mut model = opened();
    file(&mut model);
    let window = Window::new();
    click(&window, CLOSE, |ctx| crate::ui::render(ctx, &mut model));
    assert_eq!(model.enroll, None);
}

/// Two screen distances are the same one. **A tolerance rather than a bit-for-
/// bit compare** because clippy denies the second outright, and the margin is
/// honest here rather than a hedge: every value in this arithmetic is a small
/// integer times a whole-numbered constant, so the difference is exactly zero
/// and anything an epsilon would forgive is a real defect.
fn same(got: f32, want: f32) {
    assert!((got - want).abs() < f32::EPSILON, "{got} is not {want}");
}

/// **Where a module goes.** The quiet zone is painted rather than assumed — a
/// decoder uses it to find the symbol's edge, and a pane's own background is
/// not it — so module (0, 0) sits four modules in on both axes.
#[test]
fn a_module_sits_inside_its_own_quiet_zone() {
    let rect = egui::Rect::from_min_size(egui::pos2(10.0, 20.0), egui::vec2(500.0, 500.0));
    let first = module(rect, 0, 0);
    same(first.min.x, 10.0 + points(QUIET));
    same(first.min.y, 20.0 + points(QUIET));
    same(first.width(), MODULE);
    same(first.height(), MODULE);
    let along = module(rect, 3, 5);
    same(along.min.x - first.min.x, points(3));
    same(along.min.y - first.min.y, points(5));
}

/// The conversion is exact by arithmetic rather than by hope: the widest symbol
/// there is measures 185 modules with its quiet zone, so nothing saturates.
#[test]
fn every_symbol_s_span_converts_exactly() {
    same(points(0), 0.0);
    same(points(1), MODULE);
    let widest = Symbol::encode(&[0; 2331]).expect("the ceiling").side() + 2 * QUIET;
    assert_eq!(widest, 185);
    same(points(widest), 185.0 * MODULE);
}
