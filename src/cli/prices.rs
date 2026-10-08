//! **The two money acts, as argv spells them** (REMOTE §9.23; bl-9111) — the
//! doors [`crate::verbs::doors::PRICE`] and [`crate::verbs::doors::CEILING`].
//!
//! Read here, in the pure function, for [`super::trail`]'s reason: whether a
//! rate is a number is decided entirely by what was typed, so a word that is
//! not one is the typist's mistake, it earns the door's usage, and it costs no
//! connection. Both name no workspace, so both fan — every channel this box
//! holds is the subject, exactly as `lernie ack` and `lernie clear-trail` are.

use super::{Decided, Verdict, misplaced};
use crate::render::Form;
use crate::verbs::doors::{CEILING, Door, PRICE};

/// `lernie price <provider> <model> <rates…> | off`.
pub(super) fn price(provider: &str, model: &str, words: &[&str], form: Form) -> Decided {
    match crate::verbs::prices::rates(words) {
        Ok(rates) => Decided::Fanned(
            crate::verbs::price(provider.to_owned(), model.to_owned(), rates),
            form,
        ),
        Err(why) => refused(&PRICE, why, words),
    }
}

/// `lernie ceiling <usd> | off`.
pub(super) fn ceiling(word: &str, form: Form) -> Decided {
    match crate::verbs::prices::bound(word) {
        Ok(usd) => Decided::Fanned(crate::verbs::ceiling(usd), form),
        Err(why) => refused(&CEILING, why, &[word]),
    }
}

/// The refusal, with the door's usage and the misplaced-flag hint.
fn refused(door: &Door, why: String, words: &[&str]) -> Decided {
    Decided::Say(Verdict::refused(misplaced(
        format!("`lernie {}`: {why} — usage: {}", door.word, door.usage()),
        words,
    )))
}
