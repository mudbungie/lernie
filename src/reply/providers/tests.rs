//! The provider table: the four required fields, the block whose absence is the
//! reading, and the offering that is a list of bare strings.

use super::{KIND, MODELS, ProviderRow, offered, row};
use crate::reply::{Read, Reply, read};
use serde_json::json;

/// A row that can be signed in to, as the corpus carries one.
fn open() -> serde_json::Value {
    json!({
        "name": "housevendor",
        "fact": "credential present",
        "blocked": null,
        "effort": true,
        "priority": true,
    })
}

#[test]
fn a_row_carries_its_name_its_credential_fact_and_what_it_takes() {
    let read = row(&open()).expect("a provider row");
    assert_eq!(
        read,
        ProviderRow {
            name: "housevendor".to_owned(),
            fact: "credential present".to_owned(),
            blocked: None,
            effort: true,
            priority: true,
        }
    );
    assert!(read.signable());
    assert_eq!(read.takes(), Some("takes effort and priority".to_owned()));
}

/// **A block is a sentence, and its absence is the whole of *this row is
/// signable*.** The seat never composes either — the engine knows which auth
/// model a row declares and this end does not.
#[test]
fn a_blocked_row_carries_the_engines_own_reason_and_is_not_signable() {
    let mut frame = open();
    frame["blocked"] = json!("no login flow");
    let read = row(&frame).expect("a blocked row");
    assert_eq!(read.blocked, Some("no login flow".to_owned()));
    assert!(!read.signable());
    let mut absent = open();
    absent.as_object_mut().expect("the row").remove("blocked");
    assert!(row(&absent).expect("an absent block").signable());
}

/// **The two capabilities are said in one line or in none**, because a row
/// that takes both is one fact rather than two badges to read together.
#[test]
fn what_a_row_takes_is_one_line_and_a_row_that_takes_neither_says_nothing() {
    let mut effort_only = open();
    effort_only["priority"] = json!(false);
    assert_eq!(
        row(&effort_only).expect("a row").takes(),
        Some("takes effort".to_owned())
    );
    let mut priority_only = open();
    priority_only["effort"] = json!(false);
    assert_eq!(
        row(&priority_only).expect("a row").takes(),
        Some("takes priority".to_owned())
    );
    let mut neither = open();
    neither["effort"] = json!(false);
    neither["priority"] = json!(false);
    assert_eq!(row(&neither).expect("a row").takes(), None);
}

/// **Rung 1 refuses by name**, and the name is the whole remedy.
#[test]
fn every_required_field_refuses_and_says_which_one() {
    for (field, why) in [
        ("name", "non-string"),
        ("fact", "non-string"),
        ("effort", "non-boolean"),
        ("priority", "non-boolean"),
    ] {
        let mut frame = open();
        frame[field] = json!(7);
        let said = row(&frame).expect_err(field);
        assert!(said.contains(field), "{said}");
        assert!(said.contains(why), "{said}");
    }
    let mut wrong = open();
    wrong["blocked"] = json!(7);
    let said = row(&wrong).expect_err("a block of the wrong type still refuses");
    assert!(said.contains("blocked"), "{said}");
    let said = row(&json!("not an object")).expect_err("a row that is not one");
    assert!(said.contains("not an object"), "{said}");
}

/// One model id is a bare string, and anything else refuses.
#[test]
fn an_offering_is_a_list_of_bare_strings() {
    assert_eq!(
        offered(&json!("house-model-1")).expect("a model"),
        "house-model-1"
    );
    let said = offered(&json!({"name": "house-model-1"})).expect_err("an object is not one");
    assert!(said.contains("not a string"), "{said}");
}

/// Both frames, through the real door — and a listing that refuses on one row
/// refuses whole rather than shortening.
#[test]
fn the_frames_read_as_their_listings_and_one_bad_row_fails_it_all() {
    let frame = json!({"kind": KIND, "ok": true, "rows": [open()]});
    let Read::Answer(Reply::Providers(rows)) = read(&frame) else {
        panic!("a providers frame is an answer: {:?}", read(&frame));
    };
    assert_eq!(rows.len(), 1);
    let frame = json!({"kind": MODELS, "ok": true, "rows": ["opus", "sonnet"]});
    let Read::Answer(Reply::Models(rows)) = read(&frame) else {
        panic!("a models frame is an answer: {:?}", read(&frame));
    };
    assert_eq!(rows, vec!["opus".to_owned(), "sonnet".to_owned()]);
    let mut broken = open();
    broken["name"] = json!(null);
    let frame = json!({"kind": KIND, "ok": true, "rows": [open(), broken]});
    assert!(matches!(read(&frame), Read::Unreadable(_)), "one bad row");
}
