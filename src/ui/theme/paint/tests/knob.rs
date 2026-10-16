//! **The slider** (`docs/STYLE.md` §5; bl-99ed): its trough is raised on the
//! glass, and the one that holds the keyboard wears the brand ring.

use super::super::slider;
use crate::paint_probe::{fills_of, frame::Window, strokes_of};
use crate::ui::theme::{BRAND, RAISED};

/// A window painting one slider on the overlay's own surface.
fn body(value: &mut f32) -> impl FnMut(&egui::Context) + '_ {
    move |ctx: &egui::Context| {
        egui::CentralPanel::default().show(ctx, |ui| {
            slider(ui, value, (0.0, 10.0, 1.0), " pt", "a knob");
        });
    }
}

/// **The trough is `RAISED`** — egui's own rail is the resting fill, which
/// is the overlay's surface and so invisible there.
#[test]
fn the_trough_is_raised() {
    let window = Window::sized(300.0, 100.0);
    let mut value = 5.0;
    let output = window.frame(Vec::new(), &mut body(&mut value));
    assert!(
        fills_of(&output).iter().any(|(_, fill)| *fill == RAISED),
        "a raised trough"
    );
}

/// **A slider rings while it holds the keyboard and not otherwise**: one
/// brand rectangle around its run after a Tab, none at rest.
#[test]
fn a_slider_rings_while_it_holds_the_keyboard() {
    let window = Window::sized(300.0, 100.0);
    let mut value = 5.0;
    let mut body = body(&mut value);
    let rings = |output: &egui::FullOutput| {
        strokes_of(output)
            .into_iter()
            .filter(|(_, ink)| *ink == BRAND)
            .count()
    };
    assert_eq!(rings(&window.frame(Vec::new(), &mut body)), 0, "at rest");
    window.frame(
        vec![crate::paint_probe::frame::press(egui::Key::Tab)],
        &mut body,
    );
    let output = window.frame(Vec::new(), &mut body);
    assert!(rings(&output) >= 1, "the ring while it holds the keyboard");
}
