//! **The bar's head is a comfortable target** (DESIGN §4.44 item 1;
//! bl-bc9d): the pin and `+ new` are doors at the heading size with `S` of
//! air inside the seat, collapsed and open alike.

use super::{bar, tick};
use crate::paint_probe::fills_of;
use crate::test_support::window::seated;
use crate::ui::theme;

/// The heading's line height on `ctx`.
fn heading(ctx: &egui::Context) -> f32 {
    let font = egui::TextStyle::Heading.resolve(&ctx.style());
    ctx.fonts(|fonts| fonts.row_height(&font))
}

/// **The head's doors are at least a heading line tall, and their seats
/// hold `S` of air besides**: collapsed, the pin's glyph and the `+` are
/// laid at the heading size; open and under the pointer, the wash that
/// fills the pin's seat stands the heading line plus the air each side.
#[test]
fn the_pin_and_new_are_heading_sized_with_air_inside_the_seat() {
    let ctx = egui::Context::default();
    let mut model = seated();
    let shut = bar(&tick(&ctx, &mut model, 0.0, None));
    let tall = heading(&ctx);
    for door in [theme::glyph::PIN, super::super::PLUS] {
        let run = shut
            .iter()
            .find(|run| run.text == door)
            .unwrap_or_else(|| panic!("{door:?} in {shut:?}"));
        assert!(run.laid.height() >= tall, "{door:?}: {:?}", run.laid);
    }
    model.bar.pinned = true;
    let unpin = theme::worded(theme::glyph::PIN, super::super::UNPIN);
    let at = bar(&tick(&ctx, &mut model, 1.0, None))
        .into_iter()
        .find(|run| run.text == unpin)
        .expect("the open pin")
        .laid
        .center();
    tick(&ctx, &mut model, 1.1, Some(at));
    let lit = tick(&ctx, &mut model, 1.2, Some(at));
    let wash = fills_of(&lit)
        .into_iter()
        .find(|(rect, fill)| *fill == theme::hover() && rect.contains(at))
        .map(|(rect, _)| rect)
        .expect("the pin's seat lit");
    assert!(
        wash.height() >= tall + 2.0 * theme::space::S,
        "the seat holds the air: {wash:?} against {tall}"
    );
}
