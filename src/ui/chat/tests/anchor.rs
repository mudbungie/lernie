//! **Where the pane opens, and what it follows** (bl-83ae).
//!
//! The transcript used to open on message 001 and stay there: a conversation
//! streaming its eleventh step painted its second, unmoved, with the answer
//! thirty screens below the fold. The three beats here are the ordinary
//! chat-pane contract, read off the glass rather than off the scroll state —
//! open on the tail, ride an append while at the tail, and stop riding it the
//! moment the reader scrolls away.

use crate::paint_probe::frame::Window;
use crate::reply::transcript::Transcript;
use crate::test_support::window::{said, seated, seen};
use crate::ui::Model;

/// A transcript of `n` numbered lines, each its own delivered entry — long
/// enough that no window in this suite can hold it.
fn lines(range: std::ops::RangeInclusive<usize>) -> Transcript {
    Transcript {
        entries: range
            .map(|at| said("op", &format!("line {at:03}")))
            .collect(),
    }
}

/// **What one settled window has ON THE GLASS.**
///
/// The frames before the read are not decoration: egui lays a bottom panel out
/// on the pass after the one that measured it, and it spends a scroll offset
/// on the pass after the one that computed it — so a window read on its first
/// pass is a window mid-layout, anchored to a viewport the composer has not
/// taken its height out of yet.
fn glass(window: &Window, model: &mut Model) -> Vec<String> {
    for _ in 0..3 {
        window.frame(Vec::new(), |ctx| crate::ui::render(ctx, model));
    }
    on_the_glass(window, model)
}

/// The read itself, with nothing settled first — what the pane shows right now.
fn on_the_glass(window: &Window, model: &mut Model) -> Vec<String> {
    seen(window, |ctx| crate::ui::render(ctx, model))
        .into_iter()
        .map(|shown| shown.text)
        .collect()
}

/// Whether a run reading exactly `line` reached the glass.
fn shows(glyphs: &[String], line: &str) -> bool {
    glyphs.iter().any(|text| text == line)
}

/// A window the broad shape fits in, small enough that sixty rows cannot.
fn desk() -> Window {
    Window::sized(900.0, 700.0)
}

#[test]
fn a_selected_conversation_opens_on_its_tail_and_not_on_message_001() {
    let mut model = seated();
    model.transcript = lines(1..=60);
    let glyphs = glass(&desk(), &mut model);
    assert!(shows(&glyphs, "line 060"), "{glyphs:?}");
    assert!(!shows(&glyphs, "line 001"), "{glyphs:?}");
}

#[test]
fn a_conversation_that_grows_while_the_reader_is_at_the_tail_keeps_the_tail() {
    let window = desk();
    let mut model = seated();
    model.transcript = lines(1..=60);
    glass(&window, &mut model);
    model.transcript = lines(1..=75);
    let glyphs = glass(&window, &mut model);
    assert!(shows(&glyphs, "line 075"), "{glyphs:?}");
}

/// **Scrolling away releases the follow, and it stays released while the
/// conversation grows under it.** A pane that dragged the glass back to the
/// tail on every append would be unreadable in exactly the case the tail
/// anchor exists for.
#[test]
fn scrolling_up_releases_the_follow_and_an_append_does_not_take_it_back() {
    let window = desk();
    let mut model = seated();
    model.transcript = lines(1..=60);
    glass(&window, &mut model);
    wheel(&window, &mut model, 4000.0);
    let glyphs = on_the_glass(&window, &mut model);
    assert!(shows(&glyphs, "line 001"), "{glyphs:?}");
    assert!(!shows(&glyphs, "line 060"), "{glyphs:?}");
    model.transcript = lines(1..=75);
    let glyphs = glass(&window, &mut model);
    assert!(!shows(&glyphs, "line 075"), "{glyphs:?}");
}

/// Roll the wheel over the conversation pane and let egui's smoothing settle.
fn wheel(window: &Window, model: &mut Model, up: f32) {
    let over = egui::pos2(700.0, 300.0);
    window.frame(vec![egui::Event::PointerMoved(over)], |ctx| {
        crate::ui::render(ctx, model);
    });
    window.frame(
        vec![egui::Event::MouseWheel {
            unit: egui::MouseWheelUnit::Point,
            delta: egui::vec2(0.0, up),
            modifiers: egui::Modifiers::NONE,
        }],
        |ctx| crate::ui::render(ctx, model),
    );
    for _ in 0..12 {
        window.frame(Vec::new(), |ctx| crate::ui::render(ctx, model));
    }
}
