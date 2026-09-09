//! **The grows-only replay** — the same vendored answers read as an OLDER
//! engine would have written them, and as a NEWER one would (yog's
//! `docs/REMOTE.md` §3.2; DESIGN §4.9).
//!
//! `corpus.rs` beside this one asks whether every frame lands in the class its
//! directory claims. That question is asked of the frames the newest engine
//! writes, which are the only frames anyone can vendor — and the two engines a
//! seat actually meets are the other two: one that predates a field, and one
//! that has learned a word. Neither exists on the box running the suite, so
//! both are made here out of the frame that does exist and the stamps beside
//! it (`crate::test_support::corpus::editions`).
//!
//! **What it is judging is a rule and not a shape.** Under the edition
//! discipline `PROTOCOL` moves only on a break, so a client meets an engine
//! several additions away from it as the ordinary case rather than as a skew to
//! be refused. The two assertions are that neither direction costs a refusal:
//! an absent post-floor key reads as the fact before the field existed, and an
//! unrecognised word becomes a named catch-all carrying the word rather than a
//! guess or a dropped row.

use serde_json::Value;

use super::super::{Read, read};
use crate::channel::edition::{EDITION, FLOOR};
use crate::test_support::corpus::editions::{mutate, project, words};
use crate::test_support::corpus::{Signature, files, fixture, record};

/// A word no build has heard of, and none ever will: it is not a token in any
/// vocabulary REMOTE spells, and its shape says so at a glance in a failure.
const UNHEARD: &str = "a-word-no-engine-has-ever-written";

/// **Every vendored answer, with the signature upstream stamps it by.** The
/// seat's own frames in `answers/` are not among them — they are this
/// repository's inventions, and upstream states no signature for an invention.
fn painted() -> Vec<(String, Signature, Vec<Value>)> {
    let shapes = record().shapes;
    files("answers")
        .iter()
        .map(|path| fixture(path))
        .filter_map(|file| {
            let shape = file.shape.clone()?;
            let signature = shapes
                .get(&format!("reply/{shape}"))
                .unwrap_or_else(|| panic!("upstream stamps no signature for {shape}"));
            Some((shape, signature.clone(), file.frames))
        })
        .collect()
}

/// **Every answer this build paints still reads at every edition this major has
/// had.** For each one, the frame is projected back to what an engine of that
/// edition would have written — every key stamped later deleted — and must
/// still be an answer.
///
/// It is the whole of what "a post-floor key absent reads as its default"
/// means, asserted where the defaults are rather than at each reader: a decoder
/// that required a field the engine at the far end has never written would
/// refuse a listing an operator can see is fine.
#[test]
fn every_answer_reads_at_every_edition_of_this_major() {
    let mut replayed = 0_usize;
    for (shape, signature, frames) in painted() {
        for at in FLOOR..=EDITION {
            for frame in &frames {
                let older = project(frame, &signature, at);
                let answered = read(&older);
                assert!(
                    matches!(answered, Read::Answer(_)),
                    "{shape} projected to edition {at} reads as {answered:?} — a \
                     field stamped after {at} is being required",
                );
                replayed += 1;
            }
        }
    }
    assert!(
        replayed > 0,
        "no vendored answer was projected — the walk is broken, not the corpus"
    );
}

/// **An unrecognised word in any vocabulary is carried, never refused.** Every
/// string-typed path upstream spells, except the discriminant, set to a token
/// no build has heard of — one path at a time, so a failure names the path.
///
/// `kind` is excluded because it is the one place the opposite rule holds: the
/// discriminant stays strict by name (rung 2), and `corpus/unpainted/` is where
/// that is asserted.
#[test]
fn an_unheard_of_word_in_any_vocabulary_is_still_an_answer() {
    let mut mutated = 0_usize;
    for (shape, signature, frames) in painted() {
        for path in words(&signature) {
            for frame in &frames {
                let Some(said) = mutate(frame, &path, UNHEARD) else {
                    continue;
                };
                let answered = read(&said);
                assert!(
                    matches!(answered, Read::Answer(_)),
                    "{shape} with {path} set to an unheard-of word reads as \
                     {answered:?} — a vocabulary is refusing instead of carrying",
                );
                mutated += 1;
            }
        }
    }
    assert!(
        mutated > 0,
        "no word was mutated — the walk is broken, not the corpus"
    );
}
