//! The staged proposals: the row, the two depths one op answers at, and the
//! standing the engine reads for this seat rather than leaving it to infer.

use crate::reply::{Read, Reply, read};
use serde_json::{Value, json};

fn row() -> Value {
    json!({"id": "20260906T090000Z-r001", "lineages": ["default"], "parent": "9f2c1ab4",
           "fresh": true, "diffstat": "1 file changed, 6 insertions(+)",
           "subject": "notes: record what the span taught"})
}

fn listing(whole: Option<&str>) -> Value {
    let mut frame = json!({"ok": true, "kind": "proposals", "rows": [row()]});
    if let Some(text) = whole {
        frame["whole"] = json!(text);
    }
    frame
}

fn answered(frame: &Value) -> crate::reply::proposals::Proposals {
    match read(frame) {
        Read::Answer(Reply::Proposals(it)) => it,
        other => panic!("a proposals answer: {other:?}"),
    }
}

/// The row carries the reviewer, the lineage it would move, the commit it was
/// written against, how much it changes and what the reviewer called it.
#[test]
fn a_row_carries_the_reviewer_the_lineage_and_the_size_of_the_patch() {
    let answer = answered(&listing(None));
    assert_eq!(answer.rows.len(), 1);
    let row = &answer.rows[0];
    assert_eq!(row.id, "20260906T090000Z-r001");
    assert_eq!(row.lineages, vec!["default"]);
    assert_eq!(row.parent, "9f2c1ab4");
    assert_eq!(row.diffstat, "1 file changed, 6 insertions(+)");
    assert_eq!(row.subject, "notes: record what the span taught");
    assert_eq!(row.standing(), "fresh");
    assert_eq!(row.moves(), "default");
}

/// **`whole` is absent for the bare listing and present when the read named a
/// proposal** — one op at two depths, and the absence is the fact rather than
/// an empty string standing in for a patch nobody asked for.
#[test]
fn naming_a_proposal_answers_it_whole_beside_the_listing() {
    assert_eq!(answered(&listing(None)).whole, None);
    assert_eq!(
        answered(&listing(Some("commit 71011c3d\n\n+ a line\n"))).whole,
        Some("commit 71011c3d\n\n+ a line\n".to_owned())
    );
}

/// **An empty `lineages` is the stale case and says so.** Nothing is inferred
/// from it: `fresh` is the engine's own answer, read in the pass that read the
/// lineages, and the seat paints the sentence rather than a blank column.
#[test]
fn a_stale_proposal_names_its_standing_and_what_it_no_longer_moves() {
    let mut stale = row();
    stale["fresh"] = json!(false);
    stale["lineages"] = json!([]);
    let answer = answered(&json!({"ok": true, "kind": "proposals", "rows": [stale]}));
    assert_eq!(answer.rows[0].standing(), "stale");
    assert_eq!(
        answer.rows[0].moves(),
        "no lineage still heads at its parent"
    );
}

/// Every field refuses by name, and a row that is not an object refuses as
/// one — a listing this seat cannot address is not a listing.
#[test]
fn a_missing_field_refuses_and_names_itself() {
    for key in ["id", "lineages", "parent", "fresh", "diffstat", "subject"] {
        let mut short = row();
        short.as_object_mut().expect("an object").remove(key);
        let frame = json!({"ok": true, "kind": "proposals", "rows": [short]});
        let Read::Unreadable(said) = read(&frame) else {
            panic!("{key} must refuse: {:?}", read(&frame));
        };
        assert!(said.contains(key), "{key} unnamed in {said}");
    }
    let frame = json!({"ok": true, "kind": "proposals", "rows": ["not an object"]});
    let Read::Unreadable(said) = read(&frame) else {
        panic!("a row that is not an object must refuse");
    };
    assert!(said.contains("not a JSON object"), "{said}");
}
