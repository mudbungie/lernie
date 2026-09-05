//! One sign-in run: the tagged lines, the two settled facts whose absence is a
//! reading, and the append the lane's frames are.

use super::{KIND, Line, Signin, signin};
use crate::reply::{Read, Reply, read};
use serde_json::{Value, json};

fn body(v: &Value) -> Result<Signin, String> {
    signin(v.as_object().expect("an object"))
}

/// A frame mid-run: both streams tagged, and nothing settled.
fn running() -> Value {
    json!({
        "kind": KIND,
        "ok": true,
        "lines": [
            {"text": "open https://provider.invalid/auth", "err": true},
            {"text": "{\"ready\":true}", "err": false},
        ],
    })
}

#[test]
fn a_frame_carries_both_streams_tagged_and_says_nothing_has_settled() {
    let read = body(&running()).expect("a run");
    assert_eq!(
        read.lines,
        vec![
            Line {
                text: "open https://provider.invalid/auth".to_owned(),
                err: true,
            },
            Line {
                text: "{\"ready\":true}".to_owned(),
                err: false,
            },
        ]
    );
    assert_eq!(read.outcome, None);
    assert_eq!(read.fallback, None);
    assert!(!read.settled());
}

/// **A pair nobody has signed in to is an empty frame, never silence** — the
/// lane opens on one, and it reads as the default rather than as a refusal.
#[test]
fn a_pair_with_no_run_is_an_empty_reading_and_not_a_refusal() {
    assert_eq!(
        body(&json!({"lines": [], "ok": true})).expect("an empty view"),
        Signin::default()
    );
}

/// **The settled facts are the exit and the command to retype**, and the
/// fallback exists only where the exit was non-zero.
#[test]
fn a_settled_run_carries_its_exit_and_the_command_to_run_by_hand() {
    let mut frame = running();
    frame["outcome"] = json!(78);
    frame["fallback"] = json!("yog exec --ws /ws bz --login --provider acme --browser");
    let read = body(&frame).expect("a settled run");
    assert_eq!(read.outcome, Some(78));
    assert_eq!(
        read.fallback.as_deref(),
        Some("yog exec --ws /ws bz --login --provider acme --browser")
    );
    assert!(read.settled());
}

/// **Rung 1 refuses by name**, on the lines and on both settled facts.
#[test]
fn every_field_refuses_by_name_and_an_outcome_no_i32_holds_refuses_too() {
    let mut no_lines = running();
    no_lines.as_object_mut().expect("the frame").remove("lines");
    let said = body(&no_lines).expect_err("the lines are required");
    assert!(said.contains("lines"), "{said}");
    for (field, why) in [("text", "non-string"), ("err", "non-boolean")] {
        let mut frame = running();
        frame["lines"][0][field] = json!(7);
        let said = body(&frame).expect_err(field);
        assert!(said.contains(field), "{said}");
        assert!(said.contains(why), "{said}");
    }
    let mut not_an_object = running();
    not_an_object["lines"][0] = json!("a line");
    let said = body(&not_an_object).expect_err("a line that is not an object");
    assert!(said.contains("not an object"), "{said}");
    let mut wrong = running();
    wrong["outcome"] = json!("78");
    let said = body(&wrong).expect_err("an outcome of the wrong type");
    assert!(said.contains("outcome"), "{said}");
    let mut wide = running();
    wide["outcome"] = json!(i64::from(i32::MAX) + 1);
    let said = body(&wide).expect_err("an outcome no i32 holds");
    assert!(said.contains("out of range"), "{said}");
    let mut nulled = running();
    nulled["outcome"] = json!(null);
    assert_eq!(body(&nulled).expect("a null outcome").outcome, None);
}

/// **A frame is an append**, and the fold is the live tail's contract one noun
/// over: lines accrete in order, and a run that has settled stays settled.
#[test]
fn absorbing_a_later_frame_accretes_the_lines_and_keeps_what_settled() {
    let mut fold = body(&running()).expect("a run");
    let mut settled = json!({"lines": [{"text": "done", "err": false}], "outcome": 0});
    fold.absorb(body(&settled).expect("the last frame"));
    assert_eq!(fold.lines.len(), 3);
    assert_eq!(fold.outcome, Some(0));
    settled["lines"] = json!([]);
    settled
        .as_object_mut()
        .expect("the frame")
        .remove("outcome");
    fold.absorb(body(&settled).expect("a frame after the exit"));
    assert_eq!(
        fold.outcome,
        Some(0),
        "a later silence does not unsettle it"
    );
    fold.absorb(Signin {
        fallback: Some("run it by hand".to_owned()),
        ..Signin::default()
    });
    assert_eq!(fold.fallback.as_deref(), Some("run it by hand"));
}

/// The whole frame, through the real door.
#[test]
fn the_frame_reads_as_a_sign_in_run() {
    let Read::Answer(Reply::Login(run)) = read(&running()) else {
        panic!("a login frame is an answer: {:?}", read(&running()));
    };
    assert_eq!(run.lines.len(), 2);
}
