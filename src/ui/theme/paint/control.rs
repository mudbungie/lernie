//! **The controls that are egui's own, painted once** (DESIGN §4.46 item 2):
//! the button and the seat. `rules/no-bare-control.yml` confines
//! `egui::Button` and `selectable_label` to this directory, so a pane that
//! wants one reaches for these — and each takes the control's help sentence
//! beside its word, which is the whole of why they exist: a control painted
//! here cannot arrive without saying what it does.

use egui::Response;

/// **A button**: its word, whether it is live — a disabled control stays on
/// the glass in faint ink saying what would fill it (STYLE §5) — and its
/// help, told through [`super::told`].
pub fn button(ui: &mut egui::Ui, word: &str, enabled: bool, help: &str) -> Response {
    let seat = ui.add_enabled(enabled, egui::Button::new(word));
    super::told(seat, help)
}

/// **A seat among several**: one choice of a closed set, `chosen` wearing
/// the open fill — the former `selectable_label`, which it still is beneath,
/// so a box is drawn exactly where one was.
pub fn seat(ui: &mut egui::Ui, chosen: bool, word: &str, help: &str) -> Response {
    let seat = ui.selectable_label(chosen, word);
    super::told(seat, help)
}

/// **A toggle**: a check box in the control's fill with the brand check,
/// its word beside it, and its help — the one home of `egui::Checkbox`.
pub fn toggle(ui: &mut egui::Ui, on: &mut bool, word: &str, help: &str) -> Response {
    let seat = ui.checkbox(on, word);
    super::told(seat, help)
}

#[cfg(test)]
mod tests {
    use crate::paint_probe::frame::Window;
    use crate::test_support::window::click;

    /// Paint the three controls, answering which of them took a click.
    fn pressed(label: &str) -> (bool, bool, bool, bool) {
        let window = Window::sized(300.0, 100.0);
        let mut took = (false, false, false, false);
        click(&window, label, |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                took.0 |= super::button(ui, "go", true, "goes").clicked();
                took.1 |= super::button(ui, "dead", false, "would go").clicked();
                took.2 |= super::seat(ui, false, "one", "picks one").clicked();
                let mut on = false;
                took.3 |= super::toggle(ui, &mut on, "lane", "toggles").clicked();
            });
        });
        took
    }

    /// A live button and a seat take a click; a disabled button does not.
    #[test]
    fn a_button_and_a_seat_take_a_click_and_a_dead_button_does_not() {
        assert_eq!(pressed("go"), (true, false, false, false));
        assert_eq!(pressed("dead"), (false, false, false, false));
        assert_eq!(pressed("one"), (false, false, true, false));
        assert_eq!(pressed("lane"), (false, false, false, true));
    }
}
