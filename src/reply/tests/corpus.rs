//! **The corpus replay, reply direction** — every frame under `corpus/`'s
//! three assertion directories, through the real reader.
//!
//! `corpus/README.md` is the contract; this is the mechanism, and it is
//! deliberately thin. A frame's **directory is its expectation**, so nothing
//! here holds a table of expected values that a dropped-in file would have to
//! be added to — which is what lets yog's generated fixtures (yog bl-32cb) be
//! vendored verbatim and judged by placement alone.
//!
//! It is `cfg(test)`, and that is the honest placement. Nothing in production
//! replays a corpus — the seat reads frames off a socket — so a replay verb
//! would be a surface built for a gate rather than for an operator.
//!
//! **Both directions, the discipline every gate in this repo holds.** A
//! directory that enumerates nothing fails, a file that no directory claims
//! fails, and a vendored file that upstream's own shape record does not name
//! fails: a corpus that silently replays zero frames passes forever, and a
//! shape that quietly stopped being judged is the miss this exists to catch.

use std::collections::BTreeSet;

use super::super::{Read, read};
use crate::channel::hello::PROTOCOL;
use crate::test_support::corpus::{CLASSES, files, fixture, record, root};

/// Replay one class and assert every frame lands where its directory says.
fn replay(class: &str, expected: impl Fn(&Read) -> bool) {
    for path in files(class) {
        let file = fixture(&path);
        for frame in &file.frames {
            let answered = read(frame);
            assert!(
                expected(&answered),
                "{} is filed under {class} and read as {answered:?}",
                file.path.display()
            );
        }
    }
}

/// Every frame this build paints — the eight kinds' vendored fixtures, and the
/// seat's own frames for the rung-3 readings upstream's codec cannot emit.
#[test]
fn the_answers_are_answers() {
    replay("answers", |answered| matches!(answered, Read::Answer(_)));
}

/// Every frame the engine refused. The two seat-authored members here read the
/// same, which is the protocol's own rule: a workspace out of scope is
/// *absent*, not forbidden, so a seat must not be able to tell it from one
/// that does not exist.
#[test]
fn the_refusals_are_refusals() {
    replay("refusals", |answered| matches!(answered, Read::Refusal(_)));
}

/// Every frame this seat cannot read — malformed ones, and the perfectly good
/// answers of kinds no pane paints yet. **This directory is the ledger**: a
/// vendored shape sits here until a pane lands, and the diff that moves it to
/// `answers/` is the record of what that release added.
#[test]
fn the_unreadable_are_unreadable() {
    replay("unreadable", |answered| {
        matches!(answered, Read::Unreadable(_))
    });
}

/// **Nothing in the corpus is unreplayed.** A frame dropped at the corpus root
/// rather than into a class would otherwise sit there being nobody's
/// assertion — the same failure as an empty directory, one level up.
#[test]
fn every_corpus_file_belongs_to_a_class() {
    let carried = ["README.md", "shapes.json", "request"];
    for entry in std::fs::read_dir(root()).expect("the corpus").flatten() {
        let path = entry.path();
        let leaf = path.file_name().unwrap_or_default().to_string_lossy();
        assert!(
            carried.contains(&leaf.as_ref()) || CLASSES.contains(&leaf.as_ref()),
            "corpus/{leaf} is in no class — file it under one of {CLASSES:?}"
        );
    }
}

/// **Every reply shape upstream has is classified, and nothing else is.**
///
/// This is what makes the ledger a ledger rather than a habit: when yog grows
/// a kind, `scripts/refresh-corpus.sh` files it under `unreadable/` and the
/// diff says so — and a vendored file whose shape upstream retired fails here
/// rather than being replayed forever against a vocabulary that dropped it.
#[test]
fn every_vendored_reply_shape_is_classified_exactly_once() {
    let mut filed: Vec<String> = Vec::new();
    for class in CLASSES {
        for path in files(class) {
            let file = fixture(&path);
            let Some(shape) = file.shape else { continue };
            let stem = path.file_stem().unwrap_or_default().to_string_lossy();
            assert_eq!(
                shape,
                stem.as_ref(),
                "{} declares shape {shape:?} — a vendored file is named for its \
                 shape, which is how the refresh finds it again",
                file.path.display()
            );
            filed.push(format!("reply/{shape}"));
        }
    }
    let known: BTreeSet<String> = filed.iter().cloned().collect();
    assert_eq!(known.len(), filed.len(), "a shape filed under two classes");
    let upstream: BTreeSet<String> = record()
        .shapes
        .into_keys()
        .filter(|key| key.starts_with("reply/"))
        .collect();
    assert_eq!(
        known, upstream,
        "the vendored reply set and upstream's shape record disagree — \
         run scripts/refresh-corpus.sh against a yog checkout"
    );
}

/// **The corpus and this build speak one protocol**, and a mismatch says both
/// numbers — REMOTE §3's own requirement for the version preface, owed here
/// for the same reason: the sentence is the upgrade prompt.
#[test]
fn the_corpus_is_stamped_with_the_protocol_this_seat_speaks() {
    let corpus = record().protocol;
    assert_eq!(
        corpus,
        u64::from(PROTOCOL),
        "corpus/shapes.json is protocol {corpus} and this seat speaks \
         {PROTOCOL} — refresh the corpus, or the seat is behind the wire"
    );
}
