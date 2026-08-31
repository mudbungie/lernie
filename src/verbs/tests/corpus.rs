//! **The corpus replay, request direction** — every frame under
//! `corpus/request/`, against the envelope reader and this seat's own encoder.
//!
//! The reply half is judged by where a frame lands (`src/reply/tests/`); a
//! request has no such class, so the two questions asked are upstream's own
//! contract for a client, read off `corpus/README.md`:
//!
//! 1. **Decode everything** — this file. Every request frame in the vocabulary
//!    is a gesture this seat can read as one and route by, including the
//!    sixty-odd ops it has no word for, which cross through `lernie ask`
//!    unchanged. Routing is where a miss costs something: a workspace slot the
//!    seat does not look in sends the gesture down the wrong channel, and it
//!    does it silently.
//! 2. **Round-trip what you emit** — [`emits`], with rule 3's ledger beside it.

use serde_json::{Value, json};

use super::super::{PREPARE, PROMPT, table};
use crate::envelope;
use crate::test_support::corpus::{Fixture, files, fixture, record, root};

/// Rule 2, and the record of what this seat cannot compose.
mod emits;

/// Every op this seat composes: the six rows, and the start family's two
/// doors. Derived from the table rather than listed, so a verb added is in it.
fn emitted() -> Vec<String> {
    table()
        .iter()
        .map(|verb| verb.word.to_owned())
        .chain([PREPARE.to_owned(), PROMPT.to_owned()])
        .collect()
}

/// Every vendored request fixture, read.
fn vendored() -> Vec<Fixture> {
    files("request").iter().map(|p| fixture(p)).collect()
}

/// One op's fixture, by the word that names it.
fn request(word: &str) -> Fixture {
    fixture(&root().join("request").join(format!("{word}.json")))
}

/// The address upstream's own signature says a gesture carries — the top-level
/// slot, the one nested inside a prepared body, or none.
///
/// Derived from `shapes.json` rather than from the seat's rule, so it is a
/// second opinion: `config`'s destination carries a `workspace` inside
/// `target` and neither table treats it as the gesture's address (DESIGN §6, bl-4a36),
/// and this is what would notice if one of them started to.
fn addressed(signature: &[String], frame: &Value) -> Option<String> {
    let named = |field: &str| signature.iter().any(|f| f == field);
    if named("/workspace:string") {
        return frame[envelope::WORKSPACE].as_str().map(str::to_owned);
    }
    if named("/prepared/workspace:string") {
        return frame[envelope::PREPARED][envelope::WORKSPACE]
            .as_str()
            .map(str::to_owned);
    }
    None
}

/// **The vendored request set is upstream's, whole**, both directions — and
/// every op the seat has a word for is in it, so a verb naming an op the
/// engine never had fails here rather than on a connection.
#[test]
fn every_request_shape_is_vendored_and_every_verb_is_one() {
    let filed: Vec<String> = vendored()
        .iter()
        .map(|file| {
            format!(
                "request/{}",
                file.shape.clone().expect("a vendored fixture")
            )
        })
        .collect();
    let upstream: Vec<String> = record()
        .shapes
        .into_keys()
        .filter(|key| key.starts_with("request/"))
        .collect();
    assert_eq!(
        filed, upstream,
        "the vendored request set and upstream's shape record disagree — \
         run scripts/refresh-corpus.sh against a yog checkout"
    );
    for word in emitted() {
        assert!(
            filed.contains(&format!("request/{word}")),
            "this seat emits {word:?} and the vocabulary has no such op"
        );
    }
}

/// **Rule 1: decode everything.** Every frame reads as a gesture envelope, and
/// the workspace it is routed by is the one upstream says it carries.
#[test]
fn every_request_frame_is_a_gesture_this_seat_can_route() {
    let shapes = record().shapes;
    for file in vendored() {
        let shape = file.shape.clone().expect("a vendored fixture");
        let signature = &shapes[&format!("request/{shape}")];
        for frame in &file.frames {
            let parsed = envelope::parse(&frame.to_string())
                .unwrap_or_else(|why| panic!("{}: {why}", file.path.display()));
            assert_eq!(parsed[envelope::OP], json!(shape), "{shape}");
            assert_eq!(
                envelope::workspace(&parsed),
                addressed(signature, frame),
                "{shape}: the seat routes this gesture by a different address \
                 than the one its shape carries"
            );
        }
    }
}
