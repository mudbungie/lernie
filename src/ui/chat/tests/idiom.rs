//! The idiom (DESIGN §4.42, item 4): a gutter glyph per speaker, the trail
//! under one fold on painted stems, a `response` line after it, and a
//! hairline between turns.

use super::super::{RESPONSE, TRAIL};
use crate::paint_probe::frame::Window;
use crate::paint_probe::{fills_of, seen_of};
use crate::reply::transcript::{Block, Entry, EntryKind, Transcript, Usage};
use crate::test_support::window::{click, said, seated};
use crate::ui::Model;
use crate::ui::theme::{self, glyph};

fn entry(kind: EntryKind) -> Entry {
    Entry {
        name: "003-model-a.json".to_owned(),
        raw: "{}".to_owned(),
        kind,
    }
}

/// A model turn that works: some reasoning and one call.
fn working() -> Entry {
    entry(EntryKind::Model {
        model_id: "model-a".to_owned(),
        blocks: vec![
            Block::Thinking("weighing two seams".to_owned()),
            Block::ToolUse {
                id: "tu-1".to_owned(),
                name: "read".to_owned(),
                input: "src/ui.rs".to_owned(),
            },
        ],
        usage: Usage::new(),
    })
}

/// A model turn that speaks.
fn prose(text: &str) -> Entry {
    entry(EntryKind::Model {
        model_id: "model-a".to_owned(),
        blocks: vec![Block::Text(text.to_owned())],
        usage: Usage::new(),
    })
}

fn result(content: &str, is_error: bool) -> Entry {
    entry(EntryKind::ToolResult {
        tool_use_id: "tu-1".to_owned(),
        content: content.to_owned(),
        is_error,
    })
}

fn with(entries: Vec<Entry>) -> Model {
    Model {
        transcript: Transcript { entries },
        ..seated()
    }
}

fn glass(model: &mut Model) -> egui::FullOutput {
    let window = Window::sized(1400.0, 900.0);
    window.frame(Vec::new(), |ctx| crate::ui::render(ctx, model));
    window.frame(Vec::new(), |ctx| crate::ui::render(ctx, model))
}

fn texts(output: &egui::FullOutput) -> Vec<String> {
    seen_of(output).into_iter().map(|run| run.text).collect()
}

/// **The gutter says who**: the prompt's glyph in the brand before the
/// operator, the bar in weak ink before the model, the dot before a peer.
#[test]
fn the_gutter_says_who_spoke() {
    let mut model = with(vec![
        said("op", "port it"),
        said("judge-one", "seconded"),
        prose("the seam is real"),
    ]);
    let output = glass(&mut model);
    let ink = |text: &str| {
        seen_of(&output)
            .into_iter()
            .filter(|run| run.text == text)
            .map(|run| run.ink)
            .collect::<Vec<_>>()
    };
    assert!(
        ink(glyph::PROMPT).contains(&theme::BRAND),
        "{:?}",
        texts(&output)
    );
    assert!(
        ink(glyph::MODEL).contains(&theme::INK_WEAK),
        "{:?}",
        texts(&output)
    );
    assert!(
        ink(glyph::NOTE).contains(&theme::INK_WEAK),
        "{:?}",
        texts(&output)
    );
}

/// **The trail stands under one fold**, a call with its dot and a result
/// with its mark, and the fold's head shuts it.
#[test]
fn the_trail_stands_under_one_fold_and_the_head_shuts_it() {
    let mut model = with(vec![
        said("op", "port it"),
        working(),
        result("done", false),
        prose("the seam is real"),
    ]);
    let output = glass(&mut model);
    let seen = texts(&output);
    let head = format!("{} {TRAIL} (3)", glyph::OPEN);
    for word in [
        &head,
        glyph::CALL,
        glyph::DONE,
        "model-a → read tu-1",
        "tu-1 returned",
    ] {
        assert!(seen.iter().any(|t| t == word), "{word:?}: {seen:?}");
    }
    let window = Window::sized(1400.0, 900.0);
    click(&window, &head, |ctx| crate::ui::render(ctx, &mut model));
    let shut = window.text(|ctx| crate::ui::render(ctx, &mut model));
    assert!(
        shut.contains(&format!("{} {TRAIL} (3)", glyph::SHUT)),
        "{shut}"
    );
    assert!(!shut.contains("tu-1 returned"), "{shut}");
}

/// **A `response` line parts the trail from the prose after it**, and a
/// hairline parts one turn from the next — and neither stands where there is
/// nothing to part.
#[test]
fn a_response_line_and_a_hairline_part_what_follows() {
    let mut one = with(vec![said("op", "port it")]);
    let quiet = glass(&mut one);
    assert!(!texts(&quiet).iter().any(|t| t == RESPONSE));
    let hairlines = |output: &egui::FullOutput| {
        fills_of(output)
            .iter()
            .filter(|(rect, ink)| {
                rect.height() <= 1.5 && rect.width() > 300.0 && *ink == theme::HAIRLINE
            })
            .count()
    };
    let before = hairlines(&quiet);
    let mut two = with(vec![
        said("op", "port it"),
        working(),
        result("done", false),
        prose("the seam is real"),
        said("op", "and again"),
    ]);
    let parted = glass(&mut two);
    assert!(
        texts(&parted).iter().any(|t| t == RESPONSE),
        "{:?}",
        texts(&parted)
    );
    assert!(
        hairlines(&parted) > before,
        "a second turn is parted by a hairline"
    );
}

/// **A failed result wears the error accent on its mark**, and the trail's
/// stems are painted rather than typed.
#[test]
fn a_failed_step_wears_the_error_accent() {
    let mut model = with(vec![working(), result("no such file", true)]);
    let output = glass(&mut model);
    let red = theme::accent(theme::State::Error);
    assert!(
        seen_of(&output)
            .iter()
            .any(|run| run.text == glyph::FAILED && run.ink == red),
        "{:?}",
        texts(&output)
    );
}
