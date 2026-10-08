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

/// One answer with `workflow_mark` spelled as given.
fn marked(mark: Option<serde_json::Value>) -> Result<super::Governing, String> {
    let mut answer = json!({
        "oid": "b", "short_oid": "b", "follows": "default",
        "diverged_lineages": 0, "files": []
    });
    if let Some(mark) = mark {
        answer["workflow_mark"] = mark;
    }
    governing(answer.as_object().expect("an object"))
}

/// **Null and absent are one reading** (REMOTE §9.24): the tip's own
/// `workflow.yaml` governs, and a reader that never heard of the key reads
/// the answer it always did.
#[test]
fn a_null_or_absent_mark_is_no_mark() {
    for mark in [None, Some(serde_json::Value::Null)] {
        assert_eq!(marked(mark).expect("reads").workflow_mark, None);
    }
}

/// **A present mark reads whole, and a null lineage is a pinned older commit**
/// — a reading, never a refusal.
#[test]
fn a_present_mark_reads_and_a_null_lineage_is_the_pinned_commit() {
    let read = marked(Some(json!({"holder": "r-0", "oid": "dddd",
                                  "short_oid": "dd", "lineage": "strict"})))
    .expect("reads");
    let mark = read.workflow_mark.expect("a mark");
    assert_eq!(mark.lineage.as_deref(), Some("strict"));
    assert_eq!(
        mark.line(),
        "workflow.yaml marked to config/strict at dd, on r-0"
    );
    let read = marked(Some(json!({"holder": "r-0-c-1", "oid": "eeee",
                                  "short_oid": "ee", "lineage": null})))
    .expect("reads");
    let mark = read.workflow_mark.expect("a mark");
    assert_eq!(mark.lineage, None);
    assert_eq!(mark.oid, "eeee");
    assert_eq!(
        mark.line(),
        "workflow.yaml pinned at ee, a commit its lineage has moved past, on r-0-c-1"
    );
}

/// A mark missing a stated key, or not an object, refuses by name.
#[test]
fn a_malformed_mark_refuses_and_names_the_field() {
    let why = marked(Some(json!({"oid": "d", "short_oid": "d", "lineage": null})))
        .expect_err("a mark with no holder refuses");
    assert!(why.contains("holder"), "{why}");
    let why = marked(Some(json!("strict"))).expect_err("a non-object mark refuses");
    assert!(why.contains("workflow_mark"), "{why}");
}
