//! A gesture that names no workspace, asked of every channel this box holds.

use super::fanned;
use crate::channel::entries::WORKSPACE;
use crate::cli::Stream;
use crate::test_support::wire::{entry, flat, wired, yes};
use crate::test_support::{Scratch, mint};
use serde_json::json;

/// **The union, stamped**: this box's own engine first, then the entries in
/// leaf order, each answer under the name of the channel it came from — the
/// same composition the window's roster is, so the two surfaces agree on one
/// box (bl-0d54).
#[test]
fn every_channel_answers_under_its_own_name() {
    let scratch = Scratch::new();
    let own = wired(&scratch, &flat(), vec![vec![yes()]]);
    let alpha = wired(
        &scratch,
        &entry("alpha"),
        vec![vec![json!({"ok": true, "n": 1})]],
    );

    let verdict = fanned(scratch.path(), &json!({"op": "workspaces"}));
    assert_eq!(verdict.code, 0);
    assert_eq!(verdict.stream, Stream::Out);
    assert_eq!(
        verdict.text,
        format!(
            "(this box's own engine)\n    {}\nalpha\n    {}",
            yes(),
            json!({"ok": true, "n": 1})
        )
    );
    assert!(own.heard().contains(&json!({"op": "workspaces"})));
    assert!(alpha.heard().contains(&json!({"op": "workspaces"})));
}

/// **The sharp case**: a box that holds no engine of its own and is a client of
/// a server elsewhere. It has channels, it has workspaces — and the verb whose
/// whole subject is "every workspace" used to tell the operator nothing was
/// provisioned.
#[test]
fn a_box_with_no_flat_channel_still_answers_from_its_entries() {
    let scratch = Scratch::new();
    wired(&scratch, &entry("alpha"), vec![vec![yes()]]);
    let verdict = fanned(scratch.path(), &json!({"op": "workspaces"}));
    assert_eq!(verdict.code, 0, "{}", verdict.text);
    assert!(
        verdict.text.contains(&yes().to_string()),
        "{}",
        verdict.text
    );
}

/// **The hole is still SAID.** A flat root that holds nothing is a real gap on
/// a box that is supposed to hold one, so it keeps its section and its own
/// sentence beside the entries that answered.
#[test]
fn a_channel_that_cannot_answer_says_so_beside_the_ones_that_did() {
    let scratch = Scratch::new();
    wired(&scratch, &entry("alpha"), vec![vec![yes()]]);
    let verdict = fanned(scratch.path(), &json!({"op": "workspaces"}));
    let lines: Vec<&str> = verdict.text.lines().collect();
    assert_eq!(lines[0], "(this box's own engine)");
    assert!(lines[1].contains("no wire provisioned at"), "{}", lines[1]);
    assert_eq!(lines[2], "alpha");
}

/// A channel that renames carries the host's spelling in its stamp, exactly as
/// the listing prints it — one home for the label, so the two cannot disagree.
#[test]
fn a_renamed_entry_is_stamped_the_way_the_listing_names_it() {
    let scratch = Scratch::new();
    wired(&scratch, &entry("alpha"), vec![vec![yes()]]);
    std::fs::write(
        scratch.path().join(entry("alpha")).join(WORKSPACE),
        "personal",
    )
    .expect("the workspace file");
    let verdict = fanned(scratch.path(), &json!({"op": "workspaces"}));
    assert!(
        verdict
            .text
            .contains(r#"alpha (named "personal" on its host)"#),
        "{}",
        verdict.text
    );
}

/// **A fan that learned nothing is a failure**, because only then was the
/// question unanswered — where one good channel among several is an answer,
/// and the rest say why they are not.
#[test]
fn a_box_no_channel_of_which_answers_fails() {
    let scratch = Scratch::new();
    mint::provisioned(&scratch.path().join(entry("alpha")), "127.0.0.1:1");
    let verdict = fanned(scratch.path(), &json!({"op": "workspaces"}));
    assert_eq!(verdict.code, 1);
    assert!(
        verdict.text.contains("connect 127.0.0.1:1"),
        "{}",
        verdict.text
    );
    assert!(
        verdict.text.contains("no wire provisioned at"),
        "{}",
        verdict.text
    );
}

/// An engine answering `ok: false` has ANSWERED, and its section is the
/// product — but it is not a channel this fan learned anything from, so it does
/// not carry the verdict on its own.
#[test]
fn a_channel_that_answers_no_is_printed_and_does_not_count_as_an_answer() {
    let scratch = Scratch::new();
    wired(
        &scratch,
        &flat(),
        vec![vec![json!({"ok": false, "said": "no"})]],
    );
    let verdict = fanned(scratch.path(), &json!({"op": "workspaces"}));
    assert_eq!(verdict.code, 1);
    assert_eq!(verdict.stream, Stream::Out, "an answer is not a diagnosis");
    assert!(verdict.text.contains(r#""said":"no""#), "{}", verdict.text);
}
