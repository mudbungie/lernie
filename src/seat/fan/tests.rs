//! A gesture that names no workspace, asked of every channel this box holds.

use super::fanned;
use crate::channel::entries::WORKSPACE;
use crate::cli::Stream;
use crate::render::Form;
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

    let verdict = fanned(scratch.path(), &json!({"op": "workspaces"}), Form::Json);
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
    let verdict = fanned(scratch.path(), &json!({"op": "workspaces"}), Form::Json);
    assert_eq!(verdict.code, 0, "{}", verdict.text);
    assert!(
        verdict.text.contains(&yes().to_string()),
        "{}",
        verdict.text
    );
}

/// **An unprovisioned flat root is an ABSENCE, and a box that holds channels
/// hears nothing about it** (bl-b858). This is the previous test read the other
/// way: a seat for an engine somewhere else got a paragraph of refusal above
/// every working channel on every roster read, beginning *"if an engine runs on
/// this box"* on a box where none does and none is meant to — with the answer
/// it asked for underneath.
#[test]
fn an_unprovisioned_own_engine_says_nothing_beside_the_channels_that_answered() {
    let scratch = Scratch::new();
    wired(&scratch, &entry("alpha"), vec![vec![yes()]]);
    let verdict = fanned(scratch.path(), &json!({"op": "workspaces"}), Form::Json);
    let lines: Vec<&str> = verdict.text.lines().collect();
    assert_eq!(lines[0], "alpha", "{}", verdict.text);
    assert!(
        !verdict.text.contains("no wire provisioned"),
        "{}",
        verdict.text
    );
}

/// **A channel this box DOES hold still says why it could not answer.** What
/// the rule above drops is the one reading that is not about a channel at all;
/// half-provisioned material is a fault about material that is there, and it
/// keeps its section beside the entries that answered.
#[test]
fn a_flat_root_that_holds_something_broken_still_says_so() {
    let scratch = Scratch::new();
    wired(&scratch, &entry("alpha"), vec![vec![yes()]]);
    let bare = scratch.path().join(flat());
    std::fs::create_dir_all(&bare).expect("the flat root");
    std::fs::write(bare.join(crate::channel::material::ADDRESS), "h:1\n")
        .expect("half the material");
    let verdict = fanned(scratch.path(), &json!({"op": "workspaces"}), Form::Json);
    let lines: Vec<&str> = verdict.text.lines().collect();
    assert_eq!(lines[0], "(this box's own engine)");
    assert!(lines[1].contains("half-provisioned"), "{}", lines[1]);
    assert_eq!(lines[2], "alpha");
}

/// **And a box with nothing at all keeps the whole recipe**, because that is
/// precisely the box the recipe is for: the bare channel is enumerated when it
/// is provisioned, or when it is the only thing there is, which is one
/// condition rather than a special case.
#[test]
fn a_box_that_holds_nothing_at_all_is_still_told_how_to_provision_one() {
    let scratch = Scratch::new();
    let verdict = fanned(scratch.path(), &json!({"op": "workspaces"}), Form::Json);
    assert_eq!(verdict.code, 1);
    assert!(
        verdict.text.contains("no wire provisioned at"),
        "{}",
        verdict.text
    );
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
    let verdict = fanned(scratch.path(), &json!({"op": "workspaces"}), Form::Json);
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
    let verdict = fanned(scratch.path(), &json!({"op": "workspaces"}), Form::Json);
    assert_eq!(verdict.code, 1);
    assert!(
        verdict.text.contains("connect 127.0.0.1:1"),
        "{}",
        verdict.text
    );
    assert!(
        !verdict.text.contains("no wire provisioned at"),
        "a channel that answered nothing is still a channel this box holds, \
         and the absent one is not: {}",
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
    let verdict = fanned(scratch.path(), &json!({"op": "workspaces"}), Form::Json);
    assert_eq!(verdict.code, 1);
    assert_eq!(verdict.stream, Stream::Out, "an answer is not a diagnosis");
    assert!(verdict.text.contains(r#""said":"no""#), "{}", verdict.text);
}
