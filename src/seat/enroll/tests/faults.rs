//! **Every way the enrollment does not arrive**, and what each one says.
//!
//! Split from [`super::said`] at the design-time budget: an outcome added here
//! is a failure class, where one added there is a rendering. The costliest is
//! the last — an act that crossed with no answer, whose product is the one
//! reply nothing keeps.

use serde_json::{Value, json};

use super::super::INDOUBT;
use super::{acted, minted, spent, tree};
use crate::cli::Stream;
use crate::test_support::{Scratch, wire};

/// **A destination that will not take the material never costs it** (bl-1554).
/// The enrollment is already spent at the engine, so the answer goes to stdout
/// whichever way the filing went and only the exit code says no — and the
/// reason it was not filed is a diagnosis, so it is said on the other stream
/// (bl-ac76), which is what lets the answer be one envelope under `--json`.
#[test]
fn a_destination_that_refuses_still_says_the_material_and_fails() {
    let scratch = Scratch::new();
    let _engine = wire::wired(&scratch, &wire::flat(), vec![vec![minted()]]);
    let blocked = scratch.path().join("occupied");
    std::fs::write(&blocked, "notreal").expect("the blocker was written");
    let (verdict, warned) = acted(
        scratch.path(),
        "phone-1",
        "foot",
        None,
        Some(&blocked.join("under")),
        crate::render::Form::Rendered,
    );
    assert_eq!(verdict.code, 1, "{}", verdict.text);
    assert_eq!(
        verdict.stream,
        Stream::Out,
        "the product is still a product"
    );
    assert!(warned.concat().contains("not filed"), "{warned:?}");
    assert!(verdict.text.contains("yog-enroll"), "{}", verdict.text);
}

/// **A refusal is the engine answering**, so it is this run's product: it goes
/// to stdout with the rest of the stream and only the exit code says no. It
/// carries no material to withhold.
#[test]
fn a_refusal_is_answered_rather_than_drawn() {
    let (verdict, ..) = spent(vec![json!({"ok": false, "error": "not operator-grade"})]);
    assert_eq!(verdict.stream, Stream::Out);
    assert_eq!(verdict.code, 1);
    assert!(
        verdict.text.contains("not operator-grade"),
        "{}",
        verdict.text
    );
}

/// A frame this seat cannot read is a statement about **this seat**, and it
/// fails rather than answering — there is nothing to draw.
#[test]
fn an_unreadable_frame_fails_in_this_seat_s_own_words() {
    let (verdict, ..) = spent(vec![json!({"ok": true, "kind": "enrolled"})]);
    assert_eq!(verdict.stream, Stream::Err);
    assert!(verdict.text.contains("grade"), "{}", verdict.text);
}

/// **A well-formed answer of the wrong kind is not "unreadable"** — this seat
/// read it perfectly well. Saying otherwise would send an operator to upgrade
/// something that is fine.
#[test]
fn an_answer_of_another_kind_says_nothing_was_minted() {
    let (verdict, ..) = spent(vec![json!({"ok": true, "kind": "nudged"})]);
    assert_eq!(verdict.stream, Stream::Err);
    assert!(verdict.text.contains("did not mint"), "{}", verdict.text);
}

/// A channel that will not answer is a fact about this box or the far end, and
/// earns the sentence alone.
#[test]
fn a_root_with_no_channel_fails_before_anything_is_asked() {
    let scratch = Scratch::new();
    let (verdict, _) = acted(
        scratch.path(),
        "phone-1",
        "foot",
        None,
        None,
        crate::render::Form::Rendered,
    );
    assert_eq!(verdict.stream, Stream::Err);
    assert_eq!(verdict.code, 1);
    assert_eq!(tree(scratch.path()), Vec::<String>::new());
}

/// **Material too big for any symbol costs the picture, never the material.**
/// REMOTE §8.4 measures the envelope at about 1567 bytes against a 2331-byte
/// ceiling, so this is a recipe that moved — an RSA key, a longer chain — and
/// saying the size is what makes that legible rather than mysterious. But the
/// enrollment is already spent at the engine, so withholding the answer over a
/// drawing would burn the name: the line is said, and the symbol is what is
/// missing.
#[test]
fn material_too_big_for_a_symbol_says_so_and_still_says_the_material() {
    let mut oversized = minted();
    oversized["ca"] = Value::String("notreal".repeat(400));
    let (verdict, before, after) = spent(vec![oversized]);
    assert_eq!(verdict.stream, Stream::Out);
    assert_eq!(verdict.code, 0, "{}", verdict.text);
    assert!(verdict.text.contains("will not fit"), "{}", verdict.text);
    assert!(
        verdict.text.contains("2331"),
        "the ceiling: {}",
        verdict.text
    );
    assert!(verdict.text.contains("yog-enroll"), "{}", verdict.text);
    assert_eq!(before, after, "it still wrote nothing");
}

/// **An enrollment that crossed with no answer is the costliest doubt this
/// seat has** (REMOTE §3, bl-3969): its product is the one reply nothing here
/// keeps, so a registration may now exist whose material is gone. [`KEPT`]'s
/// *enroll again* is the right advice when the picture was drawn and the wrong
/// advice here, so this sentence refuses it and names the read.
#[test]
fn an_enrollment_that_crossed_with_no_answer_is_in_doubt_and_says_not_to_repeat() {
    let scratch = Scratch::new();
    let _engine = wire::wired(
        &scratch,
        &wire::flat(),
        vec![crate::test_support::engine::Answer::Hangup],
    );
    let (verdict, _) = acted(
        scratch.path(),
        "phone",
        "seat",
        None,
        None,
        crate::render::Form::Rendered,
    );
    assert_eq!(verdict.code, 1);
    assert!(verdict.text.contains(INDOUBT), "{}", verdict.text);
    assert!(
        verdict.text.contains("clients"),
        "the recovery is a read, and it is named: {}",
        verdict.text
    );
}

/// An engine that closes without a frame has answered nothing, and that is its
/// own sentence rather than a reading of an absent object.
#[test]
fn an_empty_stream_says_the_engine_answered_nothing() {
    let (verdict, ..) = spent(Vec::new());
    assert!(verdict.text.contains("nothing at all"), "{}", verdict.text);
}
