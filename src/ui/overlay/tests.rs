//! The overlay: a pane floats in the one frame, over the conversation, and
//! Escape puts it down.

use super::{HINT, stand};
use crate::test_support::window::{escape, painted, queued, seated};
use crate::ui::Model;

/// **Nothing floats over a window with nothing standing**, and the enrollment
/// is not this frame's — it keeps its whole-window cover.
#[test]
fn nothing_stands_where_nothing_is_open() {
    let mut model = seated();
    let text = painted(&mut model);
    assert!(!text.contains(HINT), "{text}");
    let mut enrolling = seated();
    enrolling.begin_enrollment();
    let text = crate::test_support::window::pane(|ui| stand(ui, &mut enrolling));
    assert!(!text.contains(HINT), "{text}");
}

/// **A standing pane floats over the conversation rather than replacing it**:
/// the transcript's own words and the pane's are on one frame, with the hint
/// under the pane.
#[test]
fn a_pane_floats_over_the_conversation_and_says_how_it_closes() {
    let mut model = queued();
    let text = painted(&mut model);
    for word in [crate::ui::queue::HEADING, HINT, "port it"] {
        assert!(text.contains(word), "{word:?}:\n{text}");
    }
}

/// **Escape puts it down**, through the window's own key handling.
#[test]
fn escape_puts_the_pane_down() {
    let mut model: Model = queued();
    let window = crate::paint_probe::frame::Window::new();
    escape(&window, |ctx| crate::ui::render(ctx, &mut model));
    assert!(!model.showing(crate::ui::Listing::Queue));
    let text = painted(&mut model);
    assert!(!text.contains(HINT), "{text}");
}
