//! The assignment, and the read that goes ahead of it.

use super::model;
use crate::render::Form;
use crate::test_support::Scratch;
use crate::test_support::wire::{flat, wired};
use serde_json::{Value, json};

/// A provider row's own listing.
fn offers(ids: &[&str]) -> Value {
    json!({"ok": true, "kind": "models", "rows": ids})
}

/// The assignment's captured run.
fn wrote() -> Value {
    json!({"ok": true, "kind": "outcome", "exit": 0, "stdout": "", "stderr": ""})
}

/// Assign a model and hand back the verdict beside anything it warned about.
fn assigned(scratch: &Scratch, id: &str) -> (crate::cli::Verdict, Vec<String>) {
    let mut warned: Vec<String> = Vec::new();
    let verdict = model(
        scratch.path(),
        "home",
        "worker",
        "claude-session-direct",
        id,
        Form::Json,
        &mut |said| warned.push(said.to_owned()),
    );
    (verdict, warned)
}

/// **An id the row does not offer is said, in full, and written anyway**
/// (bl-1e5a). The list belongs to the provider and the engine validates at the
/// first live call, so the seat is not a second authority — what it buys is
/// that the operator learns it now rather than a turn later, off a truncated
/// failure on a row.
#[test]
fn an_id_the_provider_does_not_offer_is_named_beside_the_whole_row() {
    let scratch = Scratch::new();
    let engine = wired(
        &scratch,
        &flat(),
        vec![vec![offers(&["opus-5", "sonnet-4"])], vec![wrote()]],
    );
    let (verdict, warned) = assigned(&scratch, "not-a-real-model-xyz");
    assert_eq!(verdict.code, 0, "the assignment still lands");
    assert_eq!(warned.len(), 1, "{warned:?}");
    let said = &warned[0];
    assert!(said.contains("not-a-real-model-xyz"), "{said}");
    assert!(said.contains("assigning it anyway"), "{said}");
    assert!(said.contains("opus-5, sonnet-4"), "{said}");
    assert!(
        engine
            .heard()
            .contains(&json!({"op": "model", "workspace": "home",
                                        "role": "worker",
                                        "provider": "claude-session-direct",
                                        "model": "not-a-real-model-xyz"})),
        "{:?}",
        engine.heard()
    );
}

/// An id the row offers is written with nothing said about it.
#[test]
fn an_id_the_provider_offers_is_assigned_in_silence() {
    let scratch = Scratch::new();
    wired(
        &scratch,
        &flat(),
        vec![vec![offers(&["opus-5", "sonnet-4"])], vec![wrote()]],
    );
    let (verdict, warned) = assigned(&scratch, "opus-5");
    assert_eq!(verdict.code, 0);
    assert!(warned.is_empty(), "{warned:?}");
}

/// **Every way the read can fail to answer a listing says nothing**, and the
/// assignment proceeds: a row that refuses, one that answers a kind this build
/// cannot read, and one that offers nothing are each this seat declining to
/// hold a second opinion about somebody else's table.
#[test]
fn a_row_that_cannot_say_what_it_offers_costs_the_assignment_nothing() {
    for answer in [
        json!({"ok": false, "error": "unknown provider `claude-session-direct`"}),
        json!({"ok": true, "kind": "nudged"}),
        offers(&[]),
    ] {
        let scratch = Scratch::new();
        wired(&scratch, &flat(), vec![vec![answer.clone()], vec![wrote()]]);
        let (verdict, warned) = assigned(&scratch, "not-a-real-model-xyz");
        assert_eq!(verdict.code, 0, "{answer}");
        assert!(warned.is_empty(), "{answer}: {warned:?}");
    }
}

/// A channel that will not open says nothing either, and the assignment's own
/// refusal is what the operator gets — one sentence about this box, not two.
#[test]
fn an_unreachable_channel_warns_nothing_and_the_assignment_says_why() {
    let scratch = Scratch::new();
    let (verdict, warned) = assigned(&scratch, "opus-5");
    assert_eq!(verdict.code, 1);
    assert!(warned.is_empty(), "{warned:?}");
    assert!(
        verdict.text.contains("no wire provisioned"),
        "{}",
        verdict.text
    );
}
