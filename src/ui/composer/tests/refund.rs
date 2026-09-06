//! **What a refused act does to the box it came out of** (bl-e85f).
//!
//! The seat's own sentence for an act that never left this box is *nothing
//! happened — it is safe to do it again*, and the seat then threw away the one
//! thing *again* needs. The beats here run the whole seam the operator does:
//! the composer fires, the send leg fails the way a stopped engine fails it,
//! and the words are read back off the glass.

use super::{SEND, render};
use crate::channel::Reach;
use crate::paint_probe::frame::Window;
use crate::test_support::window::{click, pane, seated};
use crate::ui::Model;

/// The paragraph an operator does not want to type twice.
const GOAL: &str = "port the paint probe, and say what the walk cannot answer";

/// **Compose a deposit the way a click does**, and hand back the act's own
/// words as the send leg would carry them — off the envelope, never off the
/// model, because that is the route the fix runs on.
fn deposited(model: &mut Model) -> Option<String> {
    let window = Window::new();
    click(&window, SEND, |ctx| {
        egui::CentralPanel::default().show(ctx, |ui| render(ui, model));
    });
    let posted = model.outbox.last().expect("the click composed a deposit");
    crate::envelope::said(&posted.envelope)
}

/// What the composer has in its box, read off the glass.
fn box_reads(model: &mut Model) -> String {
    pane(|ui| render(ui, model))
}

#[test]
fn an_act_that_never_left_this_box_gives_its_words_back() {
    let mut model = seated();
    model.draft = GOAL.to_owned();
    let said = deposited(&mut model);
    assert_eq!(said.as_deref(), Some(GOAL), "the words rode with the act");
    assert!(model.draft.is_empty(), "and the box cleared on firing");

    model.acted(
        crate::verbs::MESSAGE.word,
        &Reach::Unsent("connect: connection refused".to_owned()),
        said,
    );
    assert_eq!(model.draft, GOAL);
    assert!(
        box_reads(&mut model).contains(GOAL),
        "and it is on the glass"
    );
}

/// **IN DOUBT is the opposite case and must not refund.** The engine had the
/// gesture and may have run it; words back under a live `send` would invite
/// exactly the resend the contract forbids.
#[test]
fn an_act_that_crossed_and_was_never_answered_keeps_its_words_to_itself() {
    let mut model = seated();
    model.draft = GOAL.to_owned();
    let said = deposited(&mut model);
    model.acted(
        crate::verbs::MESSAGE.word,
        &Reach::Unanswered("read: timed out".to_owned()),
        said,
    );
    assert!(model.draft.is_empty(), "{:?}", model.draft);
}

/// **A box the operator has typed into since is theirs**, which is the start's
/// own rule (bl-b180) and the reason one refund serves both.
#[test]
fn a_refund_never_clobbers_what_was_typed_after_it() {
    let mut model = seated();
    model.draft = GOAL.to_owned();
    let said = deposited(&mut model);
    model.draft = "something else entirely".to_owned();
    model.acted(
        crate::verbs::MESSAGE.word,
        &Reach::Unsent("connect: connection refused".to_owned()),
        said,
    );
    assert_eq!(model.draft, "something else entirely");
}

/// **A gesture with no words carries none**, so the refund has nothing to give
/// back and the box is untouched — the ordinary case for every act on this
/// window that is composed from what is on the glass.
#[test]
fn a_gesture_that_carries_no_words_refunds_nothing() {
    let mut model = seated();
    model.acted(
        crate::verbs::NUDGE.word,
        &Reach::Unsent("connect: connection refused".to_owned()),
        crate::envelope::said(&crate::verbs::nudge("home".to_owned(), "c-1".to_owned())),
    );
    assert!(model.draft.is_empty());
}
