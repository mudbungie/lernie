//! **The bubbles nest** (DESIGN §4.48 item 2), read off the glass: the
//! operator's turn inside a hairline, the working one bubble with each step a
//! sub-bubble inset inside it, a call's output on the card in the monospace
//! family under a badge on its tint, the `bubbles` knob taking every one of
//! them away — and under a settled turn, the strip's `copy` door.

use super::idiom::{prose, result, running, with, working};
use super::line::head;
use crate::paint_probe::frame::{Window, locate_in};
use crate::paint_probe::{fills_of, seen_of, strokes_of};
use crate::settings::Toggle;
use crate::test_support::window::{click, said};
use crate::ui::Model;
use crate::ui::chat::COPY;
use crate::ui::theme::{self, Bubble, State, bubble, glyph, space};

/// The call's line, as the fixture names it.
fn call_line() -> String {
    format!("{} read(src/ui.rs)", glyph::DONE)
}

/// A settled turn whose one call said `Exit code: 0`, its working opened
/// and its call opened whole; answers the frame after.
fn opened(bubbles: bool) -> egui::FullOutput {
    let mut model = with(vec![
        said("op", "port it"),
        working(),
        result("Exit code: 0\nhello", false),
        prose("the seam is real"),
    ]);
    model.settings.set(Toggle::Bubbles, bubbles);
    let window = Window::sized(1400.0, 900.0);
    window.frame(Vec::new(), |ctx| crate::ui::render(ctx, &mut model));
    for door in [head(false, [1, 1, 0, 0]), call_line()] {
        click(&window, &door, |ctx| crate::ui::render(ctx, &mut model));
    }
    window.frame(Vec::new(), |ctx| crate::ui::render(ctx, &mut model))
}

/// Where the run reading exactly `text` was laid.
fn at(output: &egui::FullOutput, text: &str) -> egui::Rect {
    seen_of(output)
        .into_iter()
        .find(|run| run.text == text)
        .unwrap_or_else(|| panic!("nothing on the glass reads {text:?}"))
        .laid
}

/// The smallest rect of `ink` that holds `inside`, filled or stroked.
fn least(
    shapes: Vec<(egui::Rect, egui::Color32)>,
    ink: egui::Color32,
    inside: egui::Rect,
) -> Option<egui::Rect> {
    shapes
        .into_iter()
        .filter(|(rect, painted)| *painted == ink && rect.contains_rect(inside))
        .map(|(rect, _)| rect)
        .min_by(|a, b| a.area().total_cmp(&b.area()))
}

/// **Each layer inside the one that will collapse**: the operator's tint is
/// ringed; the working's hairline holds its line and its step; the step
/// stands on `SURFACE` `XXS` inside it; the call's output stands on the
/// card's ground inside the step, in the monospace family; and the badge
/// stands on its state's tint.
#[test]
fn the_working_is_one_bubble_and_each_step_a_sub_bubble_inside_it() {
    let output = opened(true);
    let hairline = bubble(Bubble::Operator).1.color;
    assert!(least(strokes_of(&output), hairline, at(&output, "port it")).is_some());
    let line = at(&output, &call_line());
    let working = least(strokes_of(&output), hairline, line).expect("the working's ring");
    assert!(working.contains_rect(at(&output, &head(true, [1, 1, 0, 0]))));
    let (surface, _) = bubble(Bubble::Step);
    let step = least(fills_of(&output), surface, line).expect("the step's sub-bubble");
    assert!(
        working.shrink(space::XXS - 0.5).contains_rect(step),
        "{step:?} inset in {working:?}"
    );
    let hello = seen_of(&output)
        .into_iter()
        .find(|run| run.text == "hello")
        .expect("the output is on the glass");
    assert_eq!(hello.family, egui::FontFamily::Monospace);
    let (ground, _) = bubble(Bubble::Card);
    let card = least(fills_of(&output), ground, hello.laid).expect("the card");
    assert!(step.contains_rect(card), "{card:?} inside {step:?}");
    let pill = theme::tint(State::Attention);
    assert!(least(fills_of(&output), pill, at(&output, "exit 0")).is_some());
}

/// **The `bubbles` knob off takes every fill and stroke above away** — the
/// ring, the step's surface, the card, the operator's hairline — and the
/// gutter glyphs say the role alone.
#[test]
fn bubbles_off_takes_the_nested_fills_and_strokes_away() {
    let output = opened(false);
    let hairline = bubble(Bubble::Operator).1.color;
    let line = at(&output, &call_line());
    assert!(least(strokes_of(&output), hairline, at(&output, "port it")).is_none());
    assert!(least(strokes_of(&output), hairline, line).is_none());
    assert!(least(fills_of(&output), bubble(Bubble::Step).0, line).is_none());
    let card = |output: &egui::FullOutput| {
        let hello = at(output, "hello");
        fills_of(output)
            .into_iter()
            .filter(|(rect, ink)| *ink == bubble(Bubble::Card).0 && rect.contains_rect(hello))
            .count()
    };
    assert!(
        card(&output) < card(&opened(true)),
        "no card under the output"
    );
}

/// **Under a settled turn, `copy` puts the final prose on the clipboard**;
/// under a running one there is no strip — its prose is not the answer yet.
#[test]
fn copy_under_a_settled_turn_puts_its_prose_on_the_clipboard() {
    let mut model = with(vec![said("op", "port it"), prose("the seam is real")]);
    let window = Window::sized(1400.0, 900.0);
    let mut copied = String::new();
    let mut body = |ctx: &egui::Context| {
        crate::ui::render(ctx, &mut model);
        let said = ctx.output(|o| o.copied_text.clone());
        if !said.is_empty() {
            copied = said;
        }
    };
    window.frame(Vec::new(), &mut body);
    let door = locate_in(&window, COPY, &mut body).expect("the strip's door");
    crate::paint_probe::frame::click(&window, door, &mut body);
    assert_eq!(copied, "the seam is real");
    let mut live: Model = with(vec![said("op", "port it"), running("so far")]);
    let seen = seen_of(&window.frame(Vec::new(), |ctx| crate::ui::render(ctx, &mut live)));
    assert!(
        !seen.iter().any(|run| run.text == COPY),
        "no strip while it runs"
    );
}

/// **A thought in flight stands whole on its sub-bubble**: its line is the
/// note's mark and its name — not a door — and its words stand beneath.
#[test]
fn a_thought_in_flight_is_its_mark_and_name_over_its_words() {
    let thinking = super::idiom::entry(crate::reply::transcript::EntryKind::Streaming {
        thinking: "pondering the seam".to_owned(),
        text: String::new(),
    });
    let mut model = with(vec![said("op", "port it"), thinking]);
    let window = Window::sized(1400.0, 900.0);
    window.frame(Vec::new(), |ctx| crate::ui::render(ctx, &mut model));
    let output = window.frame(Vec::new(), |ctx| crate::ui::render(ctx, &mut model));
    let mark = at(&output, glyph::NOTE);
    let words = at(&output, "pondering the seam");
    assert!(mark.max.y <= words.min.y + 1.0, "{mark:?} over {words:?}");
    let surface = bubble(Bubble::Step).0;
    assert!(least(fills_of(&output), surface, words).is_some());
}
