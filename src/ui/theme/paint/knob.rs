//! **The slider** (DESIGN §4.46 item 4; `docs/STYLE.md` §5, *a slider*):
//! the control the settings pane paints a number knob with, and the one home
//! of egui's slider. The toggle beside it is [`super::control::toggle`].
//!
//! It takes the knob's help sentence beside its bounds and ends in
//! [`super::told`] (§4.46 item 2), so a knob without a sentence does not
//! compile and the instruments find its seat.

use egui::{Response, Stroke};

use super::super::{BRAND, RADIUS, RAISED, space};

/// **A slider**: `value` between `floor` and `ceiling`, moved by `step`,
/// said with `unit` after its number, and `help` under the pointer.
///
/// **Arrows move it one step while it holds the keyboard**, which is how it
/// is reached with no pointer (§4.46 item 3). egui's own arrow moves the
/// knob one POINT of the run and then rounds to the step, which on a run a
/// hundred points long with a step a tenth of the range is no move at all;
/// so the presses are read here and the value is stepped from where the
/// frame began, which also keeps egui's own move from adding to it. **A
/// press hands it the keyboard** as a click hands a button its own: egui
/// focuses a dragged run on Tab alone, and a knob the pointer just set is
/// the knob the arrows should fine-tune.
pub fn slider(
    ui: &mut egui::Ui,
    value: &mut f32,
    (floor, ceiling, step): (f32, f32, f32),
    unit: &str,
    help: &str,
) -> Response {
    let before = *value;
    // **The wash goes under the run** (DESIGN §4.45 item 6): egui's slider
    // says the pointer with its handle's fill alone, and a handle is a
    // circle the fill walk does not count — so a place in the paint list is
    // held before the slider paints and filled after, with the same wash
    // every door wears, where the slider stands.
    let under = ui.painter().add(egui::Shape::Noop);
    // **The trough is `RAISED` and the handle wears the brand** (STYLE §5,
    // *a slider*; bl-99ed): egui paints the rail in the resting control
    // fill, which on the overlay's own surface is no rail at all, and the
    // handle as a disc in the same fill with the ink's hairline. In this
    // scope the rail and the disc are raised and the disc's ring is the
    // brand in every state, and the run below the handle is the selection's
    // tint — so where the knob stands is legible at a glance.
    let mut seat = ui
        .scope(|ui| {
            let widgets = &mut ui.visuals_mut().widgets;
            widgets.inactive.bg_fill = RAISED;
            for rung in [
                &mut widgets.inactive,
                &mut widgets.hovered,
                &mut widgets.active,
            ] {
                rung.fg_stroke = Stroke::new(1.0, BRAND);
            }
            ui.add(
                egui::Slider::new(value, floor..=ceiling)
                    .step_by(f64::from(step))
                    .suffix(unit)
                    .trailing_fill(true),
            )
        })
        .inner;
    if seat.hovered() {
        let wash =
            egui::Shape::rect_filled(seat.rect.expand(space::XS), RADIUS, super::super::hover());
        ui.painter().set(under, wash);
    }
    // **The slider that holds the keyboard wears the brand ring** (bl-99ed;
    // bl-2d6b's rule for a row, bl-8e3e's for a door): the arrows move
    // whichever knob holds it, so that knob has to be visible.
    if seat.has_focus() {
        ui.painter()
            .rect_stroke(seat.rect.expand(space::XS), RADIUS, Stroke::new(1.0, BRAND));
    }
    if seat.drag_started() {
        seat.request_focus();
    }
    if seat.has_focus() {
        let presses = |key| {
            let n = ui.input(|i| i.num_presses(key));
            // A frame carries a handful of presses at most; the lint denies
            // the cast that would say so in one token.
            f32::from(u8::try_from(n).unwrap_or(u8::MAX))
        };
        let steps = presses(egui::Key::ArrowRight) - presses(egui::Key::ArrowLeft);
        if steps != 0.0 {
            *value = step.mul_add(steps, before).clamp(floor, ceiling);
            seat.mark_changed();
        }
    }
    super::told(seat, help)
}
