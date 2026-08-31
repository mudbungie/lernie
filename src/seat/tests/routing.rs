//! Which engine a gesture reaches, and what it carries there.

use super::{ask, entry, flat, route, wired, yes};
use crate::channel::entries::WORKSPACE;
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

/// **A gesture whose op takes no workspace is naming a CHANNEL**, so a name no
/// entry holds has no downstream reader to refuse it (bl-d574). It refuses
/// here, naming what it looked for and what this box actually holds — where it
/// used to fall through and answer `ok`, at exit 0, from a channel nobody
/// named.
#[test]
fn a_selector_naming_no_entry_refuses_instead_of_answering_from_the_flat_root() {
    let scratch = Scratch::new();
    let engine = wired(&scratch, &flat(), vec![vec![yes()]]);
    mint::provisioned(&scratch.path().join(entry("alpha")), "alpha.example:9000");
    let verdict = ask(
        scratch.path(),
        &json!({"op": "workspaces", "workspace": "NoSuchWs"}),
    );
    assert_eq!(verdict.code, 1);
    assert_eq!(verdict.stream, Stream::Err);
    assert!(
        verdict.text.contains(r#"no channel named "NoSuchWs""#),
        "{}",
        verdict.text
    );
    assert!(
        verdict.text.contains(r#"(this box's own engine), "alpha""#),
        "it says what this box does hold: {}",
        verdict.text
    );
    assert!(verdict.text.contains("`mv` it under"), "{}", verdict.text);
    assert!(
        engine.heard().is_empty(),
        "nothing was asked of a channel nobody named: {:?}",
        engine.heard()
    );
}

/// The escape hatch the refusal above must not close: a selector naming an
/// entry this box DOES hold reaches that entry's engine, which is how an
/// operator asks one channel for its roster.
#[test]
fn a_selector_naming_an_entry_reaches_that_entry_s_engine() {
    let scratch = Scratch::new();
    let engine = wired(&scratch, &entry("alpha"), vec![vec![yes()]]);
    let verdict = ask(
        scratch.path(),
        &json!({"op": "workspaces", "workspace": "alpha"}),
    );
    assert_eq!(verdict.code, 0);
    assert!(
        engine
            .heard()
            .contains(&json!({"op": "workspaces", "workspace": "alpha"})),
        "{:?}",
        engine.heard()
    );
}

/// **The fallthrough's refusal is about the name that was asked for** — where
/// it used to be about `wire/`, a directory the operator never named, with a
/// remedy that sends them back to the host's CA to mint a second leaf into the
/// same wrong directory (bl-d574).
#[test]
fn a_name_that_falls_through_to_an_unprovisioned_flat_root_names_itself() {
    let scratch = Scratch::new();
    mint::provisioned(&scratch.path().join(entry("alpha")), "alpha.example:9000");
    let verdict = ask(
        scratch.path(),
        &json!({"op": "conversations", "workspace": "beta"}),
    );
    assert_eq!(verdict.code, 1);
    assert_eq!(verdict.stream, Stream::Err);
    assert!(
        verdict.text.contains(r#"no entry here holds "beta""#),
        "{}",
        verdict.text
    );
    assert!(
        verdict.text.contains(r#"(this box's own engine), "alpha""#),
        "{}",
        verdict.text
    );
    assert!(
        verdict.text.contains("rather than minting a second leaf"),
        "the rename is offered ahead of the mint: {}",
        verdict.text
    );
    assert!(
        verdict.text.contains("no wire provisioned"),
        "the flat root's own sentence is still carried, as the cause: {}",
        verdict.text
    );
}

/// **§8.2's fallthrough survives the refusals above**: a name no entry holds,
/// on an op that takes a workspace, still reaches this box's own engine — which
/// is where the flat engine's own workspaces live, and they are named nowhere
/// this seat can read.
#[test]
fn a_name_no_entry_holds_still_reaches_the_flat_engine_when_the_op_takes_one() {
    let scratch = Scratch::new();
    let engine = wired(&scratch, &flat(), vec![vec![yes()]]);
    mint::provisioned(&scratch.path().join(entry("alpha")), "alpha.example:9000");
    let verdict = ask(
        scratch.path(),
        &json!({"op": "conversations", "workspace": "beta"}),
    );
    assert_eq!(verdict.code, 0);
    assert!(
        engine
            .heard()
            .contains(&json!({"op": "conversations", "workspace": "beta"})),
        "{:?}",
        engine.heard()
    );
}
