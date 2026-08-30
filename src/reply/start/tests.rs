//! The two start receipts: what is read, what is carried, and what refuses.

use crate::reply::{Read, Reply, read};
use serde_json::json;

/// The staged body's two read fields, and the receipt's one.
#[test]
fn a_staged_body_reads_its_address_and_its_goal_and_a_receipt_reads_its_name() {
    let Read::Answer(Reply::Prepared(staged)) = read(&json!({
        "ok": true, "kind": "prepared",
        "prepared": {"workspace": "home", "goal": "do the thing"}}))
    else {
        panic!("a staged start is an answer this build paints");
    };
    assert_eq!(staged.workspace, "home");
    assert_eq!(staged.goal, "do the thing");
    assert_eq!(
        read(&json!({"ok": true, "kind": "started", "conversation": "brisk-otter"})),
        Read::Answer(Reply::Started {
            conversation: "brisk-otter".to_owned()
        })
    );
}

/// **The body is carried whole, and that is the point.** A start staged with a
/// work target, a birth lineage and a banner origin this build paints none of
/// must still fire with all three — a seat that re-encoded its own reading
/// would drop every parameter it had not learned yet, and the dropped one is a
/// conversation born in the wrong directory off the wrong config.
#[test]
fn every_field_of_a_staged_body_survives_the_read_even_the_unpainted_ones() {
    let body = json!({
        "workspace": "home", "goal": "do it",
        "binding": "/home/u/dev/thing", "lineage": "reviewer", "origin": "balls",
        "a-field-this-build-has-never-seen": 7});
    let Read::Answer(Reply::Prepared(staged)) =
        read(&json!({"ok": true, "kind": "prepared", "prepared": body.clone()}))
    else {
        panic!("a staged start is an answer");
    };
    assert_eq!(staged.body, body);
}

/// Rung 1 on the nested body: a missing field refuses **naming itself**, and a
/// body that is not an object refuses naming the field it should have been.
#[test]
fn a_body_that_will_not_read_refuses_by_name() {
    for (frame, named) in [
        (
            json!({"ok": true, "kind": "prepared", "prepared": {"workspace": "home"}}),
            "\"goal\"",
        ),
        (
            json!({"ok": true, "kind": "prepared", "prepared": {"goal": "x"}}),
            "\"workspace\"",
        ),
        (
            json!({"ok": true, "kind": "prepared", "prepared": "home"}),
            "\"prepared\"",
        ),
        (json!({"ok": true, "kind": "prepared"}), "\"prepared\""),
        (
            json!({"ok": true, "kind": "started", "conversation": 7}),
            "\"conversation\"",
        ),
        (json!({"ok": true, "kind": "started"}), "\"conversation\""),
    ] {
        let Read::Unreadable(why) = read(&frame) else {
            panic!("{frame} is not readable");
        };
        assert!(why.contains(named), "{why} should name {named}");
    }
}
