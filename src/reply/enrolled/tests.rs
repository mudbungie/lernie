//! The material's reading, and the envelope it is re-said in.

use serde_json::{Map, Value, json};

use super::{Enrolled, MARKER, VERSION, enrolled};
use crate::reply::{Read, Reply, read};

/// A whole answer, as the engine writes one.
fn frame() -> Value {
    json!({
        "ok": true,
        "kind": "enrolled",
        "grade": "foot",
        "name": "phone-1",
        "address": "engine.invalid:7737",
        "ca": "-----BEGIN CERTIFICATE-----\nnotreal-ca\n-----END CERTIFICATE-----\n",
        "cert": "-----BEGIN CERTIFICATE-----\nnotreal-leaf\n-----END CERTIFICATE-----\n",
        "key": "-----BEGIN notreal KEY-----\nnotreal-key\n-----END notreal KEY-----\n",
    })
}

/// The frame's own object.
fn obj() -> Map<String, Value> {
    frame().as_object().cloned().expect("an object")
}

/// The material read off [`frame`].
fn material() -> Enrolled {
    enrolled(&obj()).expect("the whole frame reads")
}

/// **Six fields, each read into its own place.** A field misread here is a
/// symbol that scans into a box that cannot dial, which is a failure with no
/// visible symptom at all until somebody tries.
#[test]
fn the_six_fields_land_where_they_belong() {
    let held = material();
    assert_eq!(held.grade, "foot");
    assert_eq!(held.name, "phone-1");
    assert_eq!(held.address, "engine.invalid:7737");
    assert!(held.ca.contains("notreal-ca"));
    assert!(held.cert.contains("notreal-leaf"));
    assert!(held.key.contains("notreal-key"));
}

/// **Rung 1 throughout, and every refusal names its field.** There is no
/// optional field here: half an enrollment is worse than none, because it draws
/// a picture that looks finished.
#[test]
fn every_missing_field_refuses_and_says_which() {
    for field in ["grade", "name", "address", "ca", "cert", "key"] {
        let mut without = obj();
        without.remove(field);
        let said = enrolled(&without).expect_err("a field short of an enrollment");
        assert!(said.contains(field), "{field} went unnamed: {said}");
    }
}

/// A field of the wrong JSON type refuses too, which is the same rung read the
/// other way: `null` is not a certificate.
#[test]
fn a_field_of_the_wrong_type_refuses() {
    let mut wrong = obj();
    wrong.insert("key".to_owned(), Value::Null);
    assert!(enrolled(&wrong).is_err());
}

/// The kind reaches the roster, so the window and the command line both see an
/// answer rather than an unreadable frame.
#[test]
fn the_kind_reads_as_an_answer_of_its_own() {
    assert_eq!(read(&frame()), Read::Answer(Reply::Enrolled(material())));
}

/// **The envelope is the six fields under the marker, and nothing else**
/// (REMOTE §8.4). `ok` and `kind` say what a *wire answer* is, and a photograph
/// is not one.
#[test]
fn the_envelope_carries_the_marker_the_six_fields_and_no_wire_words() {
    let text = material().envelope();
    let parsed: Value = serde_json::from_str(&text).expect("the envelope is JSON");
    let map = parsed.as_object().expect("an object");
    assert_eq!(map.get(MARKER), Some(&Value::from(VERSION)));
    assert_eq!(map.len(), 7, "the marker and six fields: {text}");
    for absent in ["ok", "kind"] {
        assert!(map.get(absent).is_none(), "{absent} rode along: {text}");
    }
}

/// **Every field crosses verbatim** — PEM as minted, newlines and banners
/// intact, so an operator can paste one into `openssl x509 -text`. REMOTE §8.4
/// weighed the alternative and declined it.
#[test]
fn the_envelope_carries_the_pem_exactly_as_it_arrived() {
    let held = material();
    let parsed: Value = serde_json::from_str(&held.envelope()).expect("JSON");
    for (field, want) in [
        ("grade", &held.grade),
        ("name", &held.name),
        ("address", &held.address),
        ("ca", &held.ca),
        ("cert", &held.cert),
        ("key", &held.key),
    ] {
        assert_eq!(
            parsed.get(field).and_then(Value::as_str),
            Some(want.as_str())
        );
    }
}

/// **Compact**, which is what makes it fit: no whitespace between the tokens.
/// The only newlines in it are the ones inside the PEM strings, and those are
/// escaped rather than literal.
#[test]
fn the_envelope_is_compact() {
    let text = material().envelope();
    assert!(!text.contains('\n'), "a literal newline: {text}");
    assert!(!text.contains(": "), "whitespace after a colon: {text}");
    assert!(!text.contains(", "), "whitespace after a comma: {text}");
}

/// **The caption is the three fields that are not material.** It exists so no
/// caller has to decide which are safe to print — a decision that only has to
/// be got wrong once, and that a scrollback then keeps.
#[test]
fn the_caption_names_the_box_and_never_the_material() {
    let held = material();
    let said = held.caption();
    for named in [&held.name, &held.grade, &held.address] {
        assert!(said.contains(named.as_str()), "{named} is missing: {said}");
    }
    for secret in ["notreal-key", "notreal-leaf", "notreal-ca", "BEGIN"] {
        assert!(!said.contains(secret), "{secret} is in the caption: {said}");
    }
}
