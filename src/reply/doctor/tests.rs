//! One doctor row: three required fields, the optional remedy carried verbatim,
//! the headline off them, and the count that is the only tally.

use super::{Check, KIND, failing, row};
use crate::reply::{Read, Reply, read};
use serde_json::json;

/// A failing row with a remedy, as the corpus carries one.
fn remedied() -> serde_json::Value {
    json!({
        "check": "address",
        "fact": "127.0.0.1:0 is a request, not an endpoint",
        "ok": false,
        "remedy": "`WIRE_HOST=<host> WIRE_PORT=<port> yog wire-certs` states it",
    })
}

#[test]
fn a_row_with_a_remedy_carries_it_verbatim() {
    let read = row(&remedied()).expect("a doctor row");
    assert_eq!(
        read,
        Check {
            check: "address".to_owned(),
            fact: "127.0.0.1:0 is a request, not an endpoint".to_owned(),
            ok: false,
            remedy: Some("`WIRE_HOST=<host> WIRE_PORT=<port> yog wire-certs` states it".to_owned()),
        }
    );
    assert_eq!(
        read.headline(),
        "NOT OK  address: 127.0.0.1:0 is a request, not an endpoint"
    );
}

/// **Absent is the answer**: a row with no remedy states none.
#[test]
fn a_row_without_a_remedy_states_none() {
    let frame = json!({"check": "listener", "fact": "listening", "ok": true});
    let read = row(&frame).expect("a doctor row");
    assert_eq!(read.remedy, None);
    assert_eq!(read.headline(), "ok  listener: listening");
}

/// **Rung 1 refuses by name.**
#[test]
fn every_required_field_refuses_by_name() {
    for field in ["check", "fact", "ok", "remedy"] {
        let mut frame = remedied();
        frame[field] = json!(7);
        let said = row(&frame).expect_err(field);
        assert!(said.contains(field), "{said}");
    }
    for field in ["check", "fact", "ok"] {
        let mut frame = remedied();
        frame.as_object_mut().expect("an object").remove(field);
        assert!(row(&frame).is_err(), "{field} is required");
    }
    let said = row(&json!("not an object")).expect_err("a row that is not one");
    assert!(said.contains("not a JSON object"), "{said}");
}

/// **The frame's `ok` is not a verdict over the rows** — the corpus's own frame
/// says `true` over a failing row — so the tally is a count of rows.
#[test]
fn the_frame_reads_as_the_rows_and_the_tally_is_counted() {
    let frame = json!({"kind": KIND, "ok": true, "rows": [
        {"check": "listener", "fact": "listening", "ok": true},
        remedied(),
    ]});
    let Read::Answer(Reply::Doctor(rows)) = read(&frame) else {
        panic!("a doctor frame is an answer: {:?}", read(&frame));
    };
    assert_eq!(rows.len(), 2);
    assert_eq!(failing(&rows), 1);
}
