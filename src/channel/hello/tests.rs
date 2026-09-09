//! The preface: what this end states, and every way a peer can fail to agree.

use super::{PROTOCOL, confirm, state};
use crate::channel::edition;
use crate::channel::{Reach, frame};
use serde_json::{Value, json};

/// The bytes a peer stating `protocol` would put on the wire.
fn stated(v: &Value) -> Vec<u8> {
    let mut wire = Vec::new();
    frame::write_value(&mut wire, v).expect("write");
    wire
}

/// What this end writes is one frame carrying the major it must agree about
/// and the edition it can spell.
#[test]
fn this_end_states_its_major_and_its_edition_in_one_frame() {
    let mut wire = Vec::new();
    state(&mut wire).expect("state");
    let mut read = wire.as_slice();
    assert_eq!(
        frame::read_value(&mut read).expect("read"),
        Some(json!({ "protocol": PROTOCOL, "edition": edition::EDITION })),
    );
    assert_eq!(
        frame::read_value(&mut read).ok(),
        None,
        "the preface is one frame and nothing follows it here"
    );
}

/// An engine speaking this major is admitted, and what it answers is what that
/// engine can SPELL.
#[test]
fn an_engine_of_this_major_is_confirmed_and_answers_its_edition() {
    let wire = stated(&json!({ "protocol": PROTOCOL, "edition": edition::EDITION + 3 }));
    assert_eq!(confirm(&mut wire.as_slice()), Ok(edition::EDITION + 3));
}

/// **An engine that states no edition is at the floor, and so is one that
/// states nonsense there.** Absent, mistyped and out of range are one answer
/// because they are one fact: an engine of this major requires every path at
/// or below the floor whatever it says about editions. Nothing here refuses —
/// that is the whole difference between the preface's two keys.
#[test]
fn an_engine_that_states_no_edition_is_read_at_the_floor() {
    for said in [
        json!({ "protocol": PROTOCOL }),
        json!({ "protocol": PROTOCOL, "edition": "nineteen" }),
        json!({ "protocol": PROTOCOL, "edition": -1 }),
        json!({ "protocol": PROTOCOL, "edition": u64::from(u32::MAX) + 1 }),
    ] {
        let wire = stated(&said);
        assert_eq!(confirm(&mut wire.as_slice()), Ok(edition::FLOOR), "{said}");
    }
}

/// A mismatch names BOTH versions and the remedy. That is the requirement, not
/// a nicety: the sentence is the upgrade prompt, so a number an operator can
/// act on has to be in it.
#[test]
fn a_mismatch_names_both_versions_and_the_remedy() {
    let wire = stated(&json!({ "protocol": 99 }));
    let reach = confirm(&mut wire.as_slice()).expect_err("refused");
    let refusal = reach.said();
    assert!(
        refusal.contains(&format!("version {PROTOCOL}")),
        "{refusal}"
    );
    assert!(refusal.contains("engine speaks 99"), "{refusal}");
    assert!(refusal.contains("upgrade the older component"), "{refusal}");
    assert!(
        !refusal.contains("negotiat") || refusal.contains("no negotiation"),
        "the refusal must not offer a negotiation: {refusal}"
    );
    // **A peer that spoke adjudicated nothing.** REMOTE §3 refuses a version
    // this build does not speak *before any gesture is decoded*, so a request
    // that met this refusal did not run and the seat must not call it in doubt.
    assert!(!reach.crossed(), "a stated mismatch never crossed");
}

/// Four ways to state nothing, and they are one case: an unversioned build, a
/// frame that is not an object, an object without the key, and a terminator
/// where a preface belongs. Every one of them is a peer that SPOKE, so nothing
/// was adjudicated and none of them is in doubt.
#[test]
fn every_way_of_stating_nothing_is_the_one_sentence() {
    let silences = [
        stated(&json!({"op": "advertise"})),
        stated(&json!(["not an object"])),
        stated(&json!({"protocol": "one"})),
        {
            let mut wire = Vec::new();
            frame::write_end(&mut wire).expect("terminator");
            wire
        },
    ];
    for wire in silences {
        let reach = confirm(&mut wire.as_slice()).expect_err("refused");
        assert!(
            reach.said().contains("the engine speaks no version"),
            "{}",
            reach.said()
        );
        assert!(!reach.crossed(), "a peer that spoke adjudicated nothing");
    }
}

/// **The fifth case, split out of the four above** (bl-3969): a preface this
/// end could not READ at all. The request went out in the same breath as this
/// end's own preface, so the connection broke with the gesture already at the
/// far end — which REMOTE §3 calls IN DOUBT. It borrows no sentence about a
/// version, because nobody stated one.
#[test]
fn a_preface_this_end_could_not_read_leaves_the_request_in_doubt() {
    let reach = confirm(&mut [].as_slice()).expect_err("refused");
    assert_eq!(
        reach,
        Reach::Unanswered("the engine stated no version: failed to fill whole buffer".to_owned())
    );
    assert!(reach.crossed());
}
