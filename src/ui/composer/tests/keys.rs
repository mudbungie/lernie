//! **What the keyboard does to the composer** (bl-f251): Enter sends and
//! Shift+Enter breaks a line, and every control on it is in the Tab order
//! with no binding of its own.
//!
//! Split from [`super`] at the line cap on the seam the two files already
//! have: that one reads what a gesture composes, this one reads which key
//! makes the gesture.

use super::super::{HINT, render};
use crate::paint_probe::frame::Window;
use crate::test_support::window::seated;
use serde_json::json;

/// **Enter sends and Shift+Enter breaks a line** (bl-f251), and the hint says
/// so. A field three lines tall is one an operator expects Enter to break a
/// line in; the one binding that sends is read beside the act, and the shifted
/// key is the field's own.
#[test]
fn enter_sends_what_was_typed_and_shift_enter_breaks_a_line() {
    let mut model = seated();
    let window = Window::new();
    let mut body = |ctx: &egui::Context| {
        egui::CentralPanel::default().show(ctx, |ui| render(ui, &mut model));
    };
    let at = crate::paint_probe::frame::locate_in(&window, HINT, &mut body).expect("the hint");
    crate::paint_probe::frame::click(&window, at, &mut body);
    window.frame(vec![egui::Event::Text("ship".to_owned())], &mut body);
    window.frame(
        vec![egui::Event::Key {
            key: egui::Key::Enter,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers: egui::Modifiers::SHIFT,
        }],
        &mut body,
    );
    window.frame(vec![egui::Event::Text(" it".to_owned())], &mut body);
    window.frame(
        vec![crate::paint_probe::frame::press(egui::Key::Enter)],
        &mut body,
    );
    window.frame(Vec::new(), &mut body);
    assert_eq!(
        model.outbox,
        vec![crate::ui::Posted::act(
            json!({"op": "message", "workspace": "home",
                    "agent": "20260830T051200Z-a1b2", "content": "ship\n it"})
        )]
    );
    assert_eq!(model.draft, "", "sent, so no longer a draft");
}

/// **The box is in the Tab order**, and it is the one control the composer
/// has (§4.42): Enter on it is the deposit, proven above, and there is
/// nothing beside it for a Space to fire.
#[test]
fn tab_reaches_the_box() {
    let mut model = seated();
    let window = Window::new();
    let mut reached = Vec::new();
    {
        let mut body = |ctx: &egui::Context| {
            egui::CentralPanel::default().show(ctx, |ui| render(ui, &mut model));
        };
        window.frame(Vec::new(), &mut body);
        for _ in 0..3 {
            window.frame(
                vec![crate::paint_probe::frame::press(egui::Key::Tab)],
                &mut body,
            );
            reached.push(window.focused());
        }
    }
    assert!(
        reached.contains(&Some(egui::Id::new(crate::ui::keys::BOX_ID))),
        "the box is in the tab order: {reached:?}"
    );
}
