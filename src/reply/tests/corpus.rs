//! **The corpus replay** — every frame in `corpus/`, through the real reader.
//!
//! `corpus/README.md` is the contract; this is the mechanism, and it is
//! deliberately thin. A frame's **directory is its expectation**, so nothing
//! here holds a table of expected values that a dropped-in file would have to
//! be added to: a corpus emitted upstream (yog bl-32cb) lands as files and this
//! replay needs no edit at all.
//!
//! It is `cfg(test)`, and that is the honest placement. Nothing in production
//! replays a corpus — the seat reads frames off a socket — so a replay verb
//! would be a surface built for a gate rather than for an operator.
//!
//! **Both directions, the discipline every gate in this repo holds.** A
//! directory that enumerates nothing fails, and a file that no directory
//! claims fails: a corpus that silently replays zero frames passes forever,
//! and a frame dropped at the wrong level would be a fixture nobody runs.

use std::path::{Path, PathBuf};

use super::super::{Read, read};

/// The corpus root, resolved off the manifest rather than the working
/// directory: `cargo test` runs from the crate root today and a harness that
/// changed that would silently empty the corpus.
fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("corpus")
}

/// The three directories, which are the three outcomes.
const CLASSES: [&str; 3] = ["answers", "refusals", "unreadable"];

/// Every `*.json` file in one class, by name.
fn frames(class: &str) -> Vec<PathBuf> {
    let dir = root().join(class);
    let mut found: Vec<PathBuf> = std::fs::read_dir(&dir)
        .unwrap_or_else(|e| panic!("corpus/{class}: {e}"))
        .flatten()
        .map(|entry| entry.path())
        .filter(|path| path.extension().is_some_and(|ext| ext == "json"))
        .collect();
    found.sort();
    assert!(
        !found.is_empty(),
        "corpus/{class} holds no frame — the walk is broken, not the corpus"
    );
    found
}

/// Replay one class and assert every frame lands where its directory says.
fn replay(class: &str, expected: impl Fn(&Read) -> bool) {
    for path in frames(class) {
        let text = std::fs::read_to_string(&path).expect("a corpus frame");
        let frame = serde_json::from_str(&text)
            .unwrap_or_else(|e| panic!("{}: not JSON — {e}", path.display()));
        let answered = read(&frame);
        assert!(
            expected(&answered),
            "{} is filed under {class} and read as {answered:?}",
            path.display()
        );
    }
}

/// Every frame this build paints.
#[test]
fn the_answers_are_answers() {
    replay("answers", |answered| matches!(answered, Read::Answer(_)));
}

/// Every frame the engine refused. Both members of the pair here read the
/// same, which is the protocol's own rule: a workspace out of scope is
/// *absent*, not forbidden, so a seat must not be able to tell it from one
/// that does not exist.
#[test]
fn the_refusals_are_refusals() {
    replay("refusals", |answered| matches!(answered, Read::Refusal(_)));
}

/// Every frame this seat cannot read — malformed ones, and the perfectly good
/// answers of kinds no pane paints yet. This directory is the ledger of the
/// second class: a kind moves out of it in the release that starts painting
/// it, and the diff of that move is the record.
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
    for entry in std::fs::read_dir(root()).expect("the corpus").flatten() {
        let path = entry.path();
        let leaf = path.file_name().unwrap_or_default().to_string_lossy();
        assert!(
            leaf == "README.md" || CLASSES.contains(&leaf.as_ref()),
            "corpus/{leaf} is in no class — file it under one of {CLASSES:?}"
        );
    }
}
