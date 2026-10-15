//! The idiom (DESIGN §4.42, item 4; §4.45 items 5 and 7): a gutter glyph per
//! speaker, the working under its one line on painted stems — open while its
//! turn runs, one line once it settled, a toggle remembered — a `response`
//! line after it while it is open, and a hairline between turns. The fills
//! under the blocks are [`super::bubble`]'s, and the three stages of the
//! working are [`super::stages`]'s.

use super::super::RESPONSE;
use crate::paint_probe::frame::Window;
use crate::paint_probe::{fills_of, frame::locate_in, seen_of};
use crate::reply::transcript::{Block, Entry, EntryKind, Transcript, Usage};
use crate::test_support::window::{click, said, seated};
use crate::ui::Model;
use crate::ui::theme::{self, glyph};

pub(super) fn entry(kind: EntryKind) -> Entry {
    Entry {
        name: "003-model-a.json".to_owned(),
        raw: "{}".to_owned(),
        kind,
    }
}

/// A model turn that works: some reasoning and one call.
pub(super) fn working() -> Entry {
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
pub(super) fn prose(text: &str) -> Entry {
    entry(EntryKind::Model {
        model_id: "model-a".to_owned(),
        blocks: vec![Block::Text(text.to_owned())],
        usage: Usage::new(),
    })
}

pub(super) fn result(content: &str, is_error: bool) -> Entry {
    entry(EntryKind::ToolResult {
        tool_use_id: "tu-1".to_owned(),
        content: content.to_owned(),
        is_error,
    })
}

/// The turn still open: the engine's streaming entry riding the response,
/// with `text` said so far.
pub(super) fn running(text: &str) -> Entry {
    entry(EntryKind::Streaming {
        thinking: String::new(),
        text: text.to_owned(),
    })
}

pub(super) fn with(entries: Vec<Entry>) -> Model {
    Model {
        transcript: Transcript { entries },
        ..seated()
    }
}

pub(super) fn glass(model: &mut Model) -> egui::FullOutput {
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

/// **The working's one line, open or shut**, in the exact words it is read
/// in: the sign, then `worked` and its two counts parted by the note glyph.
pub(super) fn head(open: bool, calls: u64, notes: u64) -> String {
    format!(
        "{} worked · {} · {}",
        if open { glyph::OPEN } else { glyph::SHUT },
        crate::render::counted(calls, "call"),
        crate::render::counted(notes, "note")
    )
}

/// **A settled working stands as one line** (§4.45 item 5), counting what it
/// holds; a click opens it to its compact steps — a call with the mark of
/// how it ended and its output beneath — and the toggle is remembered on
/// the next frame.
#[test]
fn a_settled_working_stands_as_one_line_and_the_line_opens_it() {
    let mut model = with(vec![
        said("op", "port it"),
        working(),
        result("done", false),
        prose("the seam is real"),
    ]);
    let output = glass(&mut model);
    let seen = texts(&output);
    assert!(seen.iter().any(|t| *t == head(false, 1, 1)), "{seen:?}");
    assert!(
        !seen.iter().any(|t| t.contains("read(src/ui.rs)")),
        "{seen:?}"
    );
    let window = Window::sized(1400.0, 900.0);
    click(&window, &head(false, 1, 1), |ctx| {
        crate::ui::render(ctx, &mut model);
    });
    let opened = window.text(|ctx| crate::ui::render(ctx, &mut model));
    assert!(opened.contains(&head(true, 1, 1)), "{opened}");
    // One call, met with its result on one row, compact: the mark of how it
    // ended leads the line and the output stands beneath; the reasoning is a
    // step and not a call (bl-eb4e); the id never reaches the glass.
    for word in [
        &format!("{} read(src/ui.rs)", glyph::DONE),
        "done",
        "weighing two seams",
    ] {
        assert!(opened.contains(word), "{word:?}: {opened}");
    }
    assert!(!opened.contains("tu-1"), "no id: {opened}");
}

/// **A running turn's working stands open**, its call in flight whole, and
/// shutting it is remembered against that default on the frames after.
#[test]
fn a_running_working_stands_open_and_a_toggle_is_remembered() {
    let mut model = with(vec![said("op", "port it"), working(), running("")]);
    let seen = texts(&glass(&mut model));
    assert!(seen.iter().any(|t| *t == head(true, 1, 1)), "{seen:?}");
    assert!(seen.iter().any(|t| t == "read(src/ui.rs)"), "{seen:?}");
    let window = Window::sized(1400.0, 900.0);
    click(&window, &head(true, 1, 1), |ctx| {
        crate::ui::render(ctx, &mut model);
    });
    for _ in 0..2 {
        let shut = window.text(|ctx| crate::ui::render(ctx, &mut model));
        assert!(shut.contains(&head(false, 1, 1)), "{shut}");
        assert!(!shut.contains("read(src/ui.rs)"), "{shut}");
    }
}

/// **The working's line lights under the pointer** (§4.45 item 6): the
/// hover wash under it, and nothing in that wash away from it.
#[test]
fn the_working_s_line_lights_under_the_pointer() {
    let mut model = with(vec![said("op", "port it"), working()]);
    let window = Window::sized(1400.0, 900.0);
    let at = locate_in(&window, &head(false, 1, 1), |ctx| {
        crate::ui::render(ctx, &mut model);
    })
    .expect("the head is on the glass");
    let washed = |output: &egui::FullOutput| {
        fills_of(output)
            .iter()
            .any(|(_, ink)| *ink == theme::hover())
    };
    let away = window.frame(Vec::new(), |ctx| crate::ui::render(ctx, &mut model));
    assert!(!washed(&away), "bare at rest");
    let under = window.frame(vec![egui::Event::PointerMoved(at)], |ctx| {
        crate::ui::render(ctx, &mut model);
    });
    assert!(washed(&under), "lit under the pointer");
}

/// **A `response` line parts an OPEN working from the prose after it** —
/// folded, the one line is the parting — and a hairline parts one turn from
/// the next; neither stands where there is nothing to part.
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
        !texts(&parted).iter().any(|t| t == RESPONSE),
        "folded, no elbow: {:?}",
        texts(&parted)
    );
    assert!(
        hairlines(&parted) > before,
        "a second turn is parted by a hairline"
    );
    let window = Window::sized(1400.0, 900.0);
    click(&window, &head(false, 1, 1), |ctx| {
        crate::ui::render(ctx, &mut two);
    });
    let opened = window.text(|ctx| crate::ui::render(ctx, &mut two));
    assert!(opened.contains(RESPONSE), "open, the elbow: {opened}");
}

/// **A failed result wears the error accent on its mark**, and the trail's
/// stems are painted rather than typed.
#[test]
fn a_failed_step_wears_the_error_accent() {
    let mut model = with(vec![working(), result("no such file", true), running("")]);
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

/// **A call still in flight is its row and nothing more**: the call's mark
/// and head, no mark of how it ended and no body under it (bl-eb4e).
#[test]
fn a_call_in_flight_wears_no_ending() {
    let mut model = with(vec![working(), running("")]);
    let output = glass(&mut model);
    let seen = texts(&output);
    assert!(seen.iter().any(|t| t == "read(src/ui.rs)"), "{seen:?}");
    assert!(
        !seen.iter().any(|t| t == glyph::DONE || t == glyph::FAILED),
        "{seen:?}"
    );
}
