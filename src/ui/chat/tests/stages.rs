//! **The working's three stages, read off the glass** (DESIGN §4.45 item 5):
//! a step in flight whole, a finished step compact to a line or two elided
//! at the width, a settled turn's working one counted line — and every
//! compact shape a door that opens what it hides in place.

use super::idiom::{entry, glass, head, prose, result, running, with, working};
use crate::paint_probe::frame::Window;
use crate::paint_probe::frame::locate_in;
use crate::paint_probe::{fills_of, seen_of};
use crate::reply::transcript::EntryKind;
use crate::test_support::window::{click, numbered, said};
use crate::ui::Model;
use crate::ui::chat::{rows, working::counted};
use crate::ui::theme::{self, glyph};

/// What one settled frame put on the glass, as whole runs.
fn runs(window: &Window, model: &mut Model) -> Vec<String> {
    let output = window.frame(Vec::new(), |ctx| crate::ui::render(ctx, model));
    seen_of(&output).into_iter().map(|run| run.text).collect()
}

/// The compact line of the fixture's one call, and the compact line of its
/// intermediate prose.
fn call_line() -> String {
    format!("{} read(src/ui.rs)", glyph::DONE)
}
fn prose_line() -> String {
    format!("{} model-a {} first I will read", glyph::MODEL, glyph::NOTE)
}

/// **Settled, the working is one line and no step's text is on the glass**;
/// a click opens it to compact steps — the call's line with ONE line of its
/// output beneath, the reasoning and the intermediate prose one line each —
/// and a second click opens a step whole, where its name shuts it again.
#[test]
fn a_settled_working_opens_to_compact_steps_and_a_step_opens_whole() {
    let mut model = with(vec![
        said("op", "port it"),
        prose("first I will read"),
        working(),
        result(&numbered(12), false),
        prose("the seam is real"),
    ]);
    let window = Window::sized(1400.0, 900.0);
    let shut = runs(&window, &mut model);
    assert!(shut.contains(&head(false, 1, 2)), "{shut:?}");
    for word in [
        "line 001",
        "read(src/ui.rs)",
        "first I will read",
        "weighing",
    ] {
        assert!(!shut.iter().any(|t| t.contains(word)), "{word}: {shut:?}");
    }
    click(&window, &head(false, 1, 2), |ctx| {
        crate::ui::render(ctx, &mut model);
    });
    let compact = runs(&window, &mut model);
    assert!(compact.contains(&call_line()), "{compact:?}");
    assert!(compact.contains(&prose_line()), "{compact:?}");
    let thought = format!(
        "{} model-a (thinking) {} weighing two seams",
        glyph::NOTE,
        glyph::NOTE
    );
    assert!(compact.contains(&thought), "{compact:?}");
    assert!(compact.iter().any(|t| t == "line 001"), "{compact:?}");
    assert!(
        !compact.iter().any(|t| t.contains("line 002")),
        "{compact:?}"
    );
    assert!(!compact.contains(&glyph::CALL.to_owned()), "{compact:?}");
    click(&window, &call_line(), |ctx| {
        crate::ui::render(ctx, &mut model);
    });
    let whole = runs(&window, &mut model);
    assert!(whole.contains(&glyph::CALL.to_owned()), "{whole:?}");
    assert!(whole.iter().any(|t| t.contains("line 006")), "{whole:?}");
    assert!(
        !whole.iter().any(|t| t.contains("line 007")),
        "folded: {whole:?}"
    );
    click(&window, "read(src/ui.rs)", |ctx| {
        crate::ui::render(ctx, &mut model);
    });
    let again = runs(&window, &mut model);
    assert!(again.contains(&call_line()), "{again:?}");
    assert!(!again.iter().any(|t| t.contains("line 002")), "{again:?}");
}

/// **A finished call's output is one line elided at the width**: a preview,
/// not the content, cut with `…` on the glass.
#[test]
fn a_finished_output_is_one_line_elided_at_the_width() {
    let long = format!("{} end", "x".repeat(600));
    let mut model = with(vec![
        said("op", "port it"),
        working(),
        result(&long, false),
        prose("the seam is real"),
    ]);
    let window = Window::sized(1400.0, 900.0);
    window.frame(Vec::new(), |ctx| crate::ui::render(ctx, &mut model));
    click(&window, &head(false, 1, 1), |ctx| {
        crate::ui::render(ctx, &mut model);
    });
    let compact = runs(&window, &mut model);
    let cut = compact
        .iter()
        .find(|t| t.starts_with("xxx"))
        .unwrap_or_else(|| panic!("{compact:?}"));
    assert!(cut.ends_with(glyph::ELIDED), "{cut}");
    assert!(cut.len() < long.len(), "{cut}");
    assert!(!compact.contains(&long), "{compact:?}");
}

