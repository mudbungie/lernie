//! The three things a seat has to understand about an envelope, and the
//! discipline that they are read from one table.

use super::{OP, WORKSPACE, parse, succeeded, with_workspace, workspace};
use serde_json::json;

/// A gesture is a JSON object with an `op`. Everything short of that refuses
/// here rather than spending a connection on it.
#[test]
fn parse_takes_a_gesture_and_refuses_everything_that_is_not_one() {
    assert_eq!(
        parse(r#"{"op":"workspaces"}"#).expect("a gesture"),
        json!({"op": "workspaces"})
    );
    for (text, said) in [
        ("{", "not JSON"),
        ("[1,2]", "a gesture is a JSON object"),
        (r#""workspaces""#, "a gesture is a JSON object"),
        (r#"{"workspace":"home"}"#, "missing field"),
        (r#"{"op":7}"#, "is not a string"),
    ] {
        let refusal = parse(text).expect_err("refused");
        assert!(refusal.contains(said), "{text}: {refusal}");
    }
}

/// The discriminant's name is the wire's, and the seat never interprets it: an
/// `op` this build has never heard of parses and crosses.
#[test]
fn an_unknown_op_is_carried_rather_than_judged() {
    let envelope = parse(&format!(r#"{{"{OP}":"a-verb-from-the-future"}}"#)).expect("a gesture");
    assert_eq!(workspace(&envelope), None);
}

/// **The top-level slot**: read and written through the same table.
#[test]
fn a_top_level_workspace_is_the_slot() {
    let envelope = json!({"op": "conversations", "workspace": "home"});
    assert_eq!(workspace(&envelope), Some("home".to_owned()));
    assert_eq!(
        with_workspace(&envelope, "personal"),
        json!({"op": "conversations", "workspace": "personal"})
    );
}

/// **The nested slot**, and it is load-bearing rather than an oddity: the name
/// inside a prepared body is handed straight back out as the next act's
/// address, so a prepared left in the host's spelling routes its own follow-up
/// to a name no entry claims.
#[test]
fn a_prepared_body_carries_the_slot_one_level_down() {
    let envelope = json!({
        "op": "prompt",
        "prepared": {"workspace": "home", "goal": "do the thing"},
        "goal": "do the thing",
    });
    assert_eq!(workspace(&envelope), Some("home".to_owned()));
    let written = with_workspace(&envelope, "personal");
    assert_eq!(written["prepared"]["workspace"], json!("personal"));
    assert_eq!(
        written["prepared"]["goal"], envelope["prepared"]["goal"],
        "the rewrite touched something other than the name"
    );
}

/// An envelope naming no workspace comes back byte for byte — the general path
/// with nothing to rewrite, not a case of its own. Four shapes reach that one
/// branch: no field at all, a `prepared` that is not an object, a `prepared`
/// with no name in it, and a name that is not a string.
#[test]
fn an_envelope_naming_no_workspace_crosses_unchanged() {
    for envelope in [
        json!({"op": "workspaces"}),
        json!({"op": "prompt", "prepared": "not an object"}),
        json!({"op": "prompt", "prepared": {"goal": "unaddressed"}}),
        json!({"op": "conversations", "workspace": 7}),
        json!({"op": "prompt", "prepared": {"workspace": 7}}),
    ] {
        assert_eq!(workspace(&envelope), None, "{envelope}");
        assert_eq!(
            with_workspace(&envelope, "personal"),
            envelope,
            "{envelope}"
        );
    }
}

/// **What is deliberately not a slot**: a `workspace` nested somewhere the
/// engine's own table does not read as the gesture's address. The config
/// family's destination carries one inside `target`, and it crosses untouched
/// — because yog's typed table does not treat it as the address either, and
/// the two tables agreeing is the property that matters.
#[test]
fn a_workspace_nested_anywhere_else_is_not_the_address() {
    let envelope = json!({
        "op": "config",
        "target": {"file": "brazen", "workspace": "home"},
        "text": "",
    });
    assert_eq!(workspace(&envelope), None);
    assert_eq!(with_workspace(&envelope, "personal"), envelope);
}

/// A value that is not an object at all has no slot, which is the same branch
/// as an envelope without one.
#[test]
fn a_value_that_is_not_an_object_has_no_slot() {
    let envelope = json!([1, 2, 3]);
    assert_eq!(workspace(&envelope), None);
    assert_eq!(with_workspace(&envelope, "personal"), envelope);
}

/// The rewrite's key is the wire's `workspace`, spelled once.
#[test]
fn the_slot_is_the_wire_s_own_field_name() {
    let envelope = json!({"op": "scan", WORKSPACE: "home"});
    assert_eq!(workspace(&envelope), Some("home".to_owned()));
}

/// **The last frame decides**, an empty stream is not ok, and a frame with no
/// verdict is not ok either — a seat does not read a missing `ok` as a good
/// one.
#[test]
fn the_verdict_is_the_last_frame_s_and_silence_is_never_yes() {
    assert!(succeeded(&[json!({"ok": true})]));
    assert!(succeeded(&[json!({"ok": false}), json!({"ok": true})]));
    assert!(!succeeded(&[json!({"ok": true}), json!({"ok": false})]));
    assert!(!succeeded(&[]));
    assert!(!succeeded(&[json!({"kind": "rows"})]));
    assert!(!succeeded(&[json!({"ok": "yes"})]));
}
