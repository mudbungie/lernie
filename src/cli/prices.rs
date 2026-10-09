//! **The two money acts, as argv spells them** (REMOTE §9.23; bl-9111) — the
//! doors [`crate::verbs::doors::PRICE`] and [`crate::verbs::doors::CEILING`].
//!
//! Read here, in the pure function, for [`super::trail`]'s reason: whether a
//! rate is a number is decided entirely by what was typed, so a word that is
//! not one is the typist's mistake, it earns the door's usage, and it costs no
//! connection.
//!
//! **Both name the ONE engine they write** (bl-1bb9). A price row and a bound
//! are world facts, one world per engine, so an operator typing one number
//! means one engine — `lernie ack`'s fan was the wrong precedent, and it wrote
//! the same ceiling into every world this box is a client of. The channel is
//! named by a trailing [`ON`], written rather than positional because the
//! rates' own count already varies; where it is left off, the seat writes the
//! one channel the box holds and refuses on a box holding more
//! ([`crate::seat::aimed`]). `lernie prices`, the READ, still fans.

use super::{Decided, Verdict, misplaced};
use crate::render::Form;
use crate::verbs::doors::{CEILING, Door, PRICE};

/// **The written word that names the channel an act goes down** — the leaf
/// this box files an entry under, as `lernie entries` lists it.
pub const ON: &str = "--on";

/// `lernie price <provider> <model> <rates…> | off [--on <channel>]`.
pub(super) fn price(provider: &str, model: &str, tail: &[&str], form: Form) -> Decided {
    let (own, on) = aimed(tail);
    match crate::verbs::prices::rates(tail.get(..own).unwrap_or(tail)) {
        Ok(rates) => Decided::Aimed {
            on,
            envelope: crate::verbs::price(provider.to_owned(), model.to_owned(), rates),
            form,
        },
        Err(why) => refused(&PRICE, why, tail),
    }
}

/// `lernie ceiling <usd> | off [--on <channel>]`.
pub(super) fn ceiling(tail: &[&str], form: Form) -> Decided {
    let (own, on) = aimed(tail);
    let [word] = tail.get(..own).unwrap_or(tail) else {
        let why = format!("a ceiling is one bound, a number or `off`, and got {own} word(s)");
        return refused(&CEILING, why, tail);
    };
    match crate::verbs::prices::bound(word) {
        Ok(usd) => Decided::Aimed {
            on,
            envelope: crate::verbs::ceiling(usd),
            form,
        },
        Err(why) => refused(&CEILING, why, tail),
    }
}

/// **How many of the tail's words are the act's own, and the channel it
/// names** where a trailing [`ON`] pair says one.
fn aimed(tail: &[&str]) -> (usize, Option<String>) {
    match tail {
        [words @ .., word, name] if *word == ON => (words.len(), Some((*name).to_owned())),
        _ => (tail.len(), None),
    }
}

/// The refusal, with the door's usage and the misplaced-flag hint.
fn refused(door: &Door, why: String, words: &[&str]) -> Decided {
    Decided::Say(Verdict::refused(misplaced(
        format!("`lernie {}`: {why} — usage: {}", door.word, door.usage()),
        words,
    )))
}
