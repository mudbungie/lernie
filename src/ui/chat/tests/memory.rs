//! **A toggle's memory is one conversation's** (DESIGN §4.45 item 5;
//! bl-b83d): what the operator opened in one conversation is open there on
//! every frame after and on a return to it, and says nothing about the same
//! row of another conversation, which arrives in its stage. The scroll
//! area's salt names only its own scroll state, so the memory's key is the
//! conversation's id pushed over everything under it (`super::super::render`).

use super::idiom::{prose, result, with, working};
use super::line::head;
use crate::paint_probe::frame::Window;
use crate::paint_probe::seen_of;
use crate::test_support::window::{click, numbered, said};
use crate::ui::Model;
use crate::ui::theme::glyph;

/// What one settled frame put on the glass, as whole runs.
fn runs(window: &Window, model: &mut Model) -> Vec<String> {
    let output = window.frame(Vec::new(), |ctx| crate::ui::render(ctx, model));
    seen_of(&output).into_iter().map(|run| run.text).collect()
}

/// Whether the one call stands whole: a line of its output past the preview.
fn whole(seen: &[String]) -> bool {
    seen.iter().any(|t| t.contains("line 003"))
}

/// **Opening a step in one conversation leaves the same row of another in
/// its stage, and the opened one is open again on the way back.** One panel
/// paints both; the transcript is the same shape, so only the key tells
/// them apart.
#[test]
fn a_toggle_is_remembered_per_conversation() {
    let mut model = with(vec![
        said("op", "port it"),
        working(),
        result(&numbered(12), false),
        prose("the seam is real"),
    ]);
    let first = model.conversation.clone();
    let window = Window::sized(1400.0, 900.0);
    runs(&window, &mut model);
    click(&window, &head(false, [1, 1, 0, 0]), |ctx| {
        crate::ui::render(ctx, &mut model);
    });
    let call = format!("{} read(src/ui.rs)", glyph::DONE);
    click(&window, &call, |ctx| {
        crate::ui::render(ctx, &mut model);
    });
    assert!(whole(&runs(&window, &mut model)), "opened in the first");
    // The second conversation: its working shut, and opened, its step compact.
    model.conversation = Some("20260830T051200Z-b2c3".to_owned());
    let second = runs(&window, &mut model);
    assert!(second.contains(&head(false, [1, 1, 0, 0])), "{second:?}");
    click(&window, &head(false, [1, 1, 0, 0]), |ctx| {
        crate::ui::render(ctx, &mut model);
    });
    let compact = runs(&window, &mut model);
    assert!(compact.contains(&call), "{compact:?}");
    assert!(!whole(&compact), "in its stage: {compact:?}");
    // Back to the first: as the operator left it.
    model.conversation = first;
    let back = runs(&window, &mut model);
    assert!(back.contains(&head(true, [1, 1, 0, 0])), "{back:?}");
    assert!(whole(&back), "still open: {back:?}");
}
