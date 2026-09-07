//! **`answer`'s grammar**, read back as values — the reach a run states, and
//! the refusal a word that is none of the three earns.
//!
//! Its own file beside [`super::super::answer`] for that module's own reason:
//! a verb whose grammar is more than words is its own file, and so are its
//! beats.

use super::{asked, said};
use crate::verbs::capability::{CALL, SCOPES};
use serde_json::json;

/// **An answer with no reach typed states the narrow one** (PROTOCOL 18). The
/// wire requires `scope` in both directions, so leaving it off is a fact about
/// the operator's typing and never about the envelope — `ops`' own shape,
/// where the wire demands a depth and the word supplies one.
#[test]
fn an_answer_with_no_reach_typed_still_states_the_narrow_one() {
    assert_eq!(
        asked(&["answer", "home", "c-1", "pass"]),
        json!({"op": "answer", "workspace": "home", "agent": "c-1",
               "verdict": "pass", "scope": CALL})
    );
}

/// **Each of the three is stated as the word it is** — the wire's own
/// vocabulary, with no translation table to drift.
#[test]
fn each_reach_rides_the_gesture_as_the_word_that_was_typed() {
    for scope in SCOPES {
        assert_eq!(
            asked(&["answer", "home", "c-1", "refuse", scope])["scope"],
            json!(scope)
        );
    }
}

/// **A misspelled reach is read here and costs no connection**, exactly as
/// `enroll`'s grade is: it is a closed set of three the boundary defines and
/// this binary holds, so the mistake is the caller's typo and the refusal
/// names all three words that would have worked. Falling through would spend a
/// round trip to be told `unknown scope "conversaton"` — true, and naming
/// none of them.
#[test]
fn a_reach_that_is_none_of_the_three_refuses_here_and_names_them() {
    let refusal = said(&["answer", "home", "c-1", "pass", "conversaton"]);
    assert!(refusal.text.contains("conversaton"), "{}", refusal.text);
    for scope in SCOPES {
        assert!(
            refusal.text.contains(scope),
            "{scope} unnamed in {}",
            refusal.text
        );
    }
    assert_eq!(refusal.code, super::super::verdict::REFUSED);
}
