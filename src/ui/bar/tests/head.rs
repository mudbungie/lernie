//! **The bar's head is a comfortable target** (DESIGN §4.44 item 1;
//! bl-bc9d): the pin is a door at the heading size with `S` of air inside
//! the seat, collapsed and open alike; `+ new` is the first row under it
//! (§4.48 item 5), a whole row's seat.

use super::{ROW, bar, said, tick};
use crate::paint_probe::fills_of;
use crate::paint_probe::frame::Window;
use crate::test_support::window::seated;
use crate::ui::theme;

/// The heading's line height on `ctx`.
fn heading(ctx: &egui::Context) -> f32 {
    let font = egui::TextStyle::Heading.resolve(&ctx.style());
    ctx.fonts(|fonts| fonts.row_height(&font))
}

/// **The pin is at least a heading line tall, and its seat holds `S` of
/// air besides**: collapsed, its glyph is laid at the heading size, and `+`
/// stands under it; open and under the pointer, the wash that fills the
/// pin's seat stands the heading line plus the air each side.
#[test]
fn the_pin_is_heading_sized_with_air_inside_the_seat() {
    let ctx = egui::Context::default();
    let mut model = seated();
    let shut = bar(&tick(&ctx, &mut model, 0.0, None));
    let tall = heading(&ctx);
    let pin = shut
        .iter()
        .find(|run| run.text == theme::glyph::PIN)
        .unwrap_or_else(|| panic!("the pin in {shut:?}"));
    assert!(pin.laid.height() >= tall, "{:?}", pin.laid);
    let plus = shut
        .iter()
        .find(|run| run.text == super::super::PLUS)
        .unwrap_or_else(|| panic!("+ in {shut:?}"));
    assert!(plus.laid.min.y >= pin.laid.max.y, "+ stands under the pin");
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

/// **Pinned, it stands open whatever the pointer does**; the pin is one
/// toggle wearing the act it would do, and the collapsed head shows its
/// glyph alone.
#[test]
fn the_pin_holds_it_open_and_says_what_it_would_do() {
    let window = Window::new();
    let mut model = seated();
    let mut body = |ctx: &egui::Context| crate::ui::render(ctx, &mut model);
    let shut = window.text(&mut body);
    assert!(shut.lines().any(|line| line == theme::glyph::PIN), "{shut}");
    crate::test_support::window::click(&window, theme::glyph::PIN, &mut body);
    assert!(model.bar.pinned);
    let ctx = egui::Context::default();
    let pinned = said(&tick(&ctx, &mut model, 0.0, None));
    let unpin = theme::worded(theme::glyph::PIN, super::super::UNPIN);
    assert!(pinned.contains(&unpin), "{pinned:?}");
    assert!(pinned.contains(&ROW.to_owned()), "{pinned:?}");
    model.bar.pinned = false;
    model.bar.left_at = Some(0.0);
    let open = said(&tick(&ctx, &mut model, 0.5, None));
    let pin = theme::worded(theme::glyph::PIN, super::super::PIN);
    assert!(open.contains(&pin), "{open:?}");
}
