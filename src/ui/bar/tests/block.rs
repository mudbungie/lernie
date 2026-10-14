//! **A bar row is its whole block** (DESIGN §4.44 item 5; bl-232e): the
//! pointer on its preview tints the name, the age and the preview together,
//! edge to edge of the bar, and a click there selects it.

use super::{DESK, bar, tick};
use crate::paint_probe::{fills_of, frame::Window};
use crate::test_support::window::{conv, seated};
use crate::ui::theme;

#[test]
fn the_pointer_on_a_preview_tints_and_takes_the_whole_row() {
    let mut model = seated();
    model.bar.pinned = true;
    let mut other = conv("20260830T051300Z-c3d4", "the other");
    other.preview = "what it said last".to_owned();
    model.convs.push(other);
    let ctx = egui::Context::default();
    let runs = bar(&tick(&ctx, &mut model, 0.0, None));
    let find = |text: &str| {
        runs.iter()
            .find(|run| run.text == text)
            .unwrap_or_else(|| panic!("{text:?} in {runs:?}"))
            .laid
    };
    let (name, preview) = (find("✔ the other"), find("what it said last"));
    tick(&ctx, &mut model, 0.1, Some(preview.center()));
    let lit = tick(&ctx, &mut model, 0.2, Some(preview.center()));
    let block = fills_of(&lit)
        .into_iter()
        .find(|(_, fill)| *fill == theme::hover())
        .map(|(rect, _)| rect)
        .expect("the hover wash stands");
    assert!(block.contains(name.center()), "over the name");
    assert!(block.contains(preview.center()), "over the preview");
    assert!(block.left() <= name.left() && block.width() > name.width());
    let window = Window::sized(DESK, 900.0);
    let mut body = |ctx: &egui::Context| crate::ui::render(ctx, &mut model);
    window.frame(Vec::new(), &mut body);
    crate::paint_probe::frame::click(&window, preview.center(), &mut body);
    assert_eq!(model.conversation.as_deref(), Some("20260830T051300Z-c3d4"));
}
