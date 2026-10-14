//! **The bar's edge eases** (DESIGN §4.44 item 1; bl-c72e): between the
//! glyph column and the open width in [`EASE`], on egui's input clock.

use super::super::{EASE, LINGER, SHUT, open_width};
use super::{DESK, tick};
use crate::test_support::window::seated;

/// **The width the bar's panel stood at** on the last frame.
fn panel(ctx: &egui::Context) -> f32 {
    egui::containers::panel::PanelState::load(ctx, egui::Id::new("bar"))
        .expect("the bar stood")
        .rect
        .width()
}

/// **The edge eases between its two widths in [`EASE`]** (bl-c72e): the
/// frame the pointer enters it is still near the glyph column, mid-way it is
/// strictly between the two, and once the ease has run it is exactly open —
/// and the same road back once the linger lapses.
#[test]
fn the_width_eases_between_its_two_ends() {
    let ctx = egui::Context::default();
    let mut model = seated();
    let wide = open_width(DESK);
    let over = Some(egui::pos2(10.0, 300.0));
    tick(&ctx, &mut model, 0.0, None);
    assert!((panel(&ctx) - SHUT).abs() < 0.01, "shut at rest");
    tick(&ctx, &mut model, 1.0, over);
    let mid = {
        tick(&ctx, &mut model, 1.0 + f64::from(EASE) / 2.0, over);
        panel(&ctx)
    };
    assert!(SHUT < mid && mid < wide, "mid-way: {mid}");
    tick(&ctx, &mut model, 1.0 + f64::from(EASE), over);
    tick(&ctx, &mut model, 1.1 + f64::from(EASE), over);
    assert!((panel(&ctx) - wide).abs() < 0.01, "open once eased");
    let away = Some(egui::pos2(900.0, 300.0));
    let gone = 1.2 + LINGER + f64::from(EASE);
    tick(&ctx, &mut model, gone, away);
    tick(&ctx, &mut model, gone + f64::from(EASE) / 2.0, away);
    let back = panel(&ctx);
    assert!(SHUT < back && back < wide, "on the way back: {back}");
    tick(&ctx, &mut model, gone + f64::from(EASE), away);
    tick(&ctx, &mut model, gone + 1.0, away);
    assert!((panel(&ctx) - SHUT).abs() < 0.01, "shut once eased");
}
