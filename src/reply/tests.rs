//! The envelope's three outcomes, and the policy rungs that are the
//! envelope's own. Each type's fields are tested beside that type.

use super::{Outcome, Read, Reply, read};
use serde_json::json;

/// The corpus replay — every frame in `corpus/`, through the real reader.
mod corpus;

/// The answer path, end to end: a frame with a kind this build paints becomes
/// the typed value the window draws.
#[test]
fn a_kind_this_build_paints_becomes_the_typed_answer() {
    assert_eq!(
        read(&json!({"ok": true, "kind": "nudged"})),
        Read::Answer(Reply::Nudged)
    );
}

/// **The kind-less envelope is the refusal, and only a refusal may wear it.**
/// `ok` cannot be the discriminant — a captured run spells its own verdict
/// there — so a body carrying no kind must be the refusal shape, and one that
/// claims success while saying nothing is an answer that failed to say what it
/// answers.
#[test]
fn the_kindless_envelope_is_the_refusal_and_nothing_else() {
    assert_eq!(
        read(&json!({"ok": false, "error": "unknown workspace \"hoem\""})),
        Read::Refusal("unknown workspace \"hoem\"".to_owned())
    );
    let Read::Unreadable(why) = read(&json!({"ok": true, "rows": []})) else {
        panic!("an answer with no kind is not readable");
    };
    assert!(why.contains("an answer with no"), "{why}");
}

/// A refusal that does not say why, and one that does not say whether: both
/// are shapes rung 1 refuses by name, because a seat cannot paint a verdict it
/// was not given.
#[test]
fn a_refusal_missing_its_own_fields_is_unreadable_by_name() {
    for (frame, named) in [
        (json!({"ok": false}), "\"error\""),
        (json!({"error": "no"}), "\"ok\""),
    ] {
        let Read::Unreadable(why) = read(&frame) else {
            panic!("{frame} is not readable");
        };
        assert!(why.contains(named), "{frame}: {why}");
    }
}

/// **The captured run reads its verdict off the exit code and off nothing
/// else.** The `ok` beside it is the envelope's field, and a second copy of
/// one fact could only ever disagree with it — this is the frame where the two
/// deliberately do.
#[test]
fn a_captured_run_takes_its_verdict_from_the_exit_code() {
    let frame = json!({"ok": true, "kind": "outcome", "exit": 1,
                       "stdout": "", "stderr": "the gate said no\n"});
    let Read::Answer(Reply::Outcome(outcome)) = read(&frame) else {
        panic!("a captured run");
    };
    assert!(!outcome.ok(), "exit 1 is not ok, whatever `ok` said");
    assert_eq!(outcome.stderr, "the gate said no\n");
    assert!(
        Outcome {
            exit: 0,
            stdout: String::new(),
            stderr: String::new(),
        }
        .ok()
    );
}

/// An exit status no `i32` holds is an engine saying something this seat has
/// no way to paint — the one narrowing on this surface that is a real check.
#[test]
fn an_exit_status_outside_its_own_type_is_unreadable() {
    let frame = json!({"ok": true, "kind": "outcome", "exit": 9_999_999_999_i64,
                       "stdout": "", "stderr": ""});
    let Read::Unreadable(why) = read(&frame) else {
        panic!("out of range");
    };
    assert!(why.contains("out of range"), "{why}");
}

/// **Rung 2**: a kind this build does not paint refuses, names itself, and
/// says what to do about it — the refusal is the upgrade prompt, exactly as
/// the version preface's mismatch is.
#[test]
fn a_kind_this_build_does_not_paint_refuses_by_name_with_a_remedy() {
    let Read::Unreadable(why) = read(&json!({"ok": true, "kind": "board", "rows": []})) else {
        panic!("an unpainted kind");
    };
    assert!(why.contains("\"board\""), "{why}");
    assert!(why.contains("upgrade"), "{why}");
}

/// **Rung 1**, at the envelope: bytes that are not a reply object at all, and
/// a discriminant that is not a word.
#[test]
fn a_frame_that_is_not_a_reply_object_is_unreadable() {
    for (frame, said) in [
        (json!(["workspaces", []]), "not a JSON object"),
        (json!("workspaces"), "not a JSON object"),
        (json!({"ok": true, "kind": 7}), "non-string field"),
    ] {
        let Read::Unreadable(why) = read(&frame) else {
            panic!("{frame} is not readable");
        };
        assert!(why.contains(said), "{frame}: {why}");
    }
}

/// **Nothing that arrives is a panic path.** The reader is total: every frame,
/// however hostile, lands on one of the three arms. This is the property the
/// whole module is written for, so it is asserted over a table rather than
/// inferred from the arms above.
#[test]
fn every_frame_lands_on_one_of_the_three_arms() {
    for frame in [
        json!(null),
        json!(0),
        json!({}),
        json!({"ok": "yes"}),
        json!({"kind": "workspaces"}),
        json!({"ok": true, "kind": "workspaces"}),
        json!({"ok": true, "kind": "workspaces", "rows": {}}),
        json!({"ok": true, "kind": "workspaces", "rows": [7]}),
        json!({"ok": true, "kind": "transcript", "rows": [{}]}),
        json!({"ok": true, "kind": "follow"}),
        json!({"ok": true, "kind": "conversations", "rows": [[]]}),
    ] {
        let answered = read(&frame);
        assert!(
            matches!(
                answered,
                Read::Answer(_) | Read::Refusal(_) | Read::Unreadable(_)
            ),
            "{frame}: {answered:?}"
        );
    }
}
