//! The layout: one column — the transcript, the prompt directly under it, the
//! dock and the rule under the prompt — and what stands down under what.

use crate::paint_probe::frame::Window;
use crate::test_support::window::{painted, queued, seated};
use crate::ui::theme;

/// The rule's one run no other pane paints: the provider selector's door.
const RULE: &str = crate::ui::status::selectors::NO_PROVIDER;

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

/// **The rule is the lowest painted line and the box stands directly under
/// the transcript, at every size the matrix renders** (DESIGN §4.45 item 3):
/// between the transcript and the box stands the ask while the conversation
/// asks and nothing else; between the box and the rule the dock while
/// anything runs and nothing else.
#[test]
fn the_rule_is_lowest_and_the_box_stands_directly_under_the_transcript() {
    let mut docked = seated();
    docked.convs[0].state = crate::reply::convs::AgentState::InFlight;
    let worlds = [
        ("seated", seated()),
        ("docked", docked),
        ("asked", crate::test_support::window::asked()),
    ];
    for (world, model) in &worlds {
        for (size, w, h) in crate::snapshot::SIZES {
            let at = format!("{world} at {size}");
            let (prompt, rule, above, under) = shape(&at, model.clone(), w, h);
            assert!(prompt < rule, "{at}: the box is above the rule");
            match *world {
                "docked" => {
                    assert!(above.is_empty(), "{at}: {above:?} stands over the box");
                    assert!(
                        !under.is_empty() && under.iter().all(|t| t.contains("running")),
                        "{at}: the dock alone is under the box: {under:?}"
                    );
                }
                "asked" => {
                    assert!(
                        above.iter().any(|t| t == crate::ui::ask::HEADING),
                        "{at}: the ask stands over the box: {above:?}"
                    );
                    assert!(under.is_empty(), "{at}: {under:?} stands under the box");
                }
                _ => {
                    assert!(above.is_empty(), "{at}: {above:?} stands over the box");
                    assert!(under.is_empty(), "{at}: {under:?} stands under the box");
                }
            }
        }
    }
}

/// **The column's shape on two settled frames**: the box's top and the
/// rule's top, what text stands between the transcript's last row and the
/// box, and what stands between the box and the rule — asserting on the way
/// that the rule is the lowest line of the column and the transcript is
/// above the box.
fn shape(
    at: &str,
    mut model: crate::ui::Model,
    w: f32,
    h: f32,
) -> (f32, f32, Vec<String>, Vec<String>) {
    let window = Window::sized(w, h);
    window.frame(Vec::new(), |ctx| crate::ui::render(ctx, &mut model));
    let seen = crate::test_support::window::seen(&window, |ctx| crate::ui::render(ctx, &mut model));
    let lowest = |runs: &mut dyn Iterator<Item = &crate::paint_probe::Seen>| {
        runs.max_by(|a, b| a.laid.min.y.total_cmp(&b.laid.min.y))
            .map(|run| run.laid)
    };
    let rule = lowest(
        &mut seen
            .iter()
            .filter(|run| run.text == crate::ui::status::LEAD),
    )
    .unwrap_or_else(|| panic!("{at}: the rule"));
    // Its left lead is the leftmost lead on that line: the column's edge.
    let left = seen
        .iter()
        .filter(|run| run.text == crate::ui::status::LEAD && run.laid.min.y >= rule.min.y - 1.0)
        .map(|run| run.laid.min.x)
        .fold(rule.min.x, f32::min);
    // The column is every run right of the bar, whose foot shares the bottom.
    let column: Vec<_> = seen
        .iter()
        .filter(|run| run.laid.min.x >= left - 1.0)
        .collect();
    let floor = column.iter().map(|run| run.laid.max.y).fold(0.0, f32::max);
    assert!(
        floor <= rule.max.y + 1.0,
        "{at}: the rule is the lowest line"
    );
    let prompt = lowest(
        &mut column
            .iter()
            .copied()
            .filter(|run| run.text == theme::glyph::PROMPT && run.ink == theme::BRAND),
    )
    .unwrap_or_else(|| panic!("{at}: the box"));
    let words = column
        .iter()
        .find(|run| run.text == "port it")
        .unwrap_or_else(|| panic!("{at}: the transcript"))
        .laid;
    assert!(
        words.max.y <= prompt.min.y,
        "{at}: the transcript is above the box"
    );
    let between = |top: f32, bottom: f32| {
        column
            .iter()
            .filter(|run| run.laid.min.y >= top - 1.0 && run.laid.max.y <= bottom + 1.0)
            .map(|run| run.text.clone())
            .collect::<Vec<_>>()
    };
    (
        prompt.min.y,
        rule.min.y,
        between(words.max.y, prompt.min.y),
        between(prompt.max.y, rule.min.y),
    )
}
