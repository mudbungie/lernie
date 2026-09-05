//! The trail's door and its two acts: the envelope the read requires, and the
//! two acts that carry nothing at all.

use serde_json::json;

use super::{ACK, CLEAR_TRAIL, DEPTH, OPS, ack, clear_trail, ops};

/// The read's bound is a number and it is required, which is the whole reason
/// it is a door rather than a row.
#[test]
fn the_door_builds_the_envelope_the_wire_requires() {
    assert_eq!(
        ops(DEPTH),
        json!({ "op": OPS, "max": DEPTH }),
        "the bound is a number and it is required"
    );
}

/// **Both acts carry nothing**, which is what makes them rows: the envelope is
/// the op and no field beside it.
#[test]
fn the_two_acts_build_bare_envelopes() {
    assert_eq!(ack(), json!({ "op": "ack" }));
    assert_eq!(clear_trail(), json!({ "op": "clear-trail" }));
}

/// **Neither names a workspace**, so the poster fans both — and the seat says
/// so rather than working around it (DESIGN §4.35).
#[test]
fn neither_act_addresses_a_workspace() {
    assert!(!ACK.addresses_a_workspace());
    assert!(!CLEAR_TRAIL.addresses_a_workspace());
    assert_eq!(ACK.usage(), "lernie ack");
    assert_eq!(CLEAR_TRAIL.usage(), "lernie clear-trail");
}
