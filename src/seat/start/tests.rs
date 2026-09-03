//! The composite: two acts down two connections, what the engine is handed on
//! each, and the four ways it can end.

use super::{INDOUBT, UNFIRED, start};
use crate::channel::entries::WORKSPACE;
use crate::cli::Stream;
use crate::test_support::Scratch;
use crate::test_support::engine::Answer;
use crate::test_support::wire::{entry, flat, wired};
use serde_json::{Value, json};

/// The staging receipt an engine answers, with two fields this build does not
/// read so the carry can be asserted on the wire itself.
fn staged(workspace: &str) -> Value {
    json!({"ok": true, "kind": "prepared",
           "prepared": {"workspace": workspace, "goal": "",
                        "lineage": "reviewer", "origin": "world"}})
}

/// The fire's receipt.
fn started(name: &str) -> Value {
    json!({"ok": true, "kind": "started", "conversation": name})
}

/// **The whole flow, end to end against an engine that speaks the protocol.**
/// Two connections, two envelopes, and the second carries the first's answer
/// straight back — the staged body whole, including the two fields nothing here
/// paints.
#[test]
fn a_start_stages_then_fires_and_the_second_act_carries_the_first_s_answer() {
    let scratch = Scratch::new();
    let engine = wired(
        &scratch,
        &flat(),
        vec![vec![staged("home")], vec![started("brisk-otter")]],
    );
    let verdict = start(scratch.path(), "home", "do the thing");
    assert_eq!(verdict.code, 0);
    assert_eq!(verdict.stream, Stream::Out);
    assert_eq!(
        verdict.text,
        format!("{}\n{}", staged("home"), started("brisk-otter")),
        "both streams are the product"
    );
    let heard = engine.heard();
    assert!(
        heard.contains(&json!({"op": "prepare", "workspace": "home",
                               "payload": {"rung": "bare"}})),
        "{heard:?}"
    );
    assert!(
        heard.contains(
            &json!({"op": "prompt", "goal": "do the thing", "seed": null,
                               "prepared": {"workspace": "home", "goal": "",
                                            "lineage": "reviewer", "origin": "world"}})
        ),
        "{heard:?}"
    );
}

/// **The fire reaches the engine that staged it, under a local rename.** The
/// staged body comes back in the host's spelling; the fire is composed in this
/// box's, and §8.2's one mapping rewrites it again on the way out — so the host
/// is handed its own name both times.
#[test]
fn a_renamed_entry_stages_and_fires_down_the_same_channel() {
    let scratch = Scratch::new();
    let engine = wired(
        &scratch,
        &entry("home"),
        vec![vec![staged("personal")], vec![started("brisk-otter")]],
    );
    std::fs::write(
        scratch.path().join(entry("home")).join(WORKSPACE),
        "personal",
    )
    .expect("the workspace file");
    assert_eq!(start(scratch.path(), "home", "do it").code, 0);
    let heard = engine.heard();
    assert!(
        heard.contains(&json!({"op": "prepare", "workspace": "personal",
                               "payload": {"rung": "bare"}})),
        "{heard:?}"
    );
    assert!(
        heard.iter().any(|said| said["op"] == json!("prompt")
            && said["prepared"]["workspace"] == json!("personal")),
        "{heard:?}"
    );
}

/// **A stage that answers no staged body fires nothing and exits non-zero**,
/// whatever it answered: a refusal, a kind this build cannot paint, or an
/// engine that terminated the stream saying nothing. Its frames are still the
/// engine answering, so they are still the product — and only the code says the
/// start did not happen.
#[test]
fn a_stage_that_answers_no_body_prints_what_it_said_and_starts_nothing() {
    for (answer, expected) in [
        (
            vec![json!({"ok": false, "error": "unknown workspace \"hoem\""})],
            json!({"ok": false, "error": "unknown workspace \"hoem\""}).to_string(),
        ),
        (
            vec![json!({"ok": true, "kind": "board", "rows": []})],
            json!({"ok": true, "kind": "board", "rows": []}).to_string(),
        ),
        (Vec::new(), String::new()),
    ] {
        let scratch = Scratch::new();
        let engine = wired(&scratch, &flat(), vec![answer]);
        let verdict = start(scratch.path(), "home", "do it");
        assert_eq!(verdict.code, 1, "{}", verdict.text);
        assert_eq!(verdict.stream, Stream::Out);
        assert_eq!(verdict.text, expected);
        assert!(
            !engine
                .heard()
                .iter()
                .any(|said| said["op"] == json!("prompt")),
            "nothing is fired: {:?}",
            engine.heard()
        );
    }
}

/// **A fire the engine refuses is an answer**, so both streams print and the
/// exit code is the fire's own verdict.
#[test]
fn the_exit_code_is_the_fire_s_verdict() {
    let scratch = Scratch::new();
    let refused = json!({"ok": false, "error": "no models on this wall"});
    wired(
        &scratch,
        &flat(),
        vec![vec![staged("home")], vec![refused.clone()]],
    );
    let verdict = start(scratch.path(), "home", "do it");
    assert_eq!(verdict.code, 1);
    assert_eq!(verdict.text, format!("{}\n{refused}", staged("home")));
}

/// **A stage that landed and a fire that could not be sent is its own
/// sentence** — the one outcome two acts have and one does not. The workspace
/// exists and nothing is running, and saying so beats a transport error printed
/// under a receipt that looks like success.
#[test]
fn a_fire_that_never_leaves_the_box_says_the_start_was_staged() {
    let scratch = Scratch::new();
    // One connection in the script: the stage takes it, and the fire's dial
    // finds a listener that has stopped accepting.
    wired(&scratch, &flat(), vec![vec![staged("home")]]);
    let verdict = start(scratch.path(), "home", "do it");
    assert_eq!(verdict.code, 1);
    assert_eq!(verdict.stream, Stream::Err);
    assert!(verdict.text.contains(UNFIRED), "{}", verdict.text);
}

/// **A fire that CROSSED and was not answered says the opposite thing**
/// (REMOTE §3, bl-3969), because the remedy is opposite. The stage's steps are
/// convergent, so *type it again* is right for a fire that never left; a fire
/// the engine may already have run makes a second `lernie start` a second
/// conversation, so the sentence refuses the repeat by name and hands over the
/// read instead.
#[test]
fn a_fire_that_crossed_with_no_answer_refuses_the_retype_and_names_the_read() {
    let scratch = Scratch::new();
    wired(
        &scratch,
        &flat(),
        vec![Answer::Frames(vec![staged("home")]), Answer::Hangup],
    );
    let verdict = start(scratch.path(), "home", "do it");
    assert_eq!(verdict.code, 1);
    assert_eq!(verdict.stream, Stream::Err);
    assert!(verdict.text.contains(INDOUBT), "{}", verdict.text);
    assert!(
        !verdict.text.contains(UNFIRED),
        "an act that crossed did not fail to be sent: {}",
        verdict.text
    );
    assert!(
        verdict.text.contains("lernie conversations"),
        "the recovery is a read, and it is named: {}",
        verdict.text
    );
}

/// A box with nothing provisioned never stages at all, and says so in the words
/// the flat root already has.
#[test]
fn a_box_with_no_channel_says_so_before_it_stages_anything() {
    let scratch = Scratch::new();
    let verdict = start(scratch.path(), "home", "do it");
    assert_eq!(verdict.code, 1);
    assert_eq!(verdict.stream, Stream::Err);
    assert!(
        verdict.text.contains("no wire provisioned"),
        "{}",
        verdict.text
    );
}
