//! One config file: the two views of one read, the empty schema that is a
//! reading, and the bounds that ride the control.

use super::Control;
use crate::reply::{Read, Reply, read};
use serde_json::json;

/// A file with a schema, as the corpus carries one.
fn typed() -> serde_json::Value {
    json!({"ok": true, "kind": "config", "text": "roles:\n  worker:\n", "settings": [
        {"entry": "worker", "name": "provider", "value": "gone",
         "help": "the provider row this role dispatches through",
         "fault": "brazen's table has no provider row `gone`",
         "control": {"kind": "provider"}},
        {"entry": "watcher", "name": "debounce_ms", "value": "100",
         "help": "how long a changed workspace coalesces",
         "control": {"kind": "number", "min": 0, "max": 10000}}]})
}

/// **Both views arrive together**, and the settings are the schema applied to
/// those very bytes rather than a second read of a file that may have moved.
#[test]
fn one_answer_carries_the_bytes_and_the_settings_read_out_of_them() {
    let Read::Answer(Reply::Config(held)) = read(&typed()) else {
        panic!("a config answer: {:?}", read(&typed()));
    };
    assert_eq!(held.text, "roles:\n  worker:\n");
    assert_eq!(held.settings.len(), 2);
    assert_eq!(held.settings[0].entry, "worker");
    assert_eq!(held.settings[0].value, "gone");
    assert_eq!(
        held.settings[0].fault.as_deref(),
        Some("brazen's table has no provider row `gone`")
    );
}

/// **The bounds ride the control**, as a shape rather than optional siblings —
/// judging a value at input cannot be done without the range, and a control
/// with no range says only what it is.
#[test]
fn the_bounds_ride_the_control_and_a_control_without_them_says_its_kind() {
    let Read::Answer(Reply::Config(held)) = read(&typed()) else {
        panic!("a config answer");
    };
    assert_eq!(held.settings[1].control.min, Some(0));
    assert_eq!(held.settings[1].control.max, Some(10000));
    assert_eq!(held.settings[1].control.says(), "number 0–10000");
    assert_eq!(held.settings[0].control.says(), "provider");
    assert_eq!(
        Control {
            kind: "number".to_owned(),
            min: Some(1),
            max: None,
        }
        .says(),
        "number",
        "half a range is no range to judge against"
    );
}

/// **A file with no schema answers an EMPTY array, not an absent key** — the
/// general path with empty input, which is what §9.5's raw-text destinations
/// are. The absence of a `fault` is the same kind of reading one row down.
#[test]
fn an_empty_schema_reads_and_an_absent_fault_is_nothing_wrong() {
    let bare = json!({"ok": true, "kind": "config", "text": "roles: []", "settings": []});
    let Read::Answer(Reply::Config(held)) = read(&bare) else {
        panic!("a config answer: {:?}", read(&bare));
    };
    assert!(held.settings.is_empty());
    let Read::Answer(Reply::Config(typed)) = read(&typed()) else {
        panic!("a config answer");
    };
    assert_eq!(typed.settings[1].fault, None);
}

/// Every required field refuses by name, and a `settings` key that is absent
/// rather than empty is a malformed answer rather than a file with no schema.
#[test]
fn a_missing_field_refuses_and_names_itself() {
    for (frame, key) in [
        (
            json!({"ok": true, "kind": "config", "settings": []}),
            "text",
        ),
        (
            json!({"ok": true, "kind": "config", "text": "x"}),
            "settings",
        ),
        (
            json!({"ok": true, "kind": "config", "text": "x", "settings": ["not an object"]}),
            "not a JSON object",
        ),
        (
            json!({"ok": true, "kind": "config", "text": "x", "settings": [
                {"entry": "a", "name": "b", "value": "c", "help": "d"}]}),
            "missing control",
        ),
        (
            json!({"ok": true, "kind": "config", "text": "x", "settings": [
                {"entry": "a", "name": "b", "value": "c", "help": "d",
                 "control": "not an object"}]}),
            "control: not a JSON object",
        ),
        (
            json!({"ok": true, "kind": "config", "text": "x", "settings": [
                {"entry": "a", "name": "b", "value": "c", "help": "d",
                 "control": {"kind": "number", "min": "low"}}]}),
            "min",
        ),
    ] {
        let Read::Unreadable(why) = read(&frame) else {
            panic!("{key}: {frame}");
        };
        assert!(why.contains(key), "{key:?}: {why}");
    }
}
