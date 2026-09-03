//! The table as data: what each row becomes, and the arity that is exact.

/// The wire conformance corpus, request direction.
mod corpus;

use super::{Verb, find, table};
use crate::envelope::OP;
use serde_json::json;

/// **The verb IS the op.** One fact, not two, so a row cannot name a word an
/// operator types and a different one the engine reads.
#[test]
fn every_verb_builds_an_envelope_whose_op_is_its_own_word() {
    for verb in table() {
        let args = verb
            .params
            .iter()
            .map(|p| format!("a-{p}"))
            .collect::<Vec<String>>();
        let built = verb.envelope(args).expect("the right arity");
        assert_eq!(built[OP], json!(verb.word), "{}", verb.word);
        for param in verb.params {
            assert_eq!(built[*param], json!(format!("a-{param}")), "{}", verb.word);
        }
    }
}

/// The nine reads, the conversation's eight acts, the enrollment, the model
/// assignment and the wall's own unmaking, spelled out — so the roster is asserted rather than merely
/// iterated, and a verb added or dropped is a diff here.
#[test]
fn the_roster_is_the_verbs_the_seat_can_read_the_answers_to() {
    let words: Vec<&str> = table().iter().map(|verb| verb.word).collect();
    assert_eq!(
        words,
        vec![
            "workspaces",
            "attention",
            "search",
            "conversations",
            "transcript",
            "follow",
            "steps",
            "files",
            "roles",
            "message",
            "interrupt",
            "nudge",
            "stop",
            "retarget",
            "flag",
            "seen",
            "delete-agent",
            "enroll",
            "model",
            "delete-workspace"
        ]
    );
}

/// The envelope a typed verb becomes, whole, beside the one an operator would
/// have written by hand. They are the same object — which is the property this
/// module exists for, and the only way it can be shown is by writing both.
#[test]
fn a_typed_verb_and_a_hand_written_envelope_are_one_object() {
    let verb = find("message").expect("a verb");
    let built = verb
        .envelope(
            ["home", "20260830T051200Z-a1b2", "ship it"]
                .iter()
                .map(|w| (*w).to_owned())
                .collect(),
        )
        .expect("three arguments");
    assert_eq!(
        built,
        json!({"op": "message", "workspace": "home",
               "agent": "20260830T051200Z-a1b2", "content": "ship it"})
    );
}

/// **Arity is exact, and the refusal teaches the grammar.** A verbatim payload
/// is one argument here because argv has quoting where a line does not, so an
/// unquoted tail must refuse rather than be silently joined.
#[test]
fn a_wrong_arity_refuses_naming_the_verb_and_its_usage() {
    let verb = find("message").expect("a verb");
    for count in [0, 2, 4] {
        let refusal = verb
            .envelope((0..count).map(|n| n.to_string()).collect())
            .expect_err("the wrong arity");
        assert!(refusal.contains("`lernie message`"), "{refusal}");
        assert!(refusal.contains("takes 3 argument(s)"), "{refusal}");
        assert!(
            refusal.contains("usage: lernie message <workspace> <agent> <content>"),
            "{refusal}"
        );
    }
}

/// A verb with no parameters takes none, and says so when handed one.
#[test]
fn the_bare_read_takes_nothing() {
    let verb = find("workspaces").expect("a verb");
    assert_eq!(verb.envelope(Vec::new()), Ok(json!({"op": "workspaces"})));
    assert!(verb.envelope(vec!["home".to_owned()]).is_err());
}

/// The usage line is computed from the row, so a parameter added cannot leave
/// a stored line behind saying otherwise.
#[test]
fn a_usage_line_is_derived_from_the_word_and_its_parameters() {
    assert_eq!(
        find("transcript").expect("a verb").usage(),
        "lernie transcript <workspace> <agent>"
    );
    assert_eq!(
        find("workspaces").expect("a verb").usage(),
        "lernie workspaces"
    );
    let made = Verb {
        word: "later",
        params: &["one", "two"],
        summary: "",
        detail: "",
    };
    assert_eq!(made.usage(), "lernie later <one> <two>");
}

/// A word that is no verb is not one, and the table is closed: `ask` and
/// `entries` are the command line's own words, not gestures.
#[test]
fn a_word_that_is_not_a_verb_is_not_found() {
    for word in ["ask", "entries", "help", "board", ""] {
        assert_eq!(find(word), None, "{word:?}");
    }
}
