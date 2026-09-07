//! **`start`'s grammar** — the composite's two optional words, in its own file
//! for [`super::enroll`]'s reason: a verb whose grammar is more than words is
//! its own file, and this one's tail stopped being words when the role landed.
//!
//! # Two optional words that are not the same KIND of optional
//!
//! The work target is positional, because the §3.4 rung is what it decides and
//! a rung is not a modifier: `lernie start <address> <goal> <dir>` IS the path
//! rung, and there is no second word to keep in agreement with it.
//!
//! `--role <name>` is a modifier, and it is written rather than positional
//! because it is stated on the FIRE and not on the stage (REMOTE §9.21):
//! `prepare` answers `role: null` — litany's `worker` — and which role an
//! operator wants their own conversation under is the choice made between the
//! two acts. Plan mode is exactly that choice, which is why the start is where
//! it is entered.
//!
//! # And the name is not read here
//!
//! Unlike `enroll`'s grade, a role is **not** a closed set this binary holds: a
//! workspace's roles are whatever its governing config commit declares, and
//! this seat cannot know them without reading that commit. litany resolves the
//! role before the fork, so a name it does not declare leaves no branch, no ref
//! and no worktree and refuses in its own words. What is decided here is only
//! the GRAMMAR — that the word was given a value — which is the same division
//! `enroll`'s `--at` already draws.

use super::{Decided, Verdict};
use crate::render::Form;
use crate::verbs::doors::START;
use crate::verbs::start::ROLE;

/// The written word `start` takes after its arguments. `--role`, the wire's
/// own field name and litany's own flag, so an operator who learned it at one
/// face has learned it at all three.
pub const AS_ROLE: &str = "--role";

/// **The composite start**, with the rung the positional word decides and the
/// role the written one asks for.
pub(super) fn start(address: &str, goal: &str, tail: &[&str], form: Form) -> Decided {
    let (dir, role) = match stated(tail) {
        Ok(pair) => pair,
        Err(refusal) => return Decided::Say(Verdict::refused(refusal)),
    };
    Decided::Start {
        address: address.to_owned(),
        goal: goal.to_owned(),
        dir,
        role,
        form,
    }
}

/// **The tail, read as the two things it can be** — a work target, then the
/// role word with its value — or the one refusal that teaches the whole
/// grammar.
///
/// One sentence for every way it can be wrong, `enroll`'s own rule: to the
/// operator a bad tail is one event, and naming the grammar answers an unknown
/// word, a repeat and a word with no value at once.
fn stated(tail: &[&str]) -> Result<(Option<String>, Option<String>), String> {
    let (dir, rest) = match tail {
        [first, more @ ..] if *first != AS_ROLE => (Some((*first).to_owned()), more),
        _ => (None, tail),
    };
    match rest {
        [] => Ok((dir, None)),
        [word, name] if *word == AS_ROLE => Ok((dir, Some((*name).to_owned()))),
        _ => Err(tailed(tail)),
    }
}

/// The sentence that teaches the tail, quoting back what was typed.
///
/// **It leads with the quoting**, because a tail this arm cannot read is
/// overwhelmingly an unquoted goal: argv quotes, so a goal with spaces is ONE
/// argument, and three typed words are otherwise indistinguishable from one
/// sentence. The grammar follows, off the door's own usage line rather than a
/// second copy of it.
fn tailed(tail: &[&str]) -> String {
    format!(
        "`lernie start` cannot read {:?} after its goal — a goal with spaces is ONE \
         argument, so quote it; after it come an optional work target and an optional \
         `{AS_ROLE} <name>`, the {ROLE:?} the conversation is born on. Usage: {}",
        tail.join(" "),
        START.usage()
    )
}
