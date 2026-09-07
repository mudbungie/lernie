//! The three envelopes, and the one list a control paints from.

use serde_json::json;

use super::{ANSWER, CALL, RESTORE, REVOKE, SCOPES, VERDICTS, answer, restore, revoke};
use crate::verbs::find;

/// **Each door builds the envelope the wire requires**, and the verdict is a
/// named string like every other parameter — which is what lets all three be
/// rows in the one table.
#[test]
fn each_door_builds_the_envelope_the_wire_requires() {
    assert_eq!(
        answer("home".to_owned(), "c-1".to_owned(), "pass".to_owned(), None),
        json!({"op": "answer", "workspace": "home", "agent": "c-1", "verdict": "pass",
               "scope": "call"})
    );
    assert_eq!(
        revoke("home".to_owned(), "c-1".to_owned()),
        json!({"op": "revoke", "workspace": "home", "agent": "c-1"})
    );
    assert_eq!(
        restore("home".to_owned(), "c-1".to_owned()),
        json!({"op": "restore", "workspace": "home", "agent": "c-1"})
    );
}

/// All three are rows in the one table `lernie help` prints.
#[test]
fn all_three_are_rows_in_the_one_table() {
    for verb in [ANSWER, REVOKE, RESTORE] {
        assert_eq!(find(verb.word), Some(verb), "{}", verb.word);
    }
}

/// **The words a control paints are the words the envelope carries**, so no
/// translation table exists to drift — the tuning family's own rule.
#[test]
fn the_verdicts_are_the_wires_own_words() {
    assert_eq!(VERDICTS, ["pass", "refuse", "hold"]);
    for word in VERDICTS {
        let built = answer("home".to_owned(), "c-1".to_owned(), word.to_owned(), None);
        assert_eq!(built["verdict"], word);
    }
}

/// **The scope is stated on every gesture, `call` included** (PROTOCOL 18).
/// `None` is the operator not typing one, which is a different thing from the
/// field being optional: the wire requires it in both directions, because an
/// absent field would let the two ends disagree about how wide the instruction
/// was and *wider* is the reading nobody may arrive at by accident.
#[test]
fn every_answer_states_its_reach_and_an_untyped_one_states_the_narrow_word() {
    assert_eq!(SCOPES, ["call", "conversation", "workspace"]);
    assert_eq!(SCOPES[0], CALL);
    let unstated = answer("home".to_owned(), "c-1".to_owned(), "pass".to_owned(), None);
    assert_eq!(unstated["scope"], CALL);
    for scope in SCOPES {
        let built = answer(
            "home".to_owned(),
            "c-1".to_owned(),
            "pass".to_owned(),
            Some(scope.to_owned()),
        );
        assert_eq!(built["scope"], scope);
    }
}
