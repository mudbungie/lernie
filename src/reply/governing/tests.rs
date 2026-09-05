//! The governing commit's reading: one enum rebuilt off one key, the engine's
//! two wordings, and the strictness that names a field.

use serde_json::json;

use super::{Governance, governing};

/// **The followed arm**: a lineage named, the count `0`, and the sentence
/// upstream writes for it.
#[test]
fn a_followed_conversation_names_its_lineage_and_wears_the_engines_sentence() {
    let read = governing(
        json!({
            "oid": "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb", "short_oid": "bbbbbbbb",
            "follows": "default", "diverged_lineages": 0,
            "files": ["workflow.yaml", "souls/base.md"]
        })
        .as_object()
        .expect("an object"),
    )
    .expect("a followed answer reads");
    assert_eq!(read.governance, Governance::Follows("default".to_owned()));
    assert_eq!(read.files, ["workflow.yaml", "souls/base.md"]);
    assert_eq!(
        read.label(),
        "policy follows config/default, now at bbbbbbbb"
    );
    assert_eq!(read.oid, "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb");
}

/// **The held arm is `follows: null`**, and the count is read only there — so
/// the pair cannot decode to a state the encoder could not have written.
#[test]
fn a_held_conversation_is_a_null_name_and_the_count_that_held_it() {
    let read = governing(
        json!({
            "oid": "cccccccccccccccccccccccccccccccccccccccc", "short_oid": "cccccccc",
            "follows": null, "diverged_lineages": 2, "files": []
        })
        .as_object()
        .expect("an object"),
    )
    .expect("a held answer reads");
    assert_eq!(read.governance, Governance::Held { diverged: 2 });
    assert!(read.files.is_empty());
    assert_eq!(
        read.label(),
        "policy held at cccccccc — 2 diverged config lineages"
    );
}

/// Rung 1: a missing or mistyped field refuses, and the refusal names it.
#[test]
fn a_malformed_answer_refuses_and_names_the_field() {
    let why = governing(json!({"follows": "x"}).as_object().expect("an object"))
        .expect_err("an answer with no oid refuses");
    assert!(why.contains("oid"), "{why}");
    let why = governing(
        json!({"oid": "a", "short_oid": "a", "follows": null, "files": []})
            .as_object()
            .expect("an object"),
    )
    .expect_err("a held answer with no count refuses");
    assert!(why.contains("diverged_lineages"), "{why}");
    let why = governing(
        json!({"oid": "a", "short_oid": "a", "follows": "d", "diverged_lineages": 0,
               "files": [7]})
        .as_object()
        .expect("an object"),
    )
    .expect_err("a non-string path refuses");
    assert!(why.contains("files"), "{why}");
}
