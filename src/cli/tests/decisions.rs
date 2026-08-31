//! **What one invocation decides to DO**: the flags, the typed verbs, the
//! structural doors, the hand-written envelope and help — each read back as a
//! value, which is what earns `src/main.rs` its place as the coverage floor's
//! one exclusion.
//!
//! Split from [`super::refusals`] at the design-time budget on the seam the
//! module already had: this is what a run decides, and that is every way a word
//! can fail to be one of them.

use super::super::verdict::REFUSED;
use super::super::{Decided, run, usage, version};
use super::{argv, asked, fanned, said};
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
///
/// **`enroll` is the one exception and it is asserted rather than skipped**: it
/// decides its own arm, because its reply carries a private key and the reply
/// stream's destination is a terminal's scrollback (`crate::seat::enroll`). The
/// envelope it eventually sends is still this row's — `verbs::tests` pins that
/// — so what differs is where the answer goes and nothing about what crosses.
#[test]
fn every_verb_in_the_table_is_typable() {
    for verb in crate::verbs::table() {
        let mut words = vec![verb.word];
        let filled: Vec<String> = verb.params.iter().map(|p| format!("a-{p}")).collect();
        words.extend(filled.iter().map(String::as_str));
        if verb.word == crate::verbs::ENROLL.word {
            let Decided::Enroll {
                workspace,
                name,
                grade,
            } = run(argv(&words))
            else {
                panic!("`enroll` draws its answer rather than printing it");
            };
            assert_eq!(
                (workspace.as_str(), name.as_str(), grade.as_str()),
                ("a-workspace", "a-name", "a-grade")
            );
            continue;
        }
        // **A verb with no `workspace` parameter has no way to name a
        // channel, so its subject is all of them** (bl-0d54): the envelope is
        // the same row's, and only how many channels it is asked of differs.
        let sent = if verb.addresses_a_workspace() {
            asked(&words)
        } else {
            fanned(&words)
        };
        assert_eq!(sent["op"], json!(verb.word), "{}", verb.word);
    }
}

/// **The shorthand fans; the raw door does not** (bl-0d54). `lernie
/// workspaces` has no way to name a channel, so its subject is every channel
/// this box holds — while `lernie ask` stays the escape hatch for exactly one,
/// which is what `{"op":"workspaces","workspace":"<entry>"}` asks.
#[test]
fn the_roster_word_fans_while_the_hand_written_envelope_does_not() {
    assert_eq!(fanned(&["workspaces"]), json!({"op": "workspaces"}));
    assert_eq!(
        asked(&["ask", r#"{"op":"workspaces"}"#]),
        json!({"op": "workspaces"}),
        "the raw door is one channel's, always"
    );
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
