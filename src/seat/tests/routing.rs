//! Which engine a gesture reaches, what it carries there, and what a box that
//! cannot reach one says.

use super::{ask, entry, flat, route, wired, yes};
use crate::channel::entries::{WIRE, WORKSPACE};
use crate::channel::material::ADDRESS;
use crate::cli::Stream;
use crate::test_support::{Scratch, mint};
use serde_json::json;

/// **A gesture naming no workspace goes to the box's own engine**, which is
/// where every name no entry holds goes too.
#[test]
fn an_unaddressed_gesture_goes_to_the_flat_root() {
    let scratch = Scratch::new();
    let engine = wired(&scratch, &flat(), vec![vec![yes()]]);
    let verdict = ask(scratch.path(), &json!({"op": "workspaces"}));
    assert_eq!(verdict.code, 0);
    assert_eq!(verdict.stream, Stream::Out);
    assert_eq!(verdict.text, yes().to_string());
    assert!(
        engine.heard().contains(&json!({"op": "workspaces"})),
        "{:?}",
        engine.heard()
    );
}

/// **A name an entry holds goes down that entry's channel** — on that entry's
/// own material, which is a different trust root from the flat one.
#[test]
fn a_name_an_entry_holds_goes_down_that_entry_s_channel() {
    let scratch = Scratch::new();
    let engine = wired(&scratch, &entry("home"), vec![vec![yes()]]);
    let verdict = ask(
        scratch.path(),
        &json!({"op": "conversations", "workspace": "home"}),
    );
    assert_eq!(verdict.code, 0);
    assert!(
        engine
            .heard()
            .contains(&json!({"op": "conversations", "workspace": "home"})),
        "{:?}",
        engine.heard()
    );
}

/// **The rename is spent at the channel boundary and nowhere else** (§8.2): the
/// envelope the HOST is handed carries the host's name, while every layer above
/// this one reasoned in the leaf.
#[test]
fn an_entry_that_renames_carries_the_host_s_name_across() {
    let scratch = Scratch::new();
    let engine = wired(&scratch, &entry("home"), vec![vec![yes()]]);
    std::fs::write(
        scratch.path().join(entry("home")).join(WORKSPACE),
        "personal",
    )
    .expect("the workspace file");
    assert_eq!(
        ask(
            scratch.path(),
            &json!({"op": "conversations", "workspace": "home"})
        )
        .code,
        0
    );
    assert!(
        engine
            .heard()
            .contains(&json!({"op": "conversations", "workspace": "personal"})),
        "the gesture crossed in this box's spelling: {:?}",
        engine.heard()
    );
}

/// Where the two names agree — the ordinary provisioning — the operator's own
/// envelope crosses byte for byte, because there is nothing to rewrite.
#[test]
fn an_entry_that_renames_nothing_carries_the_envelope_unchanged() {
    let scratch = Scratch::new();
    mint::provisioned(&scratch.path().join(entry("home")), "engine.example:9000");
    let envelope = json!({"op": "conversations", "workspace": "home", "extra": [1, 2]});
    let (_channel, carried) = route(scratch.path(), &envelope).expect("routed");
    assert_eq!(carried, envelope);
}

/// **An entry that exists is the answer to its name even when it cannot be
/// dialled.** Falling through to the flat root would send a gesture to the
/// wrong engine on the strength of a missing file — so a hollow entry refuses
/// in its own words while the flat root stands ready beside it.
#[test]
fn a_hollow_entry_refuses_rather_than_falling_through_to_the_flat_root() {
    let scratch = Scratch::new();
    wired(&scratch, &flat(), vec![vec![yes()]]);
    std::fs::create_dir_all(scratch.path().join(entry("home"))).expect("mkdir");
    let verdict = ask(
        scratch.path(),
        &json!({"op": "conversations", "workspace": "home"}),
    );
    assert_eq!(verdict.code, 1);
    assert_eq!(verdict.stream, Stream::Err);
    assert!(
        verdict.text.contains("is an empty entry"),
        "{}",
        verdict.text
    );
}

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
