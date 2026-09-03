//! One help row: the five required fields, the headline computed off them, and
//! the classification carried verbatim.

use super::{HelpRow, KIND, row};
use crate::reply::{Read, Reply, read};
use serde_json::json;

/// A well-formed row, as the corpus carries one.
fn published() -> serde_json::Value {
    json!({
        "verb": "scan",
        "usage": "/scan",
        "summary": "flush the focused workspace's inboxes",
        "detail": "One workspace-wide sweep, delivering pending mail.",
        "surface": "control",
    })
}

#[test]
fn a_row_carries_the_op_its_line_its_sentence_its_page_and_who_it_is_for() {
    let read = row(&published()).expect("a help row");
    assert_eq!(
        read,
        HelpRow {
            verb: "scan".to_owned(),
            usage: "/scan".to_owned(),
            summary: "flush the focused workspace's inboxes".to_owned(),
            detail: "One workspace-wide sweep, delivering pending mail.".to_owned(),
            surface: "control".to_owned(),
        }
    );
    assert_eq!(read.headline(), "/scan  [control]");
}

/// **Rung 1 refuses by name**, and the name is the whole remedy.
#[test]
fn every_field_is_required_and_the_refusal_says_which_one() {
    for field in ["verb", "usage", "summary", "detail", "surface"] {
        let mut frame = published();
        frame[field] = json!(7);
        let said = row(&frame).expect_err(field);
        assert!(said.contains(field), "{said}");
        assert!(said.contains("non-string"), "{said}");
    }
    let said = row(&json!("not an object")).expect_err("a row that is not one");
    assert!(said.contains("not an object"), "{said}");
}

/// **Rung 3: a classification this seat has no reading for is carried
/// verbatim**, because a pane only says what an op is for. The parity roster
/// refuses the same word on purpose — it decides what this seat OWES, and
/// guessing there would quietly shrink the obligation
/// (`crate::snapshot::parity::roster::classify`).
#[test]
fn a_surface_word_this_build_has_never_seen_paints_as_itself() {
    let mut frame = published();
    frame["surface"] = json!("operator-only");
    let read = row(&frame).expect("an unknown classification reads");
    assert_eq!(read.surface, "operator-only");
    assert_eq!(read.headline(), "/scan  [operator-only]");
    assert!(
        crate::snapshot::parity::roster::classify(&frame).is_err(),
        "the roster refuses what the pane carries"
    );
}

/// The whole frame, through the real door — and a listing that refuses on one
/// row refuses whole rather than shortening.
#[test]
fn the_frame_reads_as_the_listing_and_one_bad_row_fails_it_all() {
    let frame = json!({"kind": KIND, "ok": true, "rows": [published()]});
    let Read::Answer(Reply::Help(rows)) = read(&frame) else {
        panic!("a help frame is an answer: {:?}", read(&frame));
    };
    assert_eq!(rows.len(), 1);
    let mut broken = published();
    broken["verb"] = json!(null);
    let frame = json!({"kind": KIND, "ok": true, "rows": [published(), broken]});
    assert!(matches!(read(&frame), Read::Unreadable(_)), "one bad row");
}
