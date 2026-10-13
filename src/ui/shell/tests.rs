//! The layout: one column — the transcript, the rule and the prompt — and what
//! stands down under what.

use crate::paint_probe::frame::Window;
use crate::test_support::window::{painted, queued, seated};
use crate::ui::theme;

/// The rule's one run no other pane paints: the engines it counts.
const RULE: &str = "1 engine";

/// **The window is the transcript, the rule and the prompt**, and no heading
/// names a column: there is one.
#[test]
fn the_window_is_the_transcript_the_rule_and_the_prompt() {
    let mut model = seated();
    let text = painted(&mut model);
    for word in ["port it", RULE, theme::glyph::PROMPT] {
        assert!(text.contains(word), "{word:?}:\n{text}");
    }
    assert!(
        !text.lines().any(|line| line == crate::ui::roster::HEADING),
        "no column is named:\n{text}"
    );
}

/// **It is one column at every size** — the phone's shape is the same window
/// narrower, with the transcript and the prompt both on the glass.
#[test]
fn it_is_one_column_at_the_phone_s_width_too() {
    let mut model = seated();
    let text = Window::sized(400.0, 800.0).text(|ctx| crate::ui::render(ctx, &mut model));
    for word in ["port it", theme::glyph::PROMPT] {
        assert!(text.contains(word), "{word:?}:\n{text}");
    }
}

/// **The prompt and the rule stand down under an overlay**, and the
/// overlay's own name line says where you are.
#[test]
fn the_prompt_and_the_rule_stand_down_under_an_overlay() {
    let mut model = queued();
    let text = painted(&mut model);
    assert!(!text.contains(crate::ui::composer::HINT), "{text}");
    assert!(!text.contains(RULE), "{text}");
    assert!(text.contains(crate::ui::queue::HEADING), "{text}");
}

/// **Nothing live paints under the enrollment** (bl-7574): neither the
/// prompt nor the rule.
#[test]
fn nothing_live_paints_under_the_enrollment() {
    let mut model = seated();
    model.begin_enrollment();
    let text = painted(&mut model);
    assert!(!text.contains(crate::ui::composer::HINT), "{text}");
    assert!(!text.contains(RULE), "{text}");
}
