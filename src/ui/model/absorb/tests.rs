//! The door's receipts: what an act that answered with no content leaves on
//! the glass (bl-9760, bl-7ec8).

use super::{Model, Notice};
use crate::reply::{Outcome, Read, Reply};
use crate::test_support::window::{own, seated, seen};
use crate::ui::theme;

/// A run's receipt, exiting as `exit` and saying `stdout` and `stderr`.
fn ran(exit: i32, stdout: &str, stderr: &str) -> Read {
    Read::Answer(Reply::Outcome(Outcome {
        exit,
        stdout: stdout.to_owned(),
        stderr: stderr.to_owned(),
    }))
}

/// **Every receipt replaces the notice with a sentence, and none clears it.**
/// A run that failed is told in the child's own words; one that succeeded,
/// and each of the four bare receipts, is the CLI's own line — so an act that
/// did nothing still says so, and never reads as a key that went nowhere.
#[test]
fn a_receipt_replaces_the_notice_and_never_clears_it() {
    let mut model = Model {
        notice: Some(Notice::Refused("stale".to_owned())),
        ..Model::default()
    };
    let flat = own().channel;
    for (reply, said) in [
        (Reply::Nudged, "nudged — the advance is running"),
        (Reply::Flagged, "flagged — it is on the queue now"),
        (Reply::Acked, "the trail's watermark is where you are"),
        (Reply::TrailCleared, "the trail is truncated"),
    ] {
        model.absorb(&flat, Read::Answer(reply));
        assert_eq!(model.notice, Some(Notice::Said(said.to_owned())));
    }
    model.absorb(&flat, ran(1, "", "the gate said no"));
    assert_eq!(
        model.notice,
        Some(Notice::Refused("the gate said no".to_owned()))
    );
    model.absorb(&flat, ran(0, "", ""));
    assert_eq!(model.notice, Some(Notice::Said("ok".to_owned())));
}

/// **A retarget that had nothing to do says so, in the Annotation ink, on the
/// glass** — the engine's sentence read back off the painted glyphs, not off
/// the model, and Escape puts it down.
#[test]
fn a_successful_run_paints_its_own_sentence_in_the_annotation_ink() {
    let mut model = seated();
    model.absorb(
        &own().channel,
        ran(
            0,
            "litany: config/default already governs — nothing to retarget\n",
            "",
        ),
    );
    let window = crate::paint_probe::frame::Window::new();
    let shown = seen(&window, |ctx| crate::ui::render(ctx, &mut model));
    let line = shown
        .iter()
        .find(|run| run.text.contains("nothing to retarget"))
        .expect("the receipt reached the glass");
    assert!(line.text.starts_with("ok · litany"), "{}", line.text);
    assert_eq!(line.ink, theme::accent(theme::State::Annotation));
    crate::test_support::window::escape(&window, |ctx| crate::ui::render(ctx, &mut model));
    assert_eq!(model.notice, None);
}

/// **A deposit's receipt leaves the slot to the conversation**: the words sent
/// are on the transcript, so a send must not stand `ok` over its state.
#[test]
fn a_deposit_s_receipt_says_nothing_in_the_slot() {
    let mut model = Model {
        notice: Some(Notice::Refused("stale".to_owned())),
        ..Model::default()
    };
    let flat = own().channel;
    for op in [crate::verbs::MESSAGE.word, crate::verbs::INTERRUPT.word] {
        model.receipt(&flat, op, ran(0, "deposited", ""));
        assert_eq!(model.notice, None, "{op}");
    }
    model.receipt(&flat, "retarget", ran(0, "nothing to retarget", ""));
    assert_eq!(
        model.notice,
        Some(Notice::Said("ok · nothing to retarget".to_owned()))
    );
}
