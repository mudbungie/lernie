//! **The words that are not one ask** — the composites, and the one row whose
//! answer is drawn rather than printed.
//!
//! Split from [`super`] at the 300-line cap on the seam the surface itself
//! has: every other word in the table decides one `Ask` or one `Fanned`, and
//! these decide something else. `start` holds a local between two acts,
//! `model` spends a second row before it writes, and `enroll` draws its answer
//! instead of printing it. Each earns an arm of its own in
//! [`crate::cli::run`], and each is asserted here.

use super::super::super::{Decided, run};
use super::super::{argv, said};
use crate::cli::verdict::REFUSED;

/// **The composite is one word and two words of argument.** It carries them
/// rather than an envelope, because there are two envelopes and the second
/// cannot be built until the first is answered.
#[test]
fn the_start_word_carries_a_workspace_and_a_goal() {
    let Decided::Start { address, goal, .. } = run(argv(&["start", "home", "do the thing"])) else {
        panic!("`start` begins a conversation");
    };
    assert_eq!(address, "home");
    assert_eq!(goal, "do the thing");
}

/// **`--into <dir>` is read here**, in the pure function, because a destination
/// on this box is decided entirely by what was typed (bl-1554).
#[test]
fn the_enrollment_carries_the_destination_it_was_given() {
    let Decided::Enroll { into, grade, .. } = run(argv(&[
        "enroll",
        "ops",
        "box-1",
        "foot",
        crate::cli::INTO,
        "/home/u/carry",
    ])) else {
        panic!("`enroll` with a destination is still an enrollment");
    };
    assert_eq!(into.as_deref(), Some("/home/u/carry"));
    assert_eq!(grade, "foot");
}

/// A tail that is anything else refuses **naming the one word it takes**,
/// rather than earning the verb table's arity sentence about three arguments.
#[test]
fn a_tail_that_is_not_the_one_word_refuses_and_names_it() {
    for tail in [
        vec!["enroll", "ops", "box-1", "foot", "/home/u/carry"],
        vec!["enroll", "ops", "box-1", "foot", "--out", "/home/u/carry"],
        vec!["enroll", "ops", "box-1", "foot", crate::cli::INTO],
        vec![
            "enroll",
            "ops",
            "box-1",
            "foot",
            crate::cli::INTO,
            "/home/u/carry",
            "extra",
        ],
    ] {
        let refused = said(&tail);
        assert_eq!(refused.code, REFUSED, "{tail:?}");
        assert!(
            refused.text.contains(crate::cli::INTO),
            "{tail:?}: {}",
            refused.text
        );
    }
}

/// **A third word is the work target** (bl-4371), and its absence is the bare
/// rung — the `Option` IS the rung, so there is no fourth word saying which.
#[test]
fn a_third_word_aims_the_start_at_a_directory() {
    let Decided::Start {
        address, goal, dir, ..
    } = run(argv(&["start", "home", "do the thing", "/work/repo"]))
    else {
        panic!("a start with a work target");
    };
    assert_eq!(
        (address, goal, dir),
        (
            "home".to_owned(),
            "do the thing".to_owned(),
            Some("/work/repo".to_owned())
        )
    );
    let Decided::Start { dir: None, .. } = run(argv(&["start", "home", "do the thing"])) else {
        panic!("two words is the bare rung");
    };
}

/// **`model` spends a second row first** (bl-1e5a): the assignment is the
/// table's own, and what the word adds ahead of it is the read that says
/// whether the id is one the provider offers.
#[test]
fn the_model_word_carries_the_four_it_assigns() {
    let Decided::Model {
        workspace,
        role,
        provider,
        model,
        ..
    } = run(argv(&["model", "home", "worker", "codex", "gpt-5.4"]))
    else {
        panic!("`model` checks before it assigns");
    };
    assert_eq!(
        (workspace, role, provider, model),
        (
            "home".to_owned(),
            "worker".to_owned(),
            "codex".to_owned(),
            "gpt-5.4".to_owned()
        )
    );
    // **The row is still the row**, so a wrong arity earns its own usage.
    let refused = said(&["model", "home", "worker"]);
    assert_eq!(refused.code, REFUSED);
    assert!(
        refused
            .text
            .contains("lernie model <workspace> <role> <provider> <model>"),
        "{}",
        refused.text
    );
}
