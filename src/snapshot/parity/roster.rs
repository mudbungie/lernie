//! **The roster: which ops owe this seat a control**, read off the vendored
//! corpus and decided nowhere here.
//!
//! yog's help table is the one home for the fact, because it is the component
//! every client meets and the fact is a *suite* fact rather than a client's
//! (PARITY §2). It publishes itself: each `HelpRow` carries a `surface`
//! classification, the corpus's `reply/help` fixture is generated from the
//! table, and this seat already vendors that fixture verbatim
//! (`scripts/refresh-corpus.sh`). So raising the bar on this seat is an edit at
//! yog followed by a corpus refresh — never a list maintained here.
//!
//! **The fixture is found by name across the three classes**, not at a fixed
//! path. Its directory is this seat's assertion about whether the window paints
//! a help pane (`corpus/README.md`), which is a different question from what
//! the rows say, and the day a pane lands the file moves.

use std::collections::BTreeSet;

use crate::test_support::corpus::{CLASSES, fixture, root};

/// The fixture's filename, which is its shape.
const SHAPE: &str = "help.json";

/// The two values of the classification, spelled as upstream spells them.
const CONTROL: &str = "control";
const MACHINE: &str = "machine";

/// **What the roster says**, as two sets: everything the wire has a word for,
/// and the subset every seat owes an interactable.
pub(crate) struct Roster {
    /// Every op in the table — the vocabulary an `act:` token must name.
    pub(crate) ops: BTreeSet<String>,
    /// The ops classed `control`.
    pub(crate) control: BTreeSet<String>,
}

/// **What one help row says**: its op, and whether the op owes a control.
///
/// A pure function of the row, extracted from the walk so that both of its
/// answers can be asked for directly — the same shape [`super::super::clipped`]
/// gives its geometry judgement, and for the same reason: an arm that has never
/// run is an arm nobody has evidence about.
///
/// **Absent and unrecognised are one refusal**, because they have one remedy.
/// The field is compile-required upstream, so either way this corpus is older
/// than the contract and the answer is a refresh, not a smaller roster.
pub(crate) fn classify(row: &serde_json::Value) -> Result<(String, bool), String> {
    let Some(verb) = row["verb"].as_str() else {
        return Err(format!("a help row names no verb: {row}"));
    };
    match row["surface"].as_str() {
        Some(CONTROL) => Ok((verb.to_owned(), true)),
        Some(MACHINE) => Ok((verb.to_owned(), false)),
        stated => Err(format!(
            "the help row {verb:?} is classed {stated:?}, which this seat has no \
             reading for — refresh the corpus"
        )),
    }
}

/// Read the roster.
///
/// **The second direction — that the walk enumerated anything at all — is the
/// standing test's** (`super::tests`), where an assertion is the vocabulary.
/// A roster classing nothing a control would satisfy every assertion in
/// [`super`] forever, which is how this kind of gate actually dies.
pub(crate) fn roster() -> Roster {
    let path = CLASSES
        .iter()
        .map(|class| root().join(class).join(SHAPE))
        .find(|path| path.exists())
        .unwrap_or_else(|| panic!("no {SHAPE} in any of {CLASSES:?} — refresh the corpus"));
    let help = fixture(&path);
    let mut ops = BTreeSet::new();
    let mut control = BTreeSet::new();
    for frame in &help.frames {
        for row in frame["rows"].as_array().expect("the help rows") {
            let (verb, owed) = classify(row).unwrap_or_else(|why| panic!("{why}"));
            if owed {
                control.insert(verb.clone());
            }
            ops.insert(verb);
        }
    }
    Roster { ops, control }
}
