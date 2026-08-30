//! **Rule 2: round-trip what you emit** — the frame this seat composes must be
//! the frame upstream generated, key for key and therefore byte for byte,
//! since a `serde_json::Map` is sorted and so is the fixture.
//!
//! Split from [`super`] on the seam the corpus itself draws: that half asks
//! whether a gesture can be READ and routed, which is a question about every
//! op in the vocabulary, and this one asks whether the eight this seat WRITES
//! come out right. The two change for different reasons — a verb added moves
//! this file, a shape added upstream moves that one.
//!
//! **Rule 3 lives here too** — *"a shape you do not implement is still one you
//! must not misread"*. [`UNEMITTED`] is the frames of an op the seat DOES emit
//! that its encoder cannot compose, recorded by count and reason. A frame that
//! stops round-tripping has to be moved into that list by hand, which is the
//! decision being recorded rather than passed over.

use std::collections::BTreeMap;

use serde_json::{Map, Value, json};

use super::super::super::{PREPARE, PROMPT, find, prepare, prompt};
use super::{emitted, request};
use crate::envelope;

/// The rung this seat composes, and the only one it can (`src/verbs/start.rs`).
const BARE: &str = "bare";

/// **The frames the seat's encoder cannot compose**, by op, count and reason.
///
/// Both entries are a surface this build does not have rather than a field it
/// drops. A count that moves — because yog grew a rung, or because a pane
/// landed here — fails until the reason is rewritten, which is the whole point
/// of writing it down.
const UNEMITTED: &[(&str, usize, &str)] = &[
    (
        "prepare",
        9,
        "the path and ball rungs: this seat composes the bare rung only, and a \
         seat that guessed a rung would found a claim nobody asked for",
    ),
    (
        "prompt",
        6,
        "a predicted conversation seed: this seat spells `seed` null, because \
         the mint is the engine's and a seat that predicted one would have to \
         fire the name it painted",
    ),
];

/// A required string field of a frame the vocabulary guarantees carries one.
/// It names the field it did not find: a verb whose `params` drifted off the
/// wire's own spelling fails here, and the whole remedy is the name.
fn text(obj: &Map<String, Value>, key: &str) -> String {
    obj.get(key)
        .and_then(Value::as_str)
        .unwrap_or_else(|| panic!("this shape's frame carries no field {key:?}"))
        .to_owned()
}

/// **This seat's own encoding of one frame**, or `None` where it has no way to
/// compose that frame at all.
///
/// The six rows go through the one builder they always go through, so nothing
/// here is a second spelling of a gesture (DESIGN §4.10). The start pair are
/// typed doors, and `prompt`'s body is decoded by the seat's **real** reader
/// before being handed back — which is what makes this a round trip rather
/// than a re-copy, and what proves the fields no pane paints ride through
/// untouched.
fn rebuilt(frame: &Value) -> Option<Value> {
    let obj = frame.as_object().expect("a gesture envelope");
    let op = text(obj, envelope::OP);
    if let Some(verb) = find(&op) {
        let args = verb.params.iter().map(|p| text(obj, p)).collect();
        return Some(verb.envelope(args).expect("the shape's own arity"));
    }
    match op.as_str() {
        PREPARE => (obj["payload"] == json!({ "rung": BARE }))
            .then(|| prepare(text(obj, envelope::WORKSPACE))),
        PROMPT => obj["seed"].is_null().then(|| {
            let staged = crate::reply::start::prepared(obj).expect("a staged body");
            let address = staged.workspace.clone();
            prompt(&staged, address, text(obj, "goal"))
        }),
        _ => None,
    }
}

/// Round-trip every frame of every op this seat emits, and record what it
/// cannot compose.
#[test]
fn what_the_seat_emits_is_what_the_corpus_carries() {
    let mut declined: BTreeMap<String, usize> = BTreeMap::new();
    for word in emitted() {
        for frame in &request(&word).frames {
            match rebuilt(frame) {
                Some(built) => assert_eq!(&built, frame, "{word}"),
                None => *declined.entry(word.clone()).or_default() += 1,
            }
        }
    }
    let recorded: BTreeMap<String, usize> = UNEMITTED
        .iter()
        .map(|(word, n, _)| ((*word).to_owned(), *n))
        .collect();
    assert_eq!(
        declined, recorded,
        "the frames this seat cannot compose have moved — amend UNEMITTED with \
         the reason, or emit them"
    );
}