/// **While the turn runs, the step in flight stands whole and the finished
/// ones compact**: the prose streaming is on the glass whole, on its own
/// fill inside the working's; the call that returned and the prose a step
/// followed are one line each; and no elbow, since no answer stands yet.
#[test]
fn a_running_turn_paints_the_step_in_flight_whole_and_the_finished_ones_compact() {
    let mut model = with(vec![
        said("op", "port it"),
        prose("first I will read"),
        working(),
        result("done", false),
        running("so far"),
    ]);
    let output = glass(&mut model);
    let seen: Vec<String> = seen_of(&output).into_iter().map(|run| run.text).collect();
    assert!(seen.contains(&head(true, 1, 3)), "{seen:?}");
    assert!(seen.iter().any(|t| t == "so far"), "{seen:?}");
    assert!(seen.contains(&call_line()), "{seen:?}");
    assert!(seen.contains(&prose_line()), "{seen:?}");
    assert!(!seen.contains(&glyph::CALL.to_owned()), "{seen:?}");
    assert!(
        !seen.iter().any(|t| t == super::super::RESPONSE),
        "{seen:?}"
    );
    let (between, _) = theme::bubble(theme::Bubble::Intermediate);
    let streaming = seen_of(&output)
        .into_iter()
        .find(|run| run.text == "so far")
        .expect("the streaming prose is on the glass")
        .laid;
    assert!(
        fills_of(&output)
            .iter()
            .any(|(rect, ink)| *ink == between && rect.contains_rect(streaming)),
        "the streaming prose stands on the intermediate fill"
    );
}

/// **A compact step lights under the pointer** (§4.45 item 6), as the
/// working's line does: the hover wash under it, none away from it.
#[test]
fn a_compact_step_lights_under_the_pointer() {
    let mut model = with(vec![
        said("op", "port it"),
        working(),
        result("done", true),
        prose("the seam is real"),
    ]);
    let window = Window::sized(1400.0, 900.0);
    window.frame(Vec::new(), |ctx| crate::ui::render(ctx, &mut model));
    click(&window, &head(false, 1, 1), |ctx| {
        crate::ui::render(ctx, &mut model);
    });
    let failed = format!("{} read(src/ui.rs)", glyph::FAILED);
    let at = locate_in(&window, &failed, |ctx| {
        crate::ui::render(ctx, &mut model);
    })
    .expect("the failed step is on the glass");
    let washed = |output: &egui::FullOutput| {
        fills_of(output)
            .iter()
            .any(|(_, ink)| *ink == theme::hover())
    };
    // The click that opened the working left the pointer on its line; the
    // frame at rest is the one after the pointer went, since egui clears the
    // seat it interacts with a frame after the pointer itself.
    window.frame(vec![egui::Event::PointerGone], |ctx| {
        crate::ui::render(ctx, &mut model);
    });
    let away = window.frame(Vec::new(), |ctx| crate::ui::render(ctx, &mut model));
    assert!(!washed(&away), "bare at rest");
    let red = theme::accent(theme::State::Error);
    assert!(
        seen_of(&away)
            .iter()
            .any(|run| run.text == failed && run.ink == red),
        "a failed step wears the error accent compact"
    );
    let under = window.frame(vec![egui::Event::PointerMoved(at)], |ctx| {
        crate::ui::render(ctx, &mut model);
    });
    assert!(washed(&under), "lit under the pointer");
}

/// **The line counts what it hides**: calls are calls, answered or not, and a
/// result whose call is not on the read is the end of a call still; every
/// other row — the reasoning and the prose between — is a note.
#[test]
fn the_line_counts_calls_and_notes() {
    let transcript = with(vec![
        prose("first I will read"),
        working(),
        result("done", false),
        entry(EntryKind::ToolResult {
            tool_use_id: "tu-9".to_owned(),
            content: "orphaned".to_owned(),
            is_error: false,
        }),
        running("so far"),
    ])
    .transcript;
    let shown = rows(&transcript, None);
    assert_eq!(counted(&shown), "worked · 2 calls · 3 notes", "{shown:?}");
    assert_eq!(counted(&shown[1..3]), "worked · 1 call · 1 note");
}

/// **A peer's words inside the working and a call that answered nothing**:
/// the peer's compact line wears the note's glyph, since the words are not
/// the model's, and a call whose result was empty is its line alone — no
/// blank line beneath claiming output.
#[test]
fn a_peer_s_note_and_an_empty_result_compact_to_their_lines_alone() {
    let mut model = with(vec![
        said("op", "port it"),
        said("judge-one", "seconded"),
        working(),
        result("", false),
        prose("the seam is real"),
    ]);
    let window = Window::sized(1400.0, 900.0);
    window.frame(Vec::new(), |ctx| crate::ui::render(ctx, &mut model));
    click(&window, &head(false, 1, 2), |ctx| {
        crate::ui::render(ctx, &mut model);
    });
    let compact = runs(&window, &mut model);
    let peer = format!("{} judge-one {} seconded", glyph::NOTE, glyph::NOTE);
    assert!(compact.contains(&peer), "{compact:?}");
    // The next run down is the elbow's `response`, not a blank line.
    let below = compact
        .iter()
        .position(|t| *t == call_line())
        .and_then(|i| compact.get(i + 1))
        .cloned();
    assert_eq!(
        below.as_deref(),
        Some(super::super::RESPONSE),
        "{compact:?}"
    );
}
