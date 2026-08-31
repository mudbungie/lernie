//! **Every way a word can fail to be one this binary answers** — a wrong
//! arity, a body that is not a gesture, and a word that is nothing at all.
//!
//! Split from [`super::decisions`] at the design-time budget on the seam that
//! module's own doc drew. Every refusal here is a value a test reads back, and
//! every one of them names what was refused and what could have been typed
//! instead — a bare non-zero exit teaches nobody anything.

use super::super::Stream;
use super::super::verdict::REFUSED;
use super::said;

/// **The enrollment's arity is exact too**, and a wrong one earns the row's own
/// usage rather than the arm's silence — the same grammar every other verb
/// teaches from its mistake.
#[test]
fn a_misspelled_enrollment_earns_the_row_s_usage() {
    let said = said(&["enroll", "home"]);
    assert_eq!(said.code, 2);
    assert!(
        said.text.contains(&crate::verbs::ENROLL.usage()),
        "{}",
        said.text
    );
}

/// **Its arity is exact, for the verb table's own reason**: argv quotes, so a
/// goal is one argument and an unquoted tail refuses rather than being silently
/// joined — which would make three typed words indistinguishable from one
/// quoted sentence.
#[test]
fn the_start_word_refuses_by_arity_and_says_what_it_takes() {
    for words in [
        vec!["start"],
        vec!["start", "home"],
        vec!["start", "home", "do", "the", "thing"],
    ] {
        let v = said(&words);
        assert_eq!(v.code, REFUSED, "{words:?}");
        assert_eq!(v.stream, Stream::Err);
        assert!(
            v.text.contains("lernie start <workspace> <goal>"),
            "{}",
            v.text
        );
    }
}

/// **A verb's own refusal names the verb**, where a word that is no verb can
/// only be quoted back — two different mistakes, and the more specific one gets
/// the more specific answer.
#[test]
fn a_verb_with_the_wrong_arity_refuses_by_name_and_teaches_the_grammar() {
    let v = said(&["transcript", "home"]);
    assert_eq!(v.code, REFUSED);
    assert!(v.text.contains("`lernie transcript`"), "{}", v.text);
    assert!(
        v.text
            .contains("usage: lernie transcript <workspace> <agent>"),
        "{}",
        v.text
    );
}

/// **A body that is not a gesture is the CALLER's typo**, so it is decided
/// here — in the pure function, where the refusal is a value a test reads back
/// — and it earns the usage rather than a connection.
#[test]
fn a_body_that_is_not_a_gesture_is_refused_with_the_usage() {
    for (body, said_what) in [
        ("not json at all", "not JSON"),
        ("[1,2]", "a gesture is a JSON object"),
        (r#"{"workspace":"home"}"#, "missing field"),
    ] {
        let v = said(&["ask", body]);
        assert_eq!(v.code, REFUSED, "{body}");
        assert_eq!(v.stream, Stream::Err);
        assert!(v.text.contains(said_what), "{body}: {}", v.text);
        assert!(v.text.contains("usage: lernie"), "{body}: {}", v.text);
    }
}

/// **A door with the wrong number of arguments refuses like a verb does**
/// (bl-6bda): the word is real and the usage lists it, so what it earns is its
/// own grammar rather than the sentence a typo gets.
#[test]
fn a_door_with_the_wrong_arity_refuses_by_name_and_teaches_the_grammar() {
    for (words, expected) in [
        (
            vec!["ask"],
            "`lernie ask` takes 1 argument(s) and got 0 — usage: lernie ask <envelope>",
        ),
        (
            vec!["ask", "{}", "twice"],
            "`lernie ask` takes 1 argument(s) and got 2 — usage: lernie ask <envelope>",
        ),
        (
            vec!["entries", "--now"],
            "`lernie entries` takes 0 argument(s) and got 1 — usage: lernie entries",
        ),
        (
            vec!["help", "a", "b"],
            "`lernie help` takes at most 1 argument(s) and got 2 — usage: lernie help [<verb>]",
        ),
    ] {
        let v = said(&words);
        assert_eq!(v.code, REFUSED, "{words:?}");
        assert!(v.text.contains(expected), "{words:?}:\n{}", v.text);
    }
}

#[test]
fn an_unrecognised_argument_refuses_and_quotes_every_word_of_it() {
    let v = said(&["seat", "--ws", "Example"]);
    assert_eq!(v.code, REFUSED);
    assert!(
        v.text.contains("unrecognised argument: seat --ws Example"),
        "{}",
        v.text
    );
}

/// The match is on the WHOLE argument list, not on a first word, so a word that
/// would succeed alone refuses when something rides behind it — rather than
/// silently ignoring the rest.
#[test]
fn a_recognised_word_with_extra_words_is_not_recognised() {
    let extra = ["--version", "--now"];
    let v = said(&extra);
    assert_eq!(v.code, REFUSED);
    assert!(
        v.text
            .contains(&format!("unrecognised argument: {}", extra.join(" "))),
        "{}",
        v.text
    );
}
