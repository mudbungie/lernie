//! The strip on the glass: absent with nothing open, a tab per open
//! conversation with the selected one marked, the asking `?` before the
//! asking one, a click that selects, a close that closes, and the `act:` tag.

use crate::paint_probe::{self, frame::Window};
use crate::test_support::window::{click, conv, painted, seated, seen};
use crate::ui::{Model, theme};
use egui_kittest::kittest::Queryable;

const FIRST: &str = "20260830T051200Z-a1b2";
const SECOND: &str = "20260830T052000Z-b2c3";

/// The seated wall with a second conversation, asking, and both open.
fn two_open() -> Model {
    let mut model = seated();
    let mut asking = conv(SECOND, "tidy the rule");
    asking.attention = 1;
    model.convs.push(asking);
    model.select(SECOND);
    model.select(FIRST);
    model
}

/// The topmost run reading `text` — the strip's, over any later copy.
fn topmost(window: &Window, model: &mut Model, text: &str) -> paint_probe::Seen {
    seen(window, |ctx| crate::ui::render(ctx, model))
        .into_iter()
        .filter(|run| run.text == text)
        .min_by(|a, b| a.laid.min.y.total_cmp(&b.laid.min.y))
        .unwrap_or_else(|| panic!("nothing on the glass reads {text:?}"))
}

/// **With nothing open there is no strip.**
#[test]
fn no_tab_no_strip() {
    assert!(!painted(&mut seated()).contains(theme::glyph::CLOSE));
}

/// **The selected tab in body ink over a brand hairline, the rest weak, and
/// the asking `?` in the attention accent before the asking one.**
#[test]
fn the_selected_tab_is_marked_and_the_asking_one_wears_its_glyph() {
    let window = Window::new();
    let mut model = two_open();
    let first = topmost(&window, &mut model, "port the paint probe");
    let second = topmost(&window, &mut model, "tidy the rule");
    let asks = topmost(&window, &mut model, theme::glyph::ASKING);
    assert_eq!(first.ink, theme::INK);
    assert_eq!(second.ink, theme::INK_WEAK);
    assert_eq!(asks.ink, theme::accent(theme::State::Attention));
    assert!(asks.laid.max.x <= second.laid.min.x, "the ? leads its name");
    let output = window.frame(Vec::new(), |ctx| crate::ui::render(ctx, &mut model));
    assert!(
        paint_probe::fills_of(&output)
            .iter()
            .any(|(rect, fill)| *fill == theme::BRAND
                && rect.min.y >= first.laid.max.y
                && rect.center().x > first.laid.min.x
                && rect.center().x < first.laid.max.x),
        "a brand hairline under the selected tab"
    );
    let closes = seen(&window, |ctx| crate::ui::render(ctx, &mut model))
        .into_iter()
        .filter(|run| run.text == theme::glyph::CLOSE)
        .collect::<Vec<_>>();
    assert_eq!(closes.len(), 2, "a close after each name");
    assert!(closes.iter().all(|run| run.ink == theme::INK_FAINT));
}

/// **A tab whose row the seat does not hold is named by its id.**
#[test]
fn a_tab_with_no_row_wears_its_id() {
    let mut model = seated();
    model.select("unlisted-root");
    assert!(painted(&mut model).contains("unlisted-root"));
}

/// **A click on a tab selects it; a click on the close closes it.**
#[test]
fn a_click_selects_and_the_close_closes() {
    let window = Window::new();
    let mut model = two_open();
    click(&window, "tidy the rule", |ctx| {
        crate::ui::render(ctx, &mut model);
    });
    assert_eq!(model.conversation.as_deref(), Some(SECOND));
    model.close_tab(FIRST);
    click(&window, theme::glyph::CLOSE, |ctx| {
        crate::ui::render(ctx, &mut model);
    });
    assert_eq!(model.conversation, None, "the last close puts it down");
    assert!(model.strip().is_empty());
}

/// **Each tab wears the `act:` tag its select spends.**
#[test]
fn a_tab_wears_its_selects_tag() {
    let harness = crate::snapshot::seat(two_open(), 1400.0, 900.0);
    let tab = harness.get_by_label("tidy the rule");
    assert_eq!(
        tab.author_id(),
        Some(
            crate::ui::act::tokens(&[crate::verbs::TRANSCRIPT.word, crate::verbs::FOLLOW.word])
                .as_str()
        )
    );
}
