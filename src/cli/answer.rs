//! **`answer`'s grammar** — the one word on this surface whose tail is a
//! closed set of words the wire requires and the operator may leave off.
//!
//! Its own file for [`super::enroll`]'s reason, which is yog's own line
//! reader's: *a verb whose grammar is more than words is its own file*.
//!
//! # Why the word is not simply a fourth parameter
//!
//! `scope` is REQUIRED on the wire in both directions (REMOTE §9.11 as amended,
//! PROTOCOL 18) and it is OPTIONAL to type. A fourth parameter on the row
//! would make an operator write `call` on every answer they ever gave, and a
//! flag cannot carry a word. So the row keeps its three parameters, the word
//! answers the fourth field from [`crate::verbs::capability::CALL`] when
//! nothing was typed — `ops`' own shape, where the wire demands a depth and
//! the word supplies the one the window asks with — and the door states it
//! either way.
//!
//! # And why a typo is read here
//!
//! The scope is a closed set of three the boundary defines and this binary
//! already holds, exactly as `enroll`'s grade is a closed set of two. So a
//! misspelling is decided entirely by what was typed: it is the caller's typo,
//! it earns the usage, and it costs no connection — where falling through to
//! the table would spend a round trip to be told `unknown scope "conversaton"`,
//! true and naming none of the three words that would have worked.

use super::{Decided, Verdict};
use crate::render::Form;
use crate::verbs::capability::{ANSWER, SCOPES};

/// **One capability answer**, with the reach the operator asked for or the
/// narrow one they get by saying nothing.
pub(super) fn answer(
    workspace: &str,
    agent: &str,
    verdict: &str,
    scope: Option<&str>,
    form: Form,
) -> Decided {
    if let Some(word) = scope.filter(|word| !SCOPES.contains(word)) {
        return Decided::Say(Verdict::refused(format!(
            "`lernie answer` takes a scope of {} and got {word:?} — usage: {} [{}]",
            SCOPES
                .iter()
                .map(|scope| format!("{scope:?}"))
                .collect::<Vec<String>>()
                .join(" or "),
            ANSWER.usage(),
            SCOPES.join(" | ")
        )));
    }
    Decided::Ask(
        crate::verbs::answer(
            workspace.to_owned(),
            agent.to_owned(),
            verdict.to_owned(),
            scope.map(str::to_owned),
        ),
        form,
    )
}
