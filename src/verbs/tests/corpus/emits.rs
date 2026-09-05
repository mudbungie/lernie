//! **Rule 2: round-trip what you emit** — the frame this seat composes must be
//! the frame upstream generated, key for key and therefore byte for byte,
//! since a `serde_json::Map` is sorted and so is the fixture.
//!
//! Split from [`super`] on the seam the corpus itself draws: that half asks
//! whether a gesture can be READ and routed, which is a question about every
//! op in the vocabulary, and this one asks whether the ones this seat
//! WRITES come out right. The two change for different reasons — a verb added moves
//! this file, a shape added upstream moves that one.
//!
//! **Rule 3 lives here too** — *"a shape you do not implement is still one you
//! must not misread"*. [`UNEMITTED`] is the frames of an op the seat DOES emit
//! that its encoder cannot compose, recorded by count and reason. A frame that
//! stops round-tripping has to be moved into that list by hand, which is the
//! decision being recorded rather than passed over.

use std::collections::BTreeMap;

use serde_json::{Map, Value, json};

use super::super::super::{
    EFFORT, FORK, OPS, PREPARE, PRIORITY, PROMPT, effort, find, fork, ops, prepare, priority,
    prompt,
};
use super::{emitted, request};
use crate::envelope;

/// The rung this seat composes, and the only one it can (`src/verbs/start.rs`).
const BARE: &str = "bare";

/// **The frames the seat's encoder cannot compose**, by op, count and reason.
///
/// Every entry is a surface this build does not have rather than a field it
/// drops. A count that moves — because yog grew a rung, or because a pane
/// landed here — fails until the reason is rewritten, which is the whole point
/// of writing it down.
const UNEMITTED: &[(&str, usize, &str)] = &[
    (
        "files",
        3,
        "the `at` and `path` forms: this seat composes the bare listing only \
         — pinning a commit and previewing one file are controls the records \
         pane does not have yet, and a seat that guessed either would answer \
         a question nobody asked",
    ),
    (
        "stop",
        1,
        "the children cascade: this seat composes the bare stop only, and the \
         flag that takes a whole subtree down is a second control with a second \
         confirmation — it belongs beside the records that would say what is \
         under there",
    ),
    (
        "marks",
        2,
        "the amending form: this seat composes the bare READ only, and pointing \
         a wall's task space at another branch is a write with a confirmation \
         to design — it belongs beside the four acts the ball pane does not \
         have yet (bl-f7ae)",
    ),
    (
        "prepare",
        9,
        "the path and ball rungs: this seat composes the bare rung only, and a \
         seat that guessed a rung would found a claim nobody asked for",
    ),
    (
        "governing",
        1,
        "the `at` form: this seat composes the bare read only — resolving the \
         policy as of another commit is the pin the records pane does not \
         have, and a seat that guessed a rev would answer about a tree \
         nobody named",
    ),
    (
        "fork",
        1,
        "the skill list: this seat composes an attempt that pins none, because \
         the skills a lineage declares are a read the config-file pane owns \
         and this window has no way to offer them",
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
/// The nine rows go through the one builder they always go through, so nothing
/// here is a second spelling of a gesture (DESIGN §4.10). The four doors are
/// typed, and `prompt`'s body is decoded by the seat's **real** reader
/// before being handed back — which is what makes this a round trip rather
/// than a re-copy, and what proves the fields no pane paints ride through
/// untouched.
///
/// **The tuning pair are the reason the doors exist**, so they are round-tripped
/// off the frame's own JSON types rather than off `text`: `level` is a string
/// or `null` and `on` is a bool, and a reading that went through a string would
/// be the very translation the doors were written to avoid.
fn rebuilt(frame: &Value) -> Option<Value> {
    let obj = frame.as_object().expect("a gesture envelope");
    let op = text(obj, envelope::OP);
    if let Some(verb) = find(&op) {
        // **A frame carrying a field the row does not name cannot be composed
        // here**, and that is a general rule rather than one op's arm: the
        // builder writes exactly `op` plus this row's parameters, so a field
        // beyond them can only ever come out missing. It answers `None` and
        // lands in the ledger below with its reason, which is the decision
        // being recorded instead of an assertion nobody could satisfy.
        if obj.len() != verb.params.len() + 1 {
            return None;
        }
        let args = verb.params.iter().map(|p| text(obj, p)).collect();
        return Some(verb.envelope(args).expect("the shape's own arity"));
    }
    match op.as_str() {
        PREPARE => (obj["payload"] == json!({ "rung": BARE }))
            .then(|| prepare(text(obj, envelope::WORKSPACE))),
        EFFORT => Some(effort(
            text(obj, envelope::WORKSPACE),
            text(obj, "role"),
            obj["level"].as_str().map(str::to_owned),
        )),
        PRIORITY => obj["on"]
            .as_bool()
            .map(|on| priority(text(obj, envelope::WORKSPACE), text(obj, "role"), on)),
        // The trail's depth is a number, which is why it is a door at all —
        // and it round-trips off the frame's own JSON type for the tuning
        // pair's reason: a reading that went through a string would be the
        // translation the doors exist to avoid.
        OPS => obj["max"].as_u64().map(ops),
        // The fork's `skills` is the reason it is a door: a list is not a
        // named string. It round-trips off the frame's own array, so a frame
        // that pins a skill is declined here rather than composed short.
        FORK => (obj["skills"].as_array().is_some_and(Vec::is_empty)).then(|| {
            fork(
                text(obj, envelope::WORKSPACE),
                text(obj, "parent"),
                text(obj, "from"),
                text(obj, "role"),
                text(obj, "goal"),
            )
        }),
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
