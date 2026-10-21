//! **The working's one line** (DESIGN §4.45 item 5; bl-db54): what a
//! settled turn hid, counted by kind with the kinds it holds none of left
//! out, on a door the width of the bubble that lights under the pointer
//! across it and opens the steps from anywhere on it.

use super::idiom::{entry, prose, result, running, with, working};
use crate::paint_probe::frame::Window;
use crate::paint_probe::{fills_of, seen_of};
use crate::reply::transcript::EntryKind;
use crate::test_support::window::said;
use crate::ui::chat::{rows, working::counted};
use crate::ui::theme::{self, glyph};

/// **The working's one line, open or shut**, in the exact words it is read
/// in: the sign, then the counts of `[tool calls, thoughts, intermediate
/// responses, notes]` it hides, a kind with none left out, parted by the note
/// glyph.
pub(super) fn head(open: bool, counts: [u64; 4]) -> String {
    let nouns = ["tool call", "thought", "intermediate response", "note"];
    let said: Vec<String> = counts
        .into_iter()
        .zip(nouns)
        .filter(|(n, _)| *n > 0)
        .map(|(n, noun)| crate::render::counted(n, noun))
        .collect();
    format!(
        "{} {}",
        if open { glyph::OPEN } else { glyph::SHUT },
        said.join(" · ")
    )
}

/// **The line counts what it hides, by kind** (bl-db54): calls are tool
/// calls, answered or not, and a result whose call is not on the read is the
/// end of a call still; the reasoning is a thought; the model's prose between
/// is an intermediate response; a peer's words are a note — and a kind the
/// working holds none of is left out.
#[test]
fn the_line_counts_each_kind_and_leaves_out_the_ones_it_has_none_of() {
    let transcript = with(vec![
        prose("first I will read"),
        working(),
        result("done", false),
        entry(EntryKind::ToolResult {
            tool_use_id: "tu-9".to_owned(),
            content: "orphaned".to_owned(),
            is_error: false,
        }),
        said("judge-one", "seconded"),
        running("so far"),
    ])
    .transcript;
    let shown = rows(&transcript, None);
    assert_eq!(
        counted(&shown),
        "2 tool calls · 1 thought · 2 intermediate responses · 1 note",
        "{shown:?}"
    );
    assert_eq!(counted(&shown[1..3]), "1 tool call · 1 thought");
    assert_eq!(counted(&shown[..1]), "1 intermediate response");
    assert_eq!(counted(&shown[2..3]), "1 tool call");
}

/// **The working's line lights under the pointer across the bubble's width**
/// (§4.45 item 6; bl-db54): the row idiom's wash, far wider than its words,
/// none away from it, and a click at its far end — nowhere near the words —
/// opens it.
#[test]
fn the_working_s_line_lights_and_opens_across_its_width() {
    let mut model = with(vec![said("op", "port it"), working()]);
    let window = Window::sized(1400.0, 900.0);
    let words = seen_of(&window.frame(Vec::new(), |ctx| crate::ui::render(ctx, &mut model)))
        .into_iter()
        .find(|run| run.text == head(false, [1, 1, 0, 0]))
        .expect("the head is on the glass")
        .laid;
    let wash = |output: &egui::FullOutput| {
        fills_of(output)
            .into_iter()
            .find(|(_, ink)| *ink == theme::hover())
            .map(|(rect, _)| rect)
    };
    let away = window.frame(Vec::new(), |ctx| crate::ui::render(ctx, &mut model));
    assert_eq!(wash(&away), None, "bare at rest");
    let under = window.frame(vec![egui::Event::PointerMoved(words.center())], |ctx| {
        crate::ui::render(ctx, &mut model);
    });
    let lit = wash(&under).expect("lit under the pointer");
    assert!(lit.width() > 4.0 * words.width(), "{lit:?} vs {words:?}");
    let far = egui::pos2(lit.max.x - 2.0, words.center().y);
    let there = window.frame(vec![egui::Event::PointerMoved(far)], |ctx| {
        crate::ui::render(ctx, &mut model);
    });
    assert!(wash(&there).is_some(), "lit at its far end");
    crate::paint_probe::frame::click(&window, far, |ctx| crate::ui::render(ctx, &mut model));
    let opened = window.text(|ctx| crate::ui::render(ctx, &mut model));
    assert!(opened.contains(&head(true, [1, 1, 0, 0])), "{opened}");
}
