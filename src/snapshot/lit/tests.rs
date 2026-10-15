//! **The lit assertion's other direction**: a window painting one tagged
//! control that is a bare label with a click sense must be complained
//! about, and one painting a button must not — a detector that has never
//! flagged anything is a detector nobody has evidence about.

use super::{complaints, selected, takes_text};
use crate::ui::Model;
use egui_kittest::HarnessBuilder;

/// A window painting one tagged control: bare, or egui's own button.
fn toy(bare: bool) -> egui_kittest::Harness<'static, Model> {
    let mut harness = HarnessBuilder::<Model>::default()
        .with_size(egui::Vec2::new(200.0, 100.0))
        .build_state(
            move |ctx: &egui::Context, _: &mut Model| {
                crate::ui::theme::install(ctx);
                egui::CentralPanel::default().show(ctx, |ui| {
                    let seat = if bare {
                        ui.add(egui::Label::new("a bare seat").sense(egui::Sense::click()))
                    } else {
                        ui.button("a lit seat")
                    };
                    crate::ui::act::tag(&seat, &["message"]);
                });
            },
            Model::default(),
        );
    harness.run();
    harness
}

#[test]
fn a_tagged_label_that_senses_a_click_and_paints_nothing_is_a_complaint() {
    let mut harness = toy(true);
    let (judged, said) = complaints("toy", &mut harness);
    assert_eq!(judged, 1, "{said:?}");
    assert_eq!(said.len(), 1, "{said:?}");
    assert!(said[0].contains("a bare seat"), "{said:?}");
    assert!(said[0].contains("act:message"), "{said:?}");
    assert!(said[0].contains("does not light"), "{said:?}");
}

#[test]
fn a_button_lights_and_is_not() {
    let mut harness = toy(false);
    let (judged, said) = complaints("toy", &mut harness);
    assert_eq!(judged, 1, "{said:?}");
    assert!(said.is_empty(), "{said:?}");
}

#[test]
fn the_chosen_row_is_toggled_and_a_checked_box_is_not_chosen() {
    assert!(selected("Button", "Some(True)"));
    assert!(!selected("Button", "Some(False)"));
    assert!(!selected("Button", "None"));
    assert!(!selected("CheckBox", "Some(True)"));
}

#[test]
fn a_field_is_known_by_the_tree_s_name_for_it() {
    assert!(takes_text("TextInput"));
    assert!(takes_text("MultilineTextInput"));
    assert!(!takes_text("Button"));
}
