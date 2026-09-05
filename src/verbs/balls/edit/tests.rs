//! The two authoring doors: what they write, and what they leave out.

use serde_json::json;

use super::{create, update};

/// A body that was typed rides, and one that was not is **absent** — never an
/// empty string, which upstream reads as *blank this field*.
#[test]
fn a_filing_carries_a_body_only_where_there_is_one() {
    assert_eq!(
        create(
            "p".to_owned(),
            "alba".to_owned(),
            "a title".to_owned(),
            Some("the body".to_owned()),
        ),
        json!({
            "op": "create", "project": "p", "name": "alba",
            "title": "a title", "body": "the body",
        })
    );
    assert_eq!(
        create(
            "p".to_owned(),
            "alba".to_owned(),
            "a title".to_owned(),
            None
        ),
        json!({ "op": "create", "project": "p", "name": "alba", "title": "a title" })
    );
}

/// Each of the amendment's three is its own absence, so a journal note is an
/// envelope carrying a journal note and nothing else.
#[test]
fn an_amendment_carries_only_the_fields_that_were_typed() {
    assert_eq!(
        update(
            "p".to_owned(),
            "bl-1".to_owned(),
            "alba".to_owned(),
            None,
            None,
            Some("what happened".to_owned()),
        ),
        json!({
            "op": "update", "project": "p", "id": "bl-1", "name": "alba",
            "note": "what happened",
        })
    );
    assert_eq!(
        update(
            "p".to_owned(),
            "bl-1".to_owned(),
            "alba".to_owned(),
            Some("t".to_owned()),
            Some("b".to_owned()),
            Some("n".to_owned()),
        ),
        json!({
            "op": "update", "project": "p", "id": "bl-1", "name": "alba",
            "title": "t", "body": "b", "note": "n",
        })
    );
}

/// **The bare form is what an amendment of nothing is**, and upstream refuses
/// it by name. This end can spell it, which is what keeps the round trip in
/// `crate::verbs::tests::corpus` honest about a frame the vocabulary carries.
#[test]
fn an_amendment_of_nothing_is_the_bare_envelope() {
    assert_eq!(
        update(
            "p".to_owned(),
            "bl-1".to_owned(),
            "alba".to_owned(),
            None,
            None,
            None,
        ),
        json!({ "op": "update", "project": "p", "id": "bl-1", "name": "alba" })
    );
}
