//! The files answer: the discriminant that keeps "torn down" and "empty" two
//! claims, the strict row, and the preview's four arms.

use super::{FileRow, KIND, Listing, Preview, files};
use crate::reply::{Read, Reply, read};
use serde_json::json;

/// A full frame: a listing, a preview, and a working directory elsewhere.
fn walked() -> serde_json::Value {
    json!({
        "kind": KIND, "ok": true, "worktree": true,
        "rows": [{"path": "src/a.rs", "size": 12, "dir": false}],
        "truncated": true,
        "preview": {"kind": "text", "text": "body"},
        "working_dir": "/home/u/proj",
    })
}

#[test]
fn a_walked_worktree_reads_whole() {
    let Read::Answer(Reply::Files(answer)) = read(&walked()) else {
        panic!("a files frame is an answer: {:?}", read(&walked()));
    };
    assert_eq!(
        answer.listing,
        Some(Listing {
            rows: vec![FileRow {
                path: "src/a.rs".to_owned(),
                size: 12,
                dir: false
            }],
            truncated: true,
        })
    );
    assert_eq!(answer.preview, Some(Preview::Text("body".to_owned())));
    assert_eq!(answer.working_dir, Some("/home/u/proj".to_owned()));
}

/// **The worktree's absence is a fact**: no listing, and never an empty one —
/// while the preview and the working directory still read where present.
#[test]
fn an_absent_worktree_is_no_listing_rather_than_an_empty_one() {
    let torn = json!({"kind": KIND, "ok": true, "worktree": false,
                      "preview": {"kind": "binary", "size": 4}});
    let Read::Answer(Reply::Files(answer)) = read(&torn) else {
        panic!("an absent worktree is still an answer");
    };
    assert_eq!(answer.listing, None);
    assert_eq!(answer.preview, Some(Preview::Binary { size: 4 }));
    assert_eq!(answer.working_dir, None);
}

/// **Rung 1 refuses by name** on the frame, the row and the preview alike.
#[test]
fn every_required_field_refuses_and_says_which_one() {
    for field in ["worktree", "truncated", "rows"] {
        let mut frame = walked();
        frame[field] = json!(7);
        let Read::Unreadable(said) = read(&frame) else {
            panic!("a broken {field} refuses");
        };
        assert!(said.contains(field), "{said}");
    }
    for (row, names) in [
        (json!(7), "not an object"),
        (json!({"path": "p", "size": 1}), "dir"),
        (json!({"path": "p", "dir": true}), "size"),
        (json!({"size": 1, "dir": true}), "path"),
    ] {
        let mut frame = walked();
        frame["rows"] = json!([row]);
        let Read::Unreadable(said) = read(&frame) else {
            panic!("a broken row refuses: {names}");
        };
        assert!(said.contains(names), "{said}");
    }
}

/// The preview's other three arms: the bounded head with its true size, the
/// shape refusals, and — rung 3 — a class this build has no word for, carried
/// verbatim rather than refused over a readable listing.
#[test]
fn the_preview_reads_all_four_arms() {
    let mut frame = walked();
    frame["preview"] = json!({"kind": "truncated", "text": "head", "size": 999});
    let Read::Answer(Reply::Files(answer)) = read(&frame) else {
        panic!("a truncated preview reads");
    };
    assert_eq!(
        answer.preview,
        Some(Preview::Truncated {
            text: "head".to_owned(),
            size: 999
        })
    );
    frame["preview"] = json!({"kind": "hologram"});
    let Read::Answer(Reply::Files(answer)) = read(&frame) else {
        panic!("an unknown preview class is rung 3, not a refusal");
    };
    assert_eq!(
        answer.preview,
        Some(Preview::Unknown("hologram".to_owned()))
    );
    frame["preview"] = json!(7);
    let Read::Unreadable(said) = read(&frame) else {
        panic!("a preview that is not an object refuses");
    };
    assert!(said.contains("not an object"), "{said}");
    frame["preview"] = json!({"kind": "text"});
    let Read::Unreadable(said) = read(&frame) else {
        panic!("a text preview with no text refuses");
    };
    assert!(said.contains("text"), "{said}");
}

/// The reader is reachable at the door the dispatch calls.
#[test]
fn the_reader_reads_the_object_the_dispatch_hands_it() {
    let obj = json!({"worktree": true, "rows": [], "truncated": false});
    let answer = files(obj.as_object().expect("an object")).expect("an answer");
    assert_eq!(
        answer.listing,
        Some(Listing {
            rows: Vec::new(),
            truncated: false
        })
    );
}
