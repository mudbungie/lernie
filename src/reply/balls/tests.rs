//! The balls family's three readings: the binding row and its three absences,
//! the bound ball and its required figure, and the branch that is one field.

use serde_json::{Map, Value, json};

use super::{bound, marks, row};

/// The frame as a map for the readers that take one.
fn obj(value: &Value) -> Map<String, Value> {
    value.as_object().expect("an object").clone()
}

/// A whole binding row, read field for field.
#[test]
fn a_binding_row_says_which_ball_is_held_by_whom_and_where() {
    let read = row(&json!({
        "ball_id": "bl-1", "claimant": "alba", "project": "p",
        "state": "delivered", "title": "t", "workspace": "ws"
    }))
    .expect("a whole row reads");
    assert_eq!(read.ball_id, "bl-1");
    assert_eq!(read.project, "p");
    assert_eq!(read.state, "delivered");
    assert_eq!(read.title.as_deref(), Some("t"));
    assert_eq!(read.claimant.as_deref(), Some("alba"));
    assert_eq!(read.workspace.as_deref(), Some("ws"));
}

/// **Three absences, three readings.** A ball nobody holds has no claimant and
/// no wall, and a ball whose title the store could not read has none — each a
/// different claim from an empty string.
#[test]
fn an_unheld_binding_names_no_claimant_no_wall_and_may_name_no_title() {
    let read = row(&json!({ "ball_id": "bl-2", "project": "p", "state": "ready" }))
        .expect("a bare row reads");
    assert!(read.title.is_none());
    assert!(read.claimant.is_none());
    assert!(read.workspace.is_none());
}

/// A bound ball, read field for field. Its figure is REQUIRED — that is the
/// division upstream draws between the join and this listing.
#[test]
fn a_bound_ball_carries_its_badge_its_owner_and_what_it_cost() {
    let read = bound(&json!({
        "badge": "delivered", "id": "bl-1", "owner": "alba", "project": "p",
        "spend": {
            "attribution": { "kind": "workspace" },
            "tokens": { "cache_read": 0, "cache_write": 0, "input": 12, "output": 0, "total": 12 }
        },
        "state": "delivered"
    }))
    .expect("a whole ball reads");
    assert_eq!(read.id, "bl-1");
    assert_eq!(read.badge.as_deref(), Some("delivered"));
    assert_eq!(read.project, "p");
    assert_eq!(read.owner, "alba");
    assert_eq!(read.state, "delivered");
    assert_eq!(read.spend.tokens.total, 12);
}

/// **No badge is a reading**: a state that needs none says so by leaving the
/// key out, which is the roster's own spelling of nothing to say.
#[test]
fn a_bound_ball_whose_state_needs_no_badge_carries_none() {
    let read = bound(&json!({
        "id": "bl-2", "owner": "alba", "project": "p",
        "spend": {
            "attribution": { "kind": "workspace" },
            "tokens": { "cache_read": 0, "cache_write": 0, "input": 0, "output": 0, "total": 0 }
        },
        "state": "bound"
    }))
    .expect("a badgeless ball reads");
    assert!(read.badge.is_none());
}

/// The branch is the whole of that answer, and it is the branch re-read.
#[test]
fn the_marks_answer_is_the_branch() {
    assert_eq!(
        marks(&obj(
            &json!({ "branch": "marks/alba", "kind": "marks", "ok": true })
        )),
        Ok("marks/alba".to_owned())
    );
}

/// Rung 1, and every refusal names its field.
#[test]
fn a_malformed_answer_refuses_naming_what_was_wrong() {
    assert_eq!(
        row(&json!("row")),
        Err("balls row: not an object".to_owned())
    );
    assert_eq!(
        row(&json!({ "ball_id": "b", "project": "p" })),
        Err("missing or non-string field \"state\"".to_owned())
    );
    assert_eq!(
        bound(&json!("row")),
        Err("ball row: not an object".to_owned())
    );
    assert_eq!(
        bound(&json!({ "id": "b", "owner": "a", "project": "p", "state": "s" })),
        Err("ball row: missing spend".to_owned())
    );
    assert_eq!(
        marks(&obj(&json!({ "ok": true }))),
        Err("missing or non-string field \"branch\"".to_owned())
    );
}
