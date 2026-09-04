//! The steps listing: the strict row, the nested spend, the absences that are
//! readings, and the class tokens carried verbatim.

use super::{KIND, NONE, Spend, StepRow, steps};
use crate::reply::{Read, Reply, read};
use serde_json::json;

/// A well-formed row with everything present, as the corpus carries one.
fn full() -> serde_json::Value {
    json!({
        "seq": "001", "framing": "complete", "attempts": 1,
        "tokens": {"input": 11, "output": 22, "cache_read": 33,
                   "cache_write": 44, "total": 99},
        "commit": "abc", "started_at": "t0", "ended_at": "t1",
        "auth_row": "housevendor",
        "wound": "no_response", "wound_reason": "no bytes",
    })
}

#[test]
fn a_row_carries_the_summary_whole() {
    let read = super::row(&full()).expect("a step row");
    assert_eq!(
        read,
        StepRow {
            seq: "001".to_owned(),
            framing: "complete".to_owned(),
            attempts: 1,
            tokens: Spend {
                input: 11,
                output: 22,
                cache_read: 33,
                cache_write: 44,
                total: 99
            },
            commit: Some("abc".to_owned()),
            started_at: Some("t0".to_owned()),
            ended_at: Some("t1".to_owned()),
            auth_row: Some("housevendor".to_owned()),
            wound: "no_response".to_owned(),
            wound_reason: Some("no bytes".to_owned()),
        }
    );
}

/// **Rung 1 refuses by name**, on the row's own fields and on the nested
/// spend's — and a row that is not an object says so.
#[test]
fn every_required_field_refuses_and_says_which_one() {
    for (field, why) in [
        ("seq", "non-string"),
        ("framing", "non-string"),
        ("wound", "non-string"),
        ("attempts", "non-integer"),
        ("tokens", "non-object"),
    ] {
        let mut row = full();
        row[field] = json!([]);
        let said = super::row(&row).expect_err(field);
        assert!(said.contains(field), "{said}");
        assert!(said.contains(why), "{said}");
    }
    let mut counter = full();
    counter["tokens"]["total"] = json!("many");
    let said = super::row(&counter).expect_err("a non-integer counter");
    assert!(said.contains("total"), "{said}");
    let said = super::row(&json!(7)).expect_err("a row that is not an object");
    assert!(said.contains("not an object"), "{said}");
}

/// **An absent key is a fact nobody recorded**, never a refusal: the
/// timestamps, the commit, the auth row and the two reasons all read `None`.
#[test]
fn the_absences_are_readings() {
    let bare = json!({
        "seq": "002", "framing": "failed", "attempts": 2,
        "tokens": {"input": 0, "output": 0, "cache_read": 0,
                   "cache_write": 0, "total": 0},
        "wound": NONE,
    });
    let read = super::row(&bare).expect("a bare row");
    assert_eq!(read.commit, None);
    assert_eq!(read.started_at, None);
    assert_eq!(read.ended_at, None);
    assert_eq!(read.auth_row, None);
    assert_eq!(read.wound_reason, None);
    assert_eq!(read.wound, NONE);
}

/// The whole frame through the real door, with the view-level orphan beside
/// its reason — and without it, where the class left no words.
#[test]
fn the_frame_reads_as_the_listing_with_the_orphan_at_the_top() {
    let frame = json!({"kind": KIND, "ok": true, "rows": [full()],
                       "orphan": "mail", "orphan_reason": "driver died"});
    let Read::Answer(Reply::Steps(listing)) = read(&frame) else {
        panic!("a steps frame is an answer: {:?}", read(&frame));
    };
    assert_eq!(listing.rows.len(), 1);
    assert_eq!(listing.orphan, "mail");
    assert_eq!(listing.orphan_reason, Some("driver died".to_owned()));
    let quiet = json!({"kind": KIND, "ok": true, "rows": [], "orphan": NONE});
    let Read::Answer(Reply::Steps(listing)) = read(&quiet) else {
        panic!("a quiet steps frame is an answer");
    };
    assert!(listing.rows.is_empty());
    assert_eq!(listing.orphan_reason, None);
}

/// A missing view-level field refuses the frame, and one bad row fails the
/// listing whole rather than shortening it.
#[test]
fn the_listing_refuses_whole() {
    let unorphaned = json!({"kind": KIND, "ok": true, "rows": []});
    let Read::Unreadable(said) = read(&unorphaned) else {
        panic!("a steps frame with no orphan state is unreadable");
    };
    assert!(said.contains("orphan"), "{said}");
    let broken = json!({"kind": KIND, "ok": true, "orphan": NONE,
                        "rows": [full(), json!("not a row")]});
    assert!(matches!(read(&broken), Read::Unreadable(_)), "one bad row");
}

/// The steps() reader is reachable directly too — the same strictness the
/// dispatch spends, asserted at the door the dispatch calls.
#[test]
fn the_reader_reads_the_object_the_dispatch_hands_it() {
    let obj = json!({"rows": [], "orphan": "tool_window"});
    let listing = steps(obj.as_object().expect("an object")).expect("a listing");
    assert_eq!(listing.orphan, "tool_window");
}
