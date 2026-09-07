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

/// The twenty-six reads, the conversation's fifteen acts, the trail's two, the
/// enrollment, the sign-in, the model assignment, the learning loop's settle,
/// the ball family's three and the wall's own three, spelled out — so the roster is asserted rather than
/// merely iterated, and a verb added or dropped is a diff here.
#[test]
fn the_roster_is_the_verbs_the_seat_can_read_the_answers_to() {
    let words: Vec<&str> = table().iter().map(|verb| verb.word).collect();
    assert_eq!(
        words,
        vec![
            "workspaces",
            "attention",
            "balls",
            "board",
            "search",
            "conversations",
            "workspace-balls",
            "marks",
            "science",
            "work-diff",
            "transcript",
            "follow",
            "agent",
            "steps",
            "step",
            "files",
            "inbox",
            "rail",
            "governing",
            "roles",
            "clients",
            "lineages",
            "proposals",
            "providers",
            "models",
            "login-tail",
            "message",
            "interrupt",
            "nudge",
            "stop",
            "retarget",
            "flag",
            "seen",
            "proposal",
            "answer",
            "revoke",
            "restore",
            "delete-agent",
            "enroll",
            "login",
            "model",
            "pin",
            "unpin",
            "ack",
            "scan",
            "arm",
            "disarm",
            "disband",
            "clear-trail",
            "assign",
            "release",
            "close",
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
        flags: &[],
        summary: "",
        detail: "",
    };
    assert_eq!(made.usage(), "lernie later <one> <two>");
}

/// A word that is no verb is not one, and the table is closed: `ask` and
/// `entries` are the command line's own words, not gestures.
#[test]
fn a_word_that_is_not_a_verb_is_not_found() {
    for word in ["ask", "entries", "help", "invocations", ""] {
        assert_eq!(find(word), None, "{word:?}");
    }
}

/// **A flag is a boolean field spelled as its own word** (bl-9fd1): typed after
/// the parameters, it writes `true` under that name; not typed, it writes
/// nothing at all. Absent is never `false` — this seat does not assert into a
/// field nobody touched.
#[test]
fn a_flag_is_raised_by_its_own_word_and_absent_otherwise() {
    let made = Verb {
        word: "later",
        params: &["one"],
        flags: &["deeply"],
        summary: "",
        detail: "",
    };
    assert_eq!(made.usage(), "lernie later <one> [deeply]");
    assert_eq!(
        made.envelope(vec!["a".to_owned()]),
        Ok(json!({"op": "later", "one": "a"}))
    );
    assert_eq!(
        made.envelope(vec!["a".to_owned(), "deeply".to_owned()]),
        Ok(json!({"op": "later", "one": "a", "deeply": true}))
    );
}

/// **The two ways a tail can be wrong are two sentences.** Too many words is
/// answered by counting; the right number of words and the wrong one is
/// answered by naming the word it takes, because counting says nothing about
/// it.
#[test]
fn a_wrong_tail_is_told_apart_from_a_wrong_count() {
    let made = Verb {
        word: "later",
        params: &["one"],
        flags: &["deeply"],
        summary: "",
        detail: "",
    };
    let counted = made
        .envelope(vec!["a".to_owned(), "deeply".to_owned(), "b".to_owned()])
        .expect_err("three is past the range");
    assert!(
        counted.contains("takes 1 to 2 argument(s) and got 3"),
        "{counted}"
    );
    let named = made
        .envelope(vec!["a".to_owned(), "shallowly".to_owned()])
        .expect_err("that is not the word");
    assert!(named.contains("no word \"shallowly\""), "{named}");
    assert!(named.contains("\"deeply\""), "{named}");
}

/// **The cascade is the flag's one real site** (bl-9fd1), and the bare form is
/// unchanged: a `stop` that raises nothing sends no `children` field, so an
/// engine reads exactly the gesture this seat has always sent.
#[test]
fn the_stop_cascade_is_the_word_children_and_the_bare_form_says_nothing() {
    assert_eq!(
        super::stop("home".to_owned(), "c-1".to_owned(), false),
        json!({"op": "stop", "workspace": "home", "agent": "c-1"})
    );
    assert_eq!(
        super::stop("home".to_owned(), "c-1".to_owned(), true),
        json!({"op": "stop", "workspace": "home", "agent": "c-1", "children": true})
    );
    assert_eq!(
        find("stop").expect("a verb").usage(),
        "lernie stop <workspace> <agent> [children]"
    );
}
