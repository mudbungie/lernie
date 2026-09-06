//! **What one invocation decides to DO**: the flags, the typed verbs, the
//! structural doors, the hand-written envelope and help — each read back as a
//! value, which is what earns `src/main.rs` its place as the coverage floor's
//! one exclusion.
//!
//! Split from [`super::refusals`] at the design-time budget on the seam the
//! module already had: this is what a run decides, and that is every way a word
//! can fail to be one of them.

/// The words that are not one ask: the composites, and the drawn answer.
mod composites;

use super::super::verdict::REFUSED;
use super::super::{Decided, run, usage, version};
use super::{argv, asked, fanned, said};
use crate::render::Form;
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
        // **`grade` is the one parameter this binary reads itself** (bl-07b9),
        // so it is filled with a word that is one rather than with the
        // placeholder every other field takes.
        let filled: Vec<String> = verb
            .params
            .iter()
            .map(|p| {
                if *p == "grade" {
                    crate::ui::Grade::default().word()
                } else {
                    format!("a-{p}")
                }
            })
            .collect();
        words.extend(filled.iter().map(String::as_str));
        if verb.word == crate::verbs::ENROLL.word {
            let Decided::Enroll {
                workspace,
                name,
                grade,
                into,
            } = run(argv(&words))
            else {
                panic!("`enroll` renders its answer rather than printing a frame");
            };
            assert_eq!(
                (workspace.as_str(), name.as_str(), grade.as_str()),
                (
                    "a-workspace",
                    "a-name",
                    crate::ui::Grade::default().word().as_str()
                )
            );
            // **The three words alone write nothing**, which is what makes the
            // destination the operator's choice rather than this seat's.
            assert_eq!(into, None);
            continue;
        }
        // **`follow` is the other row whose word is not one ask** (bl-f076):
        // the gesture is the row's, and the word spends it until the
        // conversation comes to rest.
        if verb.word == crate::verbs::FOLLOW.word {
            let Decided::Follow { .. } = run(argv(&words)) else {
                panic!("`follow` watches rather than asking once");
            };
            continue;
        }
        // **`model` reads before it writes** (bl-1e5a): the assignment is this
        // row's own envelope, and what the word adds ahead of it is the
        // `models` read that says whether the id is one the provider offers.
        if verb.word == crate::verbs::MODEL.word {
            let Decided::Model { .. } = run(argv(&words)) else {
                panic!("`model` checks before it assigns");
            };
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

/// **`--json` is read off the front and changes only the FORM**, never the
/// envelope: the same word with and without it decides the same gesture.
#[test]
fn the_json_flag_leads_and_leaves_the_gesture_exactly_as_it_was() {
    let Decided::Fanned(bare, Form::Rendered) = run(argv(&["workspaces"])) else {
        panic!("a bare read renders");
    };
    let Decided::Fanned(flagged, Form::Json) = run(argv(&["--json", "workspaces"])) else {
        panic!("a flagged read is the machine form");
    };
    assert_eq!(bare, flagged);
    let Decided::Ask(_, Form::Json) = run(argv(&["--json", "conversations", "home"])) else {
        panic!("a workspace read carries the form too");
    };
    let Decided::Start { form, .. } = run(argv(&["--json", "start", "home", "go"])) else {
        panic!("the composite carries it");
    };
    assert_eq!(form, Form::Json);
    let Decided::Ask(_, Form::Json) = run(argv(&["--json", "ask", "{\"op\":\"workspaces\"}"]))
    else {
        panic!("the raw door carries it");
    };
}

/// **A parameter that happens to be the flag is still the parameter.** Reading
/// it anywhere but the front would make `lernie message` unable to send seven
/// particular characters, which is the whole reason the position is fixed.
#[test]
fn the_flag_after_the_word_is_the_operators_own_text() {
    assert_eq!(
        asked(&["message", "home", "Pelican", "--json"]),
        json!({"op": "message", "workspace": "home", "agent": "Pelican",
               "content": "--json"})
    );
}

/// **The cascade is a word after the address** (bl-9fd1), and it is the only
/// stop that works on a conversation with children: the bare form kills the
/// root's driver, a child's driver deposits into it, and it is running again
/// seconds later.
#[test]
fn the_stop_cascade_is_typed_as_the_word_children() {
    assert_eq!(
        asked(&["stop", "home", "Pelican"]),
        json!({"op": "stop", "workspace": "home", "agent": "Pelican"})
    );
    assert_eq!(
        asked(&["stop", "home", "Pelican", "children"]),
        json!({"op": "stop", "workspace": "home", "agent": "Pelican", "children": true})
    );
}
