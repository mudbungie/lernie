//! The diff row's reading: three states, the two churn shapes, and the
//! absences that are readings.

use serde_json::json;

use super::diff;

/// A `diff` row, read field for field — every fact upstream writes on one.
#[test]
fn a_changed_row_carries_both_refs_both_oids_and_its_files() {
    let read = diff(&json!({
        "ball_id": "bl-3", "delivered": "ccc",
        "files": [
            { "added": 3, "path": "src/a.rs", "removed": 1 },
            { "binary": true, "path": "assets/x.png" }
        ],
        "handle": "at-0badcafe", "project": "p", "source": "attempt/at-0badcafe",
        "source_oid": "ddd", "state": "diff", "target": "work/bl-3",
        "target_oid": "ccc", "truncated": false
    }))
    .expect("a whole row reads");
    assert_eq!(read.ball_id, "bl-3");
    assert_eq!(read.project, "p");
    assert_eq!(read.state, "diff");
    assert_eq!(read.handle.as_deref(), Some("at-0badcafe"));
    assert_eq!(read.delivered.as_deref(), Some("ccc"));
    assert_eq!(read.source.as_deref(), Some("attempt/at-0badcafe"));
    assert_eq!(read.target.as_deref(), Some("work/bl-3"));
    assert_eq!(read.source_oid.as_deref(), Some("ddd"));
    assert_eq!(read.target_oid.as_deref(), Some("ccc"));
    assert_eq!(read.truncated, Some(false));
    assert!(read.missing.is_empty());
    let text = read.files.first().expect("a text churn");
    assert_eq!(text.path, "src/a.rs");
    assert_eq!(text.added, Some(3));
    assert_eq!(text.removed, Some(1));
    assert!(text.binary.is_none());
    let binary = read.files.get(1).expect("a binary churn");
    assert_eq!(binary.binary, Some(true));
    assert!(binary.added.is_none());
}

/// **A ref that is not there yet is its own state**, and it names what is
/// missing rather than showing an empty file list.
#[test]
fn an_absent_row_names_the_refs_that_are_not_there() {
    let read = diff(&json!({
        "ball_id": "bl-2", "missing": ["work/bl-2"], "project": "p",
        "source": "work/bl-2", "state": "absent", "target": "main"
    }))
    .expect("an absent row reads");
    assert_eq!(read.state, "absent");
    assert_eq!(read.missing, vec!["work/bl-2".to_owned()]);
    assert!(read.files.is_empty());
    assert!(read.truncated.is_none());
}

/// **A project the engine could not read says so and says nothing else**, and
/// the word rides verbatim — so a fourth state upstream grows paints as itself
/// rather than refusing the listing ([`super::super`]'s rung 3).
#[test]
fn an_unreadable_row_says_only_that_and_an_unknown_state_rides_too() {
    let read = diff(&json!({ "ball_id": "bl-1", "project": "p", "state": "unreadable" }))
        .expect("an unreadable row reads");
    assert_eq!(read.state, "unreadable");
    assert!(read.source.is_none());
    assert!(read.target.is_none());
    assert!(read.handle.is_none());
    assert!(read.delivered.is_none());
    let grown = diff(&json!({ "ball_id": "bl-1", "project": "p", "state": "quarantined" }))
        .expect("a state this build has never seen reads");
    assert_eq!(grown.state, "quarantined");
}

/// Rung 1, and every refusal names its field.
#[test]
fn a_malformed_row_refuses_naming_what_was_wrong() {
    assert_eq!(
        diff(&json!("row")),
        Err("attempt: not an object".to_owned())
    );
    assert_eq!(
        diff(&json!({ "ball_id": "b" })),
        Err("missing or non-string field \"project\"".to_owned())
    );
    assert_eq!(
        diff(&json!({ "ball_id": "b", "files": ["f"], "project": "p", "state": "diff" })),
        Err("changed file: not an object".to_owned())
    );
    assert_eq!(
        diff(
            &json!({ "ball_id": "b", "files": [{ "added": "3", "path": "a" }],
                      "project": "p", "state": "diff" })
        ),
        Err("missing or non-integer field \"added\"".to_owned())
    );
    assert_eq!(
        diff(&json!({ "ball_id": "b", "missing": [7], "project": "p", "state": "absent" })),
        Err("field \"missing\": a non-string element".to_owned())
    );
    assert_eq!(
        diff(&json!({ "ball_id": "b", "project": "p", "state": "diff", "truncated": "no" })),
        Err("missing or non-boolean field \"truncated\"".to_owned())
    );
}
