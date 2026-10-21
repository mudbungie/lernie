//! **The four bubbles, read off the glass** (DESIGN §4.45 item 7): the fill
//! under each block says its role, the gutter glyph stands inside it, and a
//! streaming turn's prose stands on the intermediate rung.

use super::idiom::{glass, prose, result, running, with, working};
use super::line::head;
use crate::paint_probe::frame::Window;
use crate::paint_probe::{fills_of, seen_of, strokes_of};
use crate::test_support::window::{click, said};
use crate::ui::theme::{Bubble, bubble, glyph};

/// **The colours of the column-wide fills under any run reading `text`** —
/// the bubble a block stands on, and the glass under that. A run the frame
/// never painted fails here, with the text in the message.
fn under(output: &egui::FullOutput, text: &str) -> Vec<egui::Color32> {
    let runs: Vec<egui::Rect> = seen_of(output)
        .into_iter()
        .filter(|seen| seen.text == text)
        .map(|seen| seen.laid)
        .collect();
    assert!(!runs.is_empty(), "nothing on the glass reads {text:?}");
    fills_of(output)
        .into_iter()
        .filter(|(rect, _)| rect.width() > 300.0 && runs.iter().any(|run| rect.contains_rect(*run)))
        .map(|(_, ink)| ink)
        .collect()
}

/// The fill `role` wears.
fn fill(role: Bubble) -> egui::Color32 {
    bubble(role).0
}

/// **One turn of each role, each on its fill, each glyph inside its fill.**
/// The intermediate prose stands inside the working (§4.45 item 5), so it
/// is two clicks from the settled transcript: the working's line, then its
/// own compact line — and whole, it is on its own fill inside the trail's.
#[test]
fn each_block_stands_on_the_fill_its_role_wears_with_its_glyph_inside() {
    let mut model = with(vec![
        said("op", "port it"),
        prose("first I will read"),
        working(),
        result("done", false),
        prose("the seam is real"),
    ]);
    let window = Window::sized(1400.0, 900.0);
    window.frame(Vec::new(), |ctx| crate::ui::render(ctx, &mut model));
    click(&window, &head(false, [1, 1, 1, 0]), |ctx| {
        crate::ui::render(ctx, &mut model);
    });
    let line = format!("{} model-a {} first I will read", glyph::MODEL, glyph::NOTE);
    click(&window, &line, |ctx| {
        crate::ui::render(ctx, &mut model);
    });
    let output = window.frame(Vec::new(), |ctx| crate::ui::render(ctx, &mut model));
    let operator = fill(Bubble::Operator);
    assert!(under(&output, "port it").contains(&operator));
    assert!(
        under(&output, glyph::PROMPT).contains(&operator),
        "the glyph is inside the fill"
    );
    let (between, last) = (fill(Bubble::Intermediate), fill(Bubble::Final));
    let first = under(&output, "first I will read");
    assert!(
        first.contains(&between) && !first.contains(&last),
        "{first:?}"
    );
    let seam = under(&output, "the seam is real");
    assert!(seam.contains(&last) && !seam.contains(&between), "{seam:?}");
    assert!(under(&output, glyph::MODEL).contains(&last));
    // The trail is the ground inside a hairline: the ground is the glass's
    // own colour, so what tells it from the pane is the stroke around it.
    let head = seen_of(&output)
        .into_iter()
        .find(|seen| seen.text == head(true, [1, 1, 1, 0]))
        .expect("the working's line is on the glass")
        .laid;
    let (_, ring) = bubble(Bubble::Trail);
    assert!(
        strokes_of(&output)
            .iter()
            .any(|(rect, ink)| *ink == ring.color && rect.contains_rect(head)),
        "the trail is ringed by the hairline"
    );
}

/// **While the turn streams, its prose is intermediate** — what follows it
/// is not known yet — and once it settles the same words are final.
#[test]
fn a_streaming_turn_s_prose_is_intermediate_until_it_settles() {
    let (between, last) = (fill(Bubble::Intermediate), fill(Bubble::Final));
    let mut streaming = with(vec![said("op", "port it"), running("so far")]);
    let flying = under(&glass(&mut streaming), "so far");
    assert!(
        flying.contains(&between) && !flying.contains(&last),
        "{flying:?}"
    );
    let mut settled = with(vec![said("op", "port it"), prose("so far")]);
    let done = under(&glass(&mut settled), "so far");
    assert!(done.contains(&last) && !done.contains(&between), "{done:?}");
}
