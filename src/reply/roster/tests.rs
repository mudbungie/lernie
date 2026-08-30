//! The roster: the rollups, the pin rank's absence, the currency notes, and
//! rung 3 on the classification.

use super::{WorkspaceKind, WsRow, kind, row, workspaces};
use serde_json::{Value, json};

fn read(v: &Value) -> Result<super::Workspaces, String> {
    workspaces(v.as_object().expect("an object"))
}

/// The whole answer, with the rollups and both currency notes.
#[test]
fn a_roster_carries_its_rows_and_how_current_they_are() {
    let answered = read(&json!({
        "ok": true, "kind": "workspaces",
        "stale": "derivation 4m behind", "growth": "one grew 3 steps",
        "rows": [{
            "workspace": "home", "kind": "named",
            "attention": 2, "agents": 7, "running": true, "pinned": 0,
        }],
    }))
    .expect("a roster");
    assert_eq!(answered.stale.as_deref(), Some("derivation 4m behind"));
    assert_eq!(answered.growth.as_deref(), Some("one grew 3 steps"));
    assert_eq!(
        answered.rows,
        vec![WsRow {
            workspace: "home".to_owned(),
            kind: WorkspaceKind::Named,
            attention: 2,
            agents: 7,
            running: true,
            pinned: Some(0),
        }]
    );
}

/// **The ordinary answer says nothing about currency**, and that absence is
/// what makes "current" and "the engine declined to say" one reading. An empty
/// enumeration is an answer too.
#[test]
fn a_current_roster_states_no_note_and_an_empty_one_is_an_answer() {
    let answered = read(&json!({"ok": true, "kind": "workspaces", "rows": []})).expect("a roster");
    assert_eq!(answered, super::Workspaces::default());
}

/// **Rank 0 is the first hoisted row, so `pinned` is absent rather than a
/// rank of its own** — a reader must never have to tell the two apart.
#[test]
fn an_unpinned_row_states_no_rank_at_all() {
    let unpinned = row(&json!({"workspace": "scratch", "kind": "foreign",
                               "attention": 0, "agents": 1, "running": false}))
    .expect("a row");
    assert_eq!(unpinned.pinned, None);
    let nulled = row(
        &json!({"workspace": "scratch", "kind": "foreign", "pinned": null,
                             "attention": 0, "agents": 1, "running": false}),
    )
    .expect("a row");
    assert_eq!(nulled.pinned, None);
}

/// **Rung 3.** A classification this build does not know keeps its word and
/// paints it; it never becomes one of the three, because a replay painted as a
/// wall is a lie about what may be written to.
#[test]
fn an_unknown_classification_keeps_its_word() {
    for (word, expected) in [
        ("named", WorkspaceKind::Named),
        ("foreign", WorkspaceKind::Foreign),
        ("replay", WorkspaceKind::Replay),
        ("sealed", WorkspaceKind::Unknown("sealed".to_owned())),
    ] {
        assert_eq!(kind(word), expected);
        assert_eq!(kind(word).label(), word);
    }
}

/// Rung 1 still holds under rung 3: a row that is not an object, and a row
/// missing a rollup, both refuse by name.
#[test]
fn a_row_that_is_not_a_row_refuses() {
    assert_eq!(
        row(&json!("home")),
        Err("workspace row: not an object".to_owned())
    );
    let refusal =
        row(&json!({"workspace": "home", "kind": "named", "agents": 1, "running": false}))
            .expect_err("no attention");
    assert!(refusal.contains("\"attention\""), "{refusal}");
    let listing = read(&json!({"ok": true, "kind": "workspaces", "rows": [7]}))
        .expect_err("a row that is not one");
    assert!(listing.contains("not an object"), "{listing}");
}
