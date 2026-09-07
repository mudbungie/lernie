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
    ADDRESS, CREATE, DELIVER, EFFORT, ENROLL, FAN, FORK, OPS, PREPARE, PRIORITY, PROMPT, RETIRE,
    UPDATE, create, deliver, effort, enroll, fan, find, fork, ops, prepare, priority, prompt,
    retire, update,
};
use super::{emitted, request};
use crate::envelope;

/// The frames this seat's encoder cannot compose, by op, count and reason.
mod ledger;

use ledger::UNEMITTED;

/// The rung this seat composes, and the only one it can (`src/verbs/start.rs`).
const BARE: &str = "bare";
const PATH: &str = "path";

/// **The array neither authoring door composes**, named once: a frame carrying
/// it is declined here and recorded in [`UNEMITTED`] above.
const FIELDS: &str = "fields";

/// An OPTIONAL string field, read as the doors read it — absent is `None`, and
/// an empty string is a value somebody wrote.
fn said(obj: &Map<String, Value>, key: &str) -> Option<String> {
    Some(obj.get(key)?.as_str()?.to_owned())
}

/// A required string field of a frame the vocabulary guarantees carries one.
/// It names the field it did not find: a verb whose `params` drifted off the
/// wire's own spelling fails here, and the whole remedy is the name.
fn text(obj: &Map<String, Value>, key: &str) -> String {
    obj.get(key)
        .and_then(Value::as_str)
        .unwrap_or_else(|| panic!("this shape's frame carries no field {key:?}"))
        .to_owned()
}

/// **One row of the verb table, rebuilt from a frame.**
///
/// Split out of [`rebuilt`] at clippy's function-length gate, on the seam the
/// function already has: this is the GENERIC arm — every op that is a row of
/// named strings and boolean flags — where everything above and below it is an
/// op with a door of its own.
fn from_row(verb: crate::verbs::Verb, obj: &Map<String, Value>) -> Option<Value> {
    // **A frame carrying a field the row does not name cannot be composed
    // here**, and that is a general rule rather than one op's arm: the
    // builder writes exactly `op`, this row's parameters and whichever of
    // its flags the frame raises, so a field beyond them can only ever
    // come out missing. It answers `None` and lands in the ledger below
    // with its reason, which is the decision being recorded instead of an
    // assertion nobody could satisfy.
    //
    // **A flag is raised by the word and never by `false`** (bl-9fd1): a
    // frame that spells one `false` is a field this seat's builder does
    // not write, so it is declined here rather than round-tripped into an
    // assertion nobody made.
    let raised: Vec<String> = verb
        .flags
        .iter()
        .filter(|flag| obj.get(**flag) == Some(&json!(true)))
        .map(|flag| (*flag).to_owned())
        .collect();
    if obj.len() != verb.params.len() + raised.len() + 1 {
        return None;
    }
    let args = verb
        .params
        .iter()
        .map(|p| text(obj, p))
        .chain(raised)
        .collect();
    Some(verb.envelope(args).expect("the shape's own arity"))
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
    // **The one row with a door beside it** (bl-971c): `enroll` is in the verb
    // table and its request carries an optional field, so the generic arm
    // below — which composes exactly the row's parameters and its raised flags
    // — declines the `--at` form on arity alone. It is composed here through
    // the same door both faces use, absence and all: `address` unstated is the
    // engine's own, which is a value rather than a field left short.
    if op == ENROLL.word {
        return Some(enroll(
            text(obj, envelope::WORKSPACE),
            text(obj, "name"),
            text(obj, "grade"),
            said(obj, ADDRESS),
        ));
    }
    if let Some(verb) = find(&op) {
        return from_row(verb, obj);
    }
    match op.as_str() {
        // **Both rungs this seat composes** (bl-4371): the bare payload, and
        // the path payload read back off its own two fields. The ball rung's
        // eight frames still answer `None` and are recorded below.
        PREPARE => match &obj["payload"] {
            payload if *payload == json!({ "rung": BARE }) => {
                Some(prepare(text(obj, envelope::WORKSPACE), None))
            }
            payload if payload["rung"] == json!(PATH) => Some(prepare(
                text(obj, envelope::WORKSPACE),
                payload["dir"].as_str().map(str::to_owned),
            )),
            _ => None,
        },
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
        // The candidate family's three. Each is declined where the frame omits
        // `ball` — the engine's own focused-ball form, which a seat has no way
        // to mean — and `fan`'s prepared body goes through the seat's REAL
        // reader before being handed back, exactly as `prompt`'s does.
        FAN => obj.contains_key("ball").then(|| {
            let staged = crate::reply::start::prepared(obj).expect("a staged body");
            let address = staged.workspace.clone();
            fan(
                &staged,
                address,
                text(obj, "ball"),
                text(obj, "project"),
                obj["n"].as_u64().expect("a count"),
            )
        }),
        DELIVER => obj.contains_key("ball").then(|| {
            deliver(
                text(obj, "ball"),
                text(obj, "project"),
                text(obj, "handle"),
                text(obj, "summary"),
            )
        }),
        RETIRE => obj
            .contains_key("ball")
            .then(|| retire(text(obj, "ball"), text(obj, "project"), text(obj, "handle"))),
        // The two authoring doors, round-tripped off the frame's own keys
        // rather than off `text`: absence is a value on both, and a reading
        // that filled a missing key with `""` would be exactly the
        // translation the doors exist to avoid.
        CREATE => (!obj.contains_key(FIELDS)).then(|| {
            create(
                text(obj, "project"),
                text(obj, "name"),
                text(obj, "title"),
                said(obj, "body"),
            )
        }),
        UPDATE => (!obj.contains_key(FIELDS)).then(|| {
            update(
                text(obj, "project"),
                text(obj, "id"),
                text(obj, "name"),
                said(obj, "title"),
                said(obj, "body"),
                said(obj, "note"),
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
