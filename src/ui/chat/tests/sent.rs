//! **A sent message is on the glass the frame it is sent** (DESIGN §4.45
//! item 4): read off the painted glyphs, in the ink they reached the glass in.
//!
//! The beats run the whole seam the operator does — Enter in the box, the
//! receipt, the read that carries the words, the refusal — against the whole
//! window, because the claim is about one frame: the composer fires in the
//! bottom panel and the transcript paints after it in the same pass.

use crate::channel::Reach;
use crate::paint_probe::frame::{Window, press};
use crate::paint_probe::{Seen, seen_of};
use crate::reply::{Outcome, Read, Reply};
use crate::test_support::window::{own, said, seated};
use crate::ui::Model;
use crate::ui::theme;

const WORDS: &str = "ship it";

/// **Enter on the box with `WORDS` in it**, and what that frame painted.
fn sent(model: &mut Model, window: &Window) -> Vec<Seen> {
    model.draft = WORDS.to_owned();
    window.frame(Vec::new(), |ctx| crate::ui::render(ctx, model));
    window.focus(egui::Id::new(crate::ui::keys::BOX_ID));
    seen_of(&window.frame(vec![press(egui::Key::Enter)], |ctx| {
        crate::ui::render(ctx, model);
    }))
}

/// The runs reading exactly `WORDS`, as inks.
fn inks(shown: &[Seen]) -> Vec<egui::Color32> {
    shown
        .iter()
        .filter(|run| run.text == WORDS)
        .map(|run| run.ink)
        .collect()
}

/// One settled frame of the whole window, as inks of `WORDS`.
fn painted(model: &mut Model, window: &Window) -> Vec<egui::Color32> {
    inks(&seen_of(
        &window.frame(Vec::new(), |ctx| crate::ui::render(ctx, model)),
    ))
}

/// A deposit's `ok` receipt.
fn ok() -> Read {
    Read::Answer(Reply::Outcome(Outcome {
        exit: 0,
        stdout: String::new(),
        stderr: String::new(),
    }))
}

/// **The same frame, in the pending ink**; the receipt turns it to body ink;
/// the read that carries the words leaves exactly one row, in body ink.
#[test]
fn the_words_are_on_the_glass_the_frame_enter_is_pressed() {
    let mut model = seated();
    let window = Window::sized(1400.0, 900.0);
    let shown = sent(&mut model, &window);
    // The box painted its draft before Enter cleared it, in the body ink;
    // the transcript, painted after it in the same pass, has the row.
    assert_eq!(inks(&shown), vec![theme::INK, theme::INK_WEAK], "{shown:?}");
    assert!(model.draft.is_empty(), "and the box cleared");

    let flat = own().channel;
    model.receipt(&flat, crate::verbs::MESSAGE.word, ok());
    assert_eq!(painted(&mut model, &window), vec![theme::INK]);

    model.absorb(
        &flat,
        Read::Answer(Reply::Transcript(crate::reply::transcript::Transcript {
            entries: vec![said("op", "port it"), said("user", WORDS)],
        })),
    );
    assert_eq!(
        painted(&mut model, &window),
        vec![theme::INK],
        "one row, not two"
    );
    assert!(model.sent.is_empty());
}

/// **A refusal takes the row down and puts the words back in the box**, where
/// they are read off the glass again — as the draft, not as a row.
#[test]
fn a_refusal_puts_the_words_back_in_the_box_and_takes_the_row_down() {
    let mut model = seated();
    let window = Window::sized(1400.0, 900.0);
    sent(&mut model, &window);
    model.receipt(
        &own().channel,
        crate::verbs::MESSAGE.word,
        Read::Refusal("the server said no".to_owned()),
    );
    assert_eq!(model.draft, WORDS);
    let shown = seen_of(&window.frame(Vec::new(), |ctx| crate::ui::render(ctx, &mut model)));
    assert_eq!(
        inks(&shown).len(),
        1,
        "the box's copy and no row: {shown:?}"
    );
    assert!(model.sent.is_empty());
}

/// **Keyed by conversation**: another tab paints none of this one's rows, and
/// coming back paints it again.
#[test]
fn another_conversation_does_not_paint_this_one_s_row() {
    let mut model = seated();
    let window = Window::sized(1400.0, 900.0);
    sent(&mut model, &window);
    model.select("20260830T052000Z-b2c3");
    assert!(painted(&mut model, &window).is_empty());
    model.select("20260830T051200Z-a1b2");
    assert_eq!(painted(&mut model, &window), vec![theme::INK_WEAK]);
}

/// **In doubt, the row stands in the pending ink** beside the notice: the
/// server may have it.
#[test]
fn a_lost_reply_leaves_the_row_standing_in_the_pending_ink() {
    let mut model = seated();
    let window = Window::sized(1400.0, 900.0);
    sent(&mut model, &window);
    model.acted(
        crate::verbs::MESSAGE.word,
        &Reach::Unanswered("read: timed out".to_owned()),
        Some(WORDS.to_owned()),
    );
    assert_eq!(painted(&mut model, &window), vec![theme::INK_WEAK]);
    assert!(model.draft.is_empty());
}
