//! **`start`'s grammar**, read back as values — the rung the positional word
//! decides, the role the written one asks for, and the four tails between them.
//!
//! Its own file beside [`super::super::start`] for that module's own reason,
//! and [`super::answer`]'s: a verb whose grammar is more than words is its own
//! file, and so are its beats.

use super::{argv, said};
use crate::cli::{Decided, run};

/// The rung and the role one invocation decided on.
#[track_caller]
fn started(words: &[&str]) -> (Option<String>, Option<String>) {
    match run(argv(words)) {
        Decided::Start { dir, role, .. } => (dir, role),
        other => panic!("{words:?} decided {other:?}"),
    }
}

/// **The four tails, and each is one reading.** The work target is positional
/// because the §3.4 rung is what it decides — a rung is not a modifier — and
/// `--role` is written because it is stated on the FIRE rather than on the
/// stage (REMOTE §9.21): which role an operator wants is the choice made
/// between the two acts, which is exactly what plan mode is.
#[test]
fn the_rung_is_positional_and_the_role_is_written() {
    assert_eq!(started(&["start", "home", "go"]), (None, None));
    assert_eq!(
        started(&["start", "home", "go", "/w"]),
        (Some("/w".to_owned()), None)
    );
    assert_eq!(
        started(&["start", "home", "go", "--role", "planner"]),
        (None, Some("planner".to_owned()))
    );
    assert_eq!(
        started(&["start", "home", "go", "/w", "--role", "planner"]),
        (Some("/w".to_owned()), Some("planner".to_owned()))
    );
}

/// **The role word with no value is the grammar being wrong, not a role called
/// nothing** — and it earns the sentence that teaches the whole tail, because
/// to the operator a tail this arm cannot read is one event.
#[test]
fn the_role_word_with_no_name_refuses_and_teaches_the_tail() {
    let refusal = said(&["start", "home", "go", "--role"]);
    assert!(refusal.text.contains("--role"), "{}", refusal.text);
    assert!(
        refusal.text.contains(&crate::verbs::doors::START.usage()),
        "{}",
        refusal.text
    );
}

/// **A role this seat does not check**, for the reason it checks no path: a
/// workspace's roles are whatever its governing config commit declares, which
/// this seat cannot read. litany resolves the role before the fork, so a name
/// that is not declared leaves no branch, no ref and no worktree — and a second
/// validity check here could only disagree with the one that matters.
#[test]
fn a_role_this_seat_cannot_know_is_carried_rather_than_refused() {
    assert_eq!(
        started(&["start", "home", "go", "--role", "a-role-from-2027"]),
        (None, Some("a-role-from-2027".to_owned()))
    );
}
