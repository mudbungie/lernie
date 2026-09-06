//! **A machine's answer on the glass** (bl-90d0): folded to a few lines, with
//! one control saying what it is hiding and how much, and the whole of it one
//! gesture away.
//!
//! `super::super::fold`'s suite asserts the projection; this asserts what
//! reaches the operator. The two are different questions — the fold could be
//! computed perfectly and painted beside the full text — and only this one can
//! say the six screens are gone.

use crate::paint_probe::frame::Window;
use crate::reply::transcript::{Entry, EntryKind, Transcript};
use crate::test_support::window::{click, numbered, seated, seen};
use crate::ui::Model;

/// How many lines the fixture's tool call answers with — the shape of the `bl
/// --help` that was six screens of the conversation pane.
const LINES: usize = 700;

/// The seated window with one tool result of `n` numbered lines.
fn spilling(n: usize) -> Model {
    let content = numbered(n);
    let mut model = seated();
    model.transcript = Transcript {
        entries: vec![Entry {
            name: "004-tool.json".to_owned(),
            raw: content.clone(),
            kind: EntryKind::ToolResult {
                tool_use_id: "toolu_1".to_owned(),
                content,
                is_error: false,
            },
        }],
    };
    model
}

/// What one settled window has on the glass, as whole runs.
fn glass(window: &Window, model: &mut Model) -> Vec<String> {
    window.frame(Vec::new(), |ctx| crate::ui::render(ctx, model));
    seen(window, |ctx| crate::ui::render(ctx, model))
        .into_iter()
        .map(|shown| shown.text)
        .collect()
}

/// The word the control wears while a row of `n` lines is folded.
fn control(n: usize) -> String {
    format!("show all {n} lines, {} bytes", numbered(n).len())
}

/// **A result long enough to be folded and short enough to read whole**, which
/// is what the toggle beat needs: at seven hundred lines the pane is anchored
/// to the tail, so unfolding carries the control itself off the top of the
/// glass and the beat could not see what it had pressed.
const FEW: usize = 12;

#[test]
fn a_long_tool_result_stands_as_a_few_lines_and_one_control_saying_what_it_hides() {
    let mut model = spilling(LINES);
    let window = Window::sized(1400.0, 900.0);
    let glyphs = glass(&window, &mut model);
    let painted = glyphs.concat();
    assert!(painted.contains("line 001"), "{glyphs:?}");
    assert!(painted.contains("line 006"), "{glyphs:?}");
    assert!(!painted.contains("line 007"), "the rest is folded away");
    assert!(!painted.contains("line 700"), "including its tail");
    assert!(
        glyphs.iter().any(|run| *run == control(LINES)),
        "the control says the act and the size: {glyphs:?}"
    );
}

#[test]
fn the_control_brings_the_whole_of_it_and_the_other_one_puts_it_back() {
    let mut model = spilling(FEW);
    let window = Window::sized(1400.0, 900.0);
    let folded = glass(&window, &mut model).concat();
    assert!(!folded.contains("line 007"), "folded to start with");
    click(&window, &control(FEW), |ctx| {
        crate::ui::render(ctx, &mut model);
    });
    let opened = glass(&window, &mut model).concat();
    assert!(
        opened.contains("line 012"),
        "the whole of it is on the glass"
    );
    assert!(
        opened.contains(super::super::fold::UNFOLD),
        "and the control names the other act now"
    );
    click(&window, super::super::fold::UNFOLD, |ctx| {
        crate::ui::render(ctx, &mut model);
    });
    let closed = glass(&window, &mut model).concat();
    assert!(!closed.contains("line 007"), "and it folds again");
}
