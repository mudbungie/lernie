//! Which rows are asked, what is filed, and which row is marked.

use crate::reply::{Read, Reply};
use crate::test_support::window::{role, tuned};
use crate::ui::{Edit, Tuning};

/// **Every role's row and the draft's, once each** — and a blank draft asks
/// nothing.
#[test]
fn the_rows_asked_are_the_roles_and_the_draft_once_each() {
    let mut model = tuned();
    assert_eq!(model.tuned(), vec!["housevendor".to_owned()]);
    let mut edit = Edit::of(&role("worker"));
    edit.provider = " othervendor ".to_owned();
    model.tuning = Some(Tuning::Editing(edit.clone()));
    assert_eq!(model.tuned(), vec!["housevendor", "othervendor"]);
    edit.provider = String::new();
    model.tuning = Some(Tuning::Editing(edit));
    assert_eq!(model.tuned(), vec!["housevendor".to_owned()]);
}

/// **A listing that lacks the model marks the row; no listing marks
/// nothing** — and a refusal after a listing withdraws it, because a row that
/// stopped answering stopped being evidence.
#[test]
fn only_a_listing_in_hand_marks_a_row_and_silence_withdraws_it() {
    let mut model = tuned();
    let row = role("worker");
    assert_eq!(model.unoffered(&row), None, "no listing, no mark");
    let listed = |ids: &[&str]| {
        Read::Answer(Reply::Models(
            ids.iter().map(|id| (*id).to_owned()).collect(),
        ))
    };
    model.offering("housevendor", listed(&["house-model-2"]));
    assert_eq!(
        model.unoffered(&row).as_deref(),
        Some("housevendor does not offer house-model-1")
    );
    assert_eq!(model.offered_by(" housevendor"), ["house-model-2"]);
    model.offering("housevendor", listed(&["house-model-1"]));
    assert_eq!(model.unoffered(&row), None, "a listing that holds it");
    model.offering("housevendor", listed(&[]));
    assert!(
        model.offered_by("housevendor").is_empty(),
        "empty is silence"
    );
    assert_eq!(model.unoffered(&row), None);
}
