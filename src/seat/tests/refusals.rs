//! **What a box that cannot reach an engine says**, and what an engine's own
//! answer is worth.
//!
//! Split from [`super::routing`] at the line cap on the seam that module's own
//! doc already draws: [`super::routing`] is which engine a gesture reaches and
//! what it carries there, and this is what comes back — a channel that will not
//! open, one that will not answer, and the reply stream that is this seat's
//! product however it went.

use super::{ask, flat, wired};
use crate::channel::entries::WIRE;
use crate::channel::material::ADDRESS;
use crate::cli::Stream;
use crate::test_support::{Scratch, mint};
use serde_json::json;

/// A box with no channel at all refuses naming the directory and the act that
/// fills it — and carries no usage, because it is not about what was typed.
#[test]
fn a_box_with_no_wire_names_the_directory_and_the_operator_s_act() {
    let scratch = Scratch::new();
    let verdict = ask(scratch.path(), &json!({"op": "workspaces"}));
    assert_eq!(verdict.code, 1);
    assert!(
        verdict.text.contains("no wire provisioned"),
        "{}",
        verdict.text
    );
    assert!(
        verdict.text.contains("carried here by hand"),
        "{}",
        verdict.text
    );
    assert!(!verdict.text.contains("usage:"), "{}", verdict.text);
}

/// A half-provisioned flat root refuses with the material's own sentence rather
/// than the absent one.
#[test]
fn a_half_provisioned_flat_root_says_which_file_is_missing() {
    let scratch = Scratch::new();
    let dir = scratch.dir(WIRE);
    mint::provisioned(&dir, "engine.example:9000");
    std::fs::remove_file(dir.join(ADDRESS)).expect("rm");
    let verdict = ask(scratch.path(), &json!({"op": "workspaces"}));
    assert_eq!(verdict.code, 1);
    assert!(
        verdict.text.contains("half-provisioned"),
        "{}",
        verdict.text
    );
}

/// **Port zero is a request, not an address** (REMOTE §8): only the engine that
/// bound it knows what it became, and it tells its own in-process window in
/// RAM. A separately installed seat wants a stated address, and saying so beats
/// the raw connect error port zero earns.
#[test]
fn a_self_provisioned_loopback_root_says_a_seat_wants_a_stated_address() {
    let scratch = Scratch::new();
    mint::provisioned(&scratch.dir(WIRE), "127.0.0.1:0");
    let verdict = ask(scratch.path(), &json!({"op": "workspaces"}));
    assert_eq!(verdict.code, 1);
    assert!(
        verdict.text.contains("kernel-chosen port"),
        "{}",
        verdict.text
    );
    assert!(
        verdict.text.contains("a seat wants a stated address"),
        "{}",
        verdict.text
    );
}

/// A channel that will not answer is a failure about the far end, said alone.
#[test]
fn an_engine_that_is_not_there_fails_with_the_transport_s_sentence() {
    let scratch = Scratch::new();
    mint::provisioned(&scratch.dir(WIRE), "127.0.0.1:1");
    let verdict = ask(scratch.path(), &json!({"op": "workspaces"}));
    assert_eq!(verdict.code, 1);
    assert_eq!(verdict.stream, Stream::Err);
    assert!(
        verdict.text.contains("connect 127.0.0.1:1"),
        "{}",
        verdict.text
    );
}

/// **An engine answering no has ANSWERED.** The reply stream is the seat's
/// product, so it goes to stdout with every frame in order, and only the exit
/// code says no.
#[test]
fn a_refusing_reply_is_still_the_product_and_only_the_code_says_no() {
    let scratch = Scratch::new();
    let refusal = json!({"ok": false, "said": "no such workspace"});
    wired(
        &scratch,
        &flat(),
        vec![vec![json!({"ok": true}), refusal.clone()]],
    );
    let verdict = ask(scratch.path(), &json!({"op": "workspaces"}));
    assert_eq!(verdict.code, 1);
    assert_eq!(verdict.stream, Stream::Out, "an answer is not a diagnosis");
    assert_eq!(
        verdict.text,
        format!("{}\n{}", json!({"ok": true}), refusal),
        "every frame is the product, in order"
    );
}

/// An engine that terminates without answering is not ok, and prints nothing.
#[test]
fn an_engine_that_answers_nothing_is_not_ok() {
    let scratch = Scratch::new();
    wired(&scratch, &flat(), vec![vec![]]);
    let verdict = ask(scratch.path(), &json!({"op": "workspaces"}));
    assert_eq!(verdict.code, 1);
    assert_eq!(verdict.text, "");
}
