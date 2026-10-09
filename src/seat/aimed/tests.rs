//! A world-level act, down the one channel named — or the one there is.

use super::aimed;
use crate::channel::entries::WORKSPACE;
use crate::cli::Stream;
use crate::render::Form;
use crate::test_support::Scratch;
use crate::test_support::wire::{entry, flat, wired, yes};
use serde_json::json;

/// The ceiling every test writes.
fn bound() -> serde_json::Value {
    json!({"op": "ceiling", "usd": 25})
}

/// **One channel, no name: that channel is the subject** — the ordinary box,
/// which has one engine to write and needs no word for it.
#[test]
fn a_box_holding_one_channel_writes_it_unnamed() {
    let scratch = Scratch::new();
    let alpha = wired(&scratch, &entry("alpha"), vec![vec![yes()]]);
    let verdict = aimed(scratch.path(), None, &bound(), Form::Json);
    assert_eq!(verdict.code, 0, "{}", verdict.text);
    assert_eq!(verdict.text, yes().to_string());
    assert!(alpha.heard().contains(&bound()));
}

/// **Two channels, no name: refused, naming both, and neither written** — the
/// fan this ball removed wrote one number into two worlds.
#[test]
fn a_box_holding_two_channels_refuses_an_unnamed_write_naming_both() {
    let scratch = Scratch::new();
    let own = wired(&scratch, &flat(), vec![vec![yes()]]);
    let alpha = wired(&scratch, &entry("alpha"), vec![vec![yes()]]);
    let verdict = aimed(scratch.path(), None, &bound(), Form::Json);
    assert_eq!(verdict.code, 1);
    assert_eq!(verdict.stream, Stream::Err);
    for named in [
        "2 channels",
        "`ceiling`",
        "--on <channel>",
        "(this box's own engine)",
        "alpha",
    ] {
        assert!(verdict.text.contains(named), "{named}: {}", verdict.text);
    }
    assert!(!own.heard().contains(&bound()));
    assert!(!alpha.heard().contains(&bound()));
}

/// **A named entry is the one written**, its neighbours untouched.
#[test]
fn a_named_entry_is_the_only_channel_written() {
    let scratch = Scratch::new();
    let own = wired(&scratch, &flat(), vec![vec![yes()]]);
    let alpha = wired(&scratch, &entry("alpha"), vec![vec![yes()]]);
    let verdict = aimed(scratch.path(), Some("alpha"), &bound(), Form::Json);
    assert_eq!(verdict.code, 0, "{}", verdict.text);
    assert!(alpha.heard().contains(&bound()));
    assert!(!own.heard().contains(&bound()));
}

/// **An entry whose workspace is named otherwise on its host is reached by
/// this box's name for it**, and the act crosses byte for byte: it names no
/// workspace, so there is nothing for §8.2's rename to rewrite.
#[test]
fn a_workspace_held_on_an_entry_is_reached_by_its_leaf_unrewritten() {
    let scratch = Scratch::new();
    wired(&scratch, &flat(), vec![vec![yes()]]);
    let alpha = wired(&scratch, &entry("alpha"), vec![vec![yes()]]);
    std::fs::write(
        scratch.path().join(entry("alpha")).join(WORKSPACE),
        "personal",
    )
    .expect("the workspace file");
    let verdict = aimed(scratch.path(), Some("alpha"), &bound(), Form::Json);
    assert_eq!(verdict.code, 0, "{}", verdict.text);
    assert!(alpha.heard().contains(&bound()));
}

/// **A name this box holds no channel by is refused, naming what it holds** —
/// and it never falls through to the flat root, which would write a world
/// nobody named.
#[test]
fn a_name_no_channel_answers_to_is_refused_naming_the_held_ones() {
    let scratch = Scratch::new();
    let own = wired(&scratch, &flat(), vec![vec![yes()]]);
    wired(&scratch, &entry("alpha"), vec![vec![yes()]]);
    let verdict = aimed(scratch.path(), Some("beta"), &bound(), Form::Json);
    assert_eq!(verdict.code, 1);
    assert!(
        verdict.text.contains(r#"no channel named "beta""#),
        "{}",
        verdict.text
    );
    assert!(verdict.text.contains(r#""alpha""#), "{}", verdict.text);
    assert!(!own.heard().contains(&bound()));
}
