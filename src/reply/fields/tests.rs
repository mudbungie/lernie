//! Rung 1, one reader at a time: what each takes, and that every refusal
//! **names the field** it refused on.

use super::{count, exit, flag, list, opt_text, rows, secs, text};
use serde_json::{Map, Value, json};

/// A body to read fields out of.
fn body(v: &Value) -> Map<String, Value> {
    v.as_object().expect("an object").clone()
}

/// The required readers: the value when it is there and the right shape, and a
/// refusal naming the key when it is not.
#[test]
fn a_required_field_reads_or_refuses_by_name() {
    let o = body(&json!({
        "word": "home", "yes": true, "age": -3, "n": 7,
        "wrong": [],
    }));
    assert_eq!(text(&o, "word"), Ok("home".to_owned()));
    assert_eq!(flag(&o, "yes"), Ok(true));
    assert_eq!(secs(&o, "age"), Ok(-3));
    assert_eq!(count(&o, "n"), Ok(7));
    for refusal in [
        text(&o, "wrong").expect_err("not a string"),
        text(&o, "absent").expect_err("absent"),
    ] {
        assert!(refusal.contains("non-string"), "{refusal}");
    }
    assert!(
        flag(&o, "word")
            .expect_err("not a bool")
            .contains("\"word\""),
        "the key is named"
    );
    assert!(
        secs(&o, "word")
            .expect_err("not an int")
            .contains("\"word\"")
    );
    assert!(count(&o, "age").expect_err("negative").contains("\"age\""));
}

/// The exit status is the one narrowing on this surface that is a real check:
/// a status no `i32` holds is an engine saying something unpaintable.
#[test]
fn an_exit_status_is_narrowed_and_says_so() {
    assert_eq!(exit(&body(&json!({"exit": 0}))), Ok(0));
    assert_eq!(exit(&body(&json!({"exit": -1}))), Ok(-1));
    assert!(
        exit(&body(&json!({"exit": 9_999_999_999_i64})))
            .expect_err("out of range")
            .contains("out of range")
    );
    assert!(
        exit(&body(&json!({})))
            .expect_err("absent")
            .contains("\"exit\"")
    );
}

/// **Absent and null are one reading; a wrong type is still a refusal.** `None`
/// and `Some("")` stay two different claims.
#[test]
fn an_optional_field_reads_absence_as_a_fact() {
    let o = body(&json!({"said": "", "null": null, "wrong": 7}));
    assert_eq!(opt_text(&o, "said"), Ok(Some(String::new())));
    assert_eq!(opt_text(&o, "null"), Ok(None));
    assert_eq!(opt_text(&o, "absent"), Ok(None));
    assert!(
        opt_text(&o, "wrong")
            .expect_err("not a string")
            .contains("\"wrong\"")
    );
}

/// A listing reads every element, and **one bad element fails the listing**
/// rather than shortening it — a shorter list is a lie a window paints
/// silently.
#[test]
fn a_listing_refuses_rather_than_shortening() {
    let read = |v: &Value| {
        v.as_str()
            .map(str::to_owned)
            .ok_or_else(|| "row: not a string".to_owned())
    };
    let o = body(&json!({"rows": ["a", "b"], "other": ["c"], "flat": 7}));
    assert_eq!(rows(&o, read), Ok(vec!["a".to_owned(), "b".to_owned()]));
    assert_eq!(list(&o, "other", read), Ok(vec!["c".to_owned()]));
    assert!(
        list(&o, "flat", read)
            .expect_err("not an array")
            .contains("non-array")
    );
    let bad = body(&json!({"rows": ["a", 7]}));
    assert_eq!(rows(&bad, read), Err("row: not a string".to_owned()));
}
