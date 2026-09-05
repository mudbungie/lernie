//! The board's reading: a whole row, the absences that are readings, the
//! fleet array that is absent rather than empty, and rung 1's refusals.

use serde_json::{Map, Value, json};

use super::{board, row};

/// The frame the corpus carries, as a map for the reader.
fn obj(value: &Value) -> Map<String, Value> {
    value.as_object().expect("an object").clone()
}

/// A claimed row, read field for field — every fact upstream writes on one.
#[test]
fn a_claimed_row_carries_its_column_its_holder_its_gates_and_its_figures() {
    let read = row(&json!({
        "claimant": "alba",
        "column": "gated",
        "drones": [{ "name": "Cobalt", "root_id": "c-1" }],
        "gates": [{ "id": "bl-gate", "mints": "close", "title": "g" }],
        "id": "bl-1",
        "parent": "bl-epic",
        "priority": -2,
        "project": "p",
        "rollup": {
            "attribution": { "kind": "workspace" },
            "tokens": { "cache_read": 0, "cache_write": 0, "input": 0, "output": 0, "total": 1 }
        },
        "spend": {
            "attribution": { "kind": "workspace" },
            "tokens": { "cache_read": 0, "cache_write": 0, "input": 0, "output": 0, "total": 2 }
        },
        "state": "bound",
        "title": "t",
        "workspace": "ws"
    }))
    .expect("a whole row reads");
    assert_eq!(read.id, "bl-1");
    assert_eq!(read.column, "gated");
    assert_eq!(read.state, "bound");
    assert_eq!(read.title, "t");
    assert_eq!(read.priority, -2);
    assert_eq!(read.project, "p");
    assert_eq!(read.workspace.as_deref(), Some("ws"));
    assert_eq!(read.claimant.as_deref(), Some("alba"));
    assert_eq!(read.parent.as_deref(), Some("bl-epic"));
    assert_eq!(read.gates.first().expect("a gate").mints, "close");
    assert_eq!(read.gates.first().expect("a gate").title, "g");
    assert_eq!(read.gates.first().expect("a gate").id, "bl-gate");
    assert_eq!(read.drones.first().expect("a drone").name, "Cobalt");
    assert_eq!(read.drones.first().expect("a drone").root_id, "c-1");
    assert_eq!(read.spend.expect("a spend").tokens.total, 2);
    assert_eq!(read.rollup.expect("a rollup").tokens.total, 1);
}

/// **Every absence is a reading.** A ball nobody holds names no claimant, no
/// wall and no epic, and it has no figure — four different claims from four
/// empty strings.
#[test]
fn an_unheld_row_names_no_holder_no_epic_and_no_figure() {
    let read = row(&json!({
        "column": "ready", "drones": [], "gates": [], "id": "bl-2",
        "priority": 0, "project": "p", "state": "ready", "title": "u"
    }))
    .expect("a bare row reads");
    assert!(read.workspace.is_none());
    assert!(read.claimant.is_none());
    assert!(read.parent.is_none());
    assert!(read.spend.is_none());
    assert!(read.rollup.is_none());
    assert!(read.gates.is_empty());
    assert!(read.drones.is_empty());
}

/// **The fleet is ABSENT rather than empty on a box running nothing**, so the
/// reader answers the same `Vec` either way — the one case where a `Vec` and
/// an `Option<Vec>` are not two claims.
#[test]
fn a_board_with_no_loop_answers_no_fleet_and_a_board_with_one_answers_it() {
    let quiet = board(&obj(&json!({ "kind": "board", "ok": true, "rows": [] })))
        .expect("a quiet board reads");
    assert!(quiet.fleet.is_empty());
    assert!(quiet.rows.is_empty());
    let armed = board(&obj(&json!({
        "fleet": [{
            "cap": 4, "ceiling": "over budget", "count": 1, "label": "1/4 drones",
            "project": "p", "room": true, "workspace": "ws"
        }],
        "kind": "board", "ok": true, "rows": []
    })))
    .expect("an armed board reads");
    let loop_ = armed.fleet.first().expect("one loop");
    assert_eq!(loop_.workspace, "ws");
    assert_eq!(loop_.project, "p");
    assert_eq!(loop_.cap, 4);
    assert_eq!(loop_.count, 1);
    assert!(loop_.room);
    assert_eq!(loop_.ceiling.as_deref(), Some("over budget"));
    assert_eq!(loop_.label, "1/4 drones");
}

/// A loop with no ceiling standing says so by leaving the key out.
#[test]
fn a_loop_under_no_ceiling_carries_none() {
    let read = board(&obj(&json!({
        "fleet": [{ "cap": 1, "count": 0, "label": "0/1", "project": "p", "room": true,
                    "workspace": "ws" }],
        "kind": "board", "ok": true, "rows": []
    })))
    .expect("a board reads");
    assert!(read.fleet.first().expect("one loop").ceiling.is_none());
}

/// Rung 1, and every refusal names its field — at each of the four depths this
/// answer has one.
#[test]
fn a_malformed_board_refuses_naming_what_was_wrong() {
    assert_eq!(
        row(&json!("row")),
        Err("board row: not an object".to_owned())
    );
    assert_eq!(
        row(
            &json!({ "column": "ready", "drones": [], "gates": [], "id": "b",
                     "priority": 0, "project": "p", "state": "s" })
        ),
        Err("missing or non-string field \"title\"".to_owned())
    );
    assert_eq!(
        row(
            &json!({ "column": "ready", "drones": [], "gates": ["g"], "id": "b",
                     "priority": 0, "project": "p", "state": "s", "title": "t" })
        ),
        Err("board gate: not an object".to_owned())
    );
    assert_eq!(
        row(
            &json!({ "column": "ready", "drones": ["d"], "gates": [], "id": "b",
                     "priority": 0, "project": "p", "state": "s", "title": "t" })
        ),
        Err("board drone: not an object".to_owned())
    );
    assert_eq!(
        row(
            &json!({ "column": "ready", "drones": [{ "root_id": "c" }], "gates": [], "id": "b",
                     "priority": 0, "project": "p", "state": "s", "title": "t" })
        ),
        Err("missing or non-string field \"name\"".to_owned())
    );
    assert_eq!(
        row(
            &json!({ "column": "ready", "drones": [], "gates": [{ "id": "g", "mints": "close" }],
                     "id": "b", "priority": 0, "project": "p", "state": "s", "title": "t" })
        ),
        Err("missing or non-string field \"title\"".to_owned())
    );
    assert_eq!(
        board(&obj(&json!({ "fleet": ["x"], "rows": [] }))),
        Err("fleet facts: not an object".to_owned())
    );
    assert_eq!(
        board(&obj(
            &json!({ "fleet": [{ "cap": 1, "count": 0, "project": "p", "room": true,
                                        "workspace": "ws" }], "rows": [] })
        )),
        Err("missing or non-string field \"label\"".to_owned())
    );
    assert_eq!(
        board(&obj(
            &json!({ "fleet": [{ "cap": 1, "count": 0, "label": "l", "project": "p",
                                        "workspace": "ws" }], "rows": [] })
        )),
        Err("missing or non-boolean field \"room\"".to_owned())
    );
}
