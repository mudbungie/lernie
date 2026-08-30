//! **What one invocation decides**: the flags, the typed verbs, the
//! hand-written envelope, help, and every way a word can fail to be one of
//! them — each read back as a value, which is what earns `src/main.rs` its
//! place as the coverage floor's one exclusion.

use super::super::verdict::REFUSED;
use super::super::{Decided, Stream, run, usage, version};
use super::{argv, asked, said};
use serde_json::json;

#[test]
fn both_version_spellings_print_the_version_and_succeed() {
    for spelling in ["--version", "-V"] {
        let v = said(&[spelling]);
        assert_eq!(v.code, 0, "{spelling} did not succeed");
        assert_eq!(v.text, version(), "{spelling} printed something else");
    }
}

/// The verbs decide, and say nothing: what they do needs this process's own
/// environment, which is the entry point's to fold.
#[test]
fn the_verbs_decide_and_carry_what_they_were_given() {
    assert!(matches!(run(argv(&["entries"])), Decided::Entries));
    assert_eq!(
        asked(&["ask", r#"{"op":"workspaces"}"#]),
        json!({"op": "workspaces"})
    );
}

/// **A typed verb and a hand-written envelope arrive as one value.** That is
/// the whole property: the verbs are a serialization of the envelope, not a
/// second spelling of a gesture, so what leaves this function is identical
/// either way and only one thing is ever routed.
#[test]
fn a_typed_verb_and_the_envelope_it_stands_for_decide_alike() {
    let typed = asked(&["conversations", "home"]);
    let written = asked(&["ask", r#"{"op":"conversations","workspace":"home"}"#]);
    assert_eq!(typed, written);
    assert_eq!(typed, json!({"op": "conversations", "workspace": "home"}));
}

/// Every verb in the table is reachable from argv, with the arguments in the
/// order its usage states.
#[test]
fn every_verb_in_the_table_is_typable() {
    for verb in crate::verbs::table() {
        let mut words = vec![verb.word];
        let filled: Vec<String> = verb.params.iter().map(|p| format!("a-{p}")).collect();
        words.extend(filled.iter().map(String::as_str));
        assert_eq!(asked(&words)["op"], json!(verb.word), "{}", verb.word);
    }
}

/// **The composite is one word and two words of argument.** It carries them
/// rather than an envelope, because there are two envelopes and the second
/// cannot be built until the first is answered.
#[test]
fn the_start_word_carries_a_workspace_and_a_goal() {
    let Decided::Start { address, goal } = run(argv(&["start", "home", "do the thing"])) else {
        panic!("`start` begins a conversation");
    };
    assert_eq!(address, "home");
    assert_eq!(goal, "do the thing");
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

/// **One help, three spellings**, because the subject is one — this binary's
/// interface. It answers with no engine up and no channel provisioned, which is
/// the property that makes it answerable here at all.
#[test]
fn every_help_spelling_prints_the_one_usage() {
    for spelling in ["help", "--help", "-h"] {
        let v = said(&[spelling]);
        assert_eq!(v.code, 0, "{spelling} did not succeed");
        assert_eq!(v.text, usage(), "{spelling} printed something else");
    }
}

/// One verb's page, and a word that is not a verb.
#[test]
fn help_on_a_verb_answers_its_page_and_refuses_a_word_that_is_not_one() {
    let page = said(&["help", "nudge"]);
    assert_eq!(page.code, 0);
    assert!(
        page.text.starts_with("usage: lernie nudge"),
        "{}",
        page.text
    );
    let refusal = said(&["help", "nudje"]);
    assert_eq!(refusal.code, REFUSED);
    assert!(refusal.text.contains("\"nudje\""), "{}", refusal.text);
}

/// **A bare invocation is the window**, because a seat is a window. Every other
/// spelling is a way of reaching one gesture without one.
#[test]
fn a_bare_invocation_opens_the_window() {
    assert!(matches!(run(argv(&[])), Decided::Window));
}

#[test]
fn ask_with_no_envelope_says_what_it_wanted() {
    let v = said(&["ask"]);
    assert_eq!(v.code, REFUSED);
    assert!(v.text.contains("wants one gesture envelope"), "{}", v.text);
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
    for extra in [["--version", "--now"], ["entries", "--now"]] {
        let v = said(&extra);
        assert_eq!(v.code, REFUSED);
        assert!(
            v.text
                .contains(&format!("unrecognised argument: {}", extra.join(" "))),
            "{}",
            v.text
        );
    }
    let v = said(&["ask", "{}", "twice"]);
    assert_eq!(v.code, REFUSED);
}
