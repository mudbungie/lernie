//! One role row: the four required fields, the one that is an option, and the
//! rung-3 reading that carries a level this seat has no word for.

use super::{KIND, RoleRow, row};
use crate::reply::{Read, Reply, read};
use serde_json::json;

/// A well-formed row, as the corpus carries one.
fn tuned() -> serde_json::Value {
    json!({
        "role": "worker",
        "provider": "housevendor",
        "model": "house-model-1",
        "priority": true,
        "effort": "high",
    })
}

#[test]
fn a_row_carries_the_role_what_it_runs_on_and_how_it_is_tuned() {
    let read = row(&tuned()).expect("a role row");
    assert_eq!(
        read,
        RoleRow {
            role: "worker".to_owned(),
            provider: "housevendor".to_owned(),
            model: "house-model-1".to_owned(),
            priority: true,
            effort: Some("high".to_owned()),
        }
    );
    assert_eq!(read.runs_on(), "housevendor  house-model-1");
}

/// **Rung 1 refuses by name**, and the name is the whole remedy.
#[test]
fn every_required_field_refuses_and_says_which_one() {
    for (field, why) in [
        ("role", "non-string"),
        ("provider", "non-string"),
        ("model", "non-string"),
        ("priority", "non-boolean"),
    ] {
        let mut frame = tuned();
        frame[field] = json!(7);
        let said = row(&frame).expect_err(field);
        assert!(said.contains(field), "{said}");
        assert!(said.contains(why), "{said}");
    }
    let said = row(&json!("not an object")).expect_err("a row that is not one");
    assert!(said.contains("not an object"), "{said}");
}

/// **The absence is a reading, not a gap.** Null and absent are both *no level
/// requested*, which is the one thing the wire has no word for.
#[test]
fn an_effort_that_is_null_or_absent_is_the_absence_and_not_a_refusal() {
    let mut nulled = tuned();
    nulled["effort"] = json!(null);
    assert_eq!(row(&nulled).expect("a null level").effort, None);
    let mut absent = tuned();
    absent.as_object_mut().expect("the row").remove("effort");
    assert_eq!(row(&absent).expect("an absent level").effort, None);
    let mut wrong = tuned();
    wrong["effort"] = json!(3);
    let said = row(&wrong).expect_err("a level of the wrong type still refuses");
    assert!(said.contains("effort"), "{said}");
}

/// **Rung 3: a level outside the closed set is carried verbatim.** The gesture
/// asserts one of four; this reports what the config file holds, and a file may
/// hold a word written by a hand or by a newer engine.
#[test]
fn a_level_this_seat_has_no_word_for_is_carried_rather_than_refused() {
    let mut frame = tuned();
    frame["effort"] = json!("extreme");
    assert_eq!(
        row(&frame).expect("an unknown level reads").effort,
        Some("extreme".to_owned())
    );
}

/// The whole frame, through the real door — and a listing that refuses on one
/// row refuses whole rather than shortening.
#[test]
fn the_frame_reads_as_the_listing_and_one_bad_row_fails_it_all() {
    let frame = json!({"kind": KIND, "ok": true, "rows": [tuned()]});
    let Read::Answer(Reply::Roles(rows)) = read(&frame) else {
        panic!("a roles frame is an answer: {:?}", read(&frame));
    };
    assert_eq!(rows.len(), 1);
    let mut broken = tuned();
    broken["role"] = json!(null);
    let frame = json!({"kind": KIND, "ok": true, "rows": [tuned(), broken]});
    assert!(matches!(read(&frame), Read::Unreadable(_)), "one bad row");
}
