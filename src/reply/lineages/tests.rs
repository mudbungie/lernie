//! The lineage listing: the rows, and the oid this seat carries in both
//! spellings rather than abbreviating one.

use crate::reply::{Read, Reply, read};
use serde_json::json;

fn listing() -> serde_json::Value {
    json!({"ok": true, "kind": "lineages", "rows": [
        {"name": "main", "oid": "abcdef1234", "short_oid": "abcdef1",
         "committed": 1_700_000_000_i64, "files": ["workflow.yaml", "providers.yaml"]}]})
}

/// A row carries the branch, its tip in both spellings, when it was made, and
/// every path a config read on it may address.
#[test]
fn a_row_carries_the_branch_its_tip_and_the_paths_it_holds() {
    let Read::Answer(Reply::Lineages(rows)) = read(&listing()) else {
        panic!("a lineages answer: {:?}", read(&listing()));
    };
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].name, "main");
    assert_eq!(rows[0].oid, "abcdef1234");
    assert_eq!(rows[0].short_oid, "abcdef1");
    assert_eq!(rows[0].committed, 1_700_000_000);
    assert_eq!(rows[0].files, vec!["workflow.yaml", "providers.yaml"]);
    assert_eq!(rows[0].line(), "main  @abcdef1  2 file(s)");
}

/// Every field refuses by name, the file list included: a path that is not a
/// string is a listing this seat cannot index into.
#[test]
fn a_missing_field_refuses_and_names_itself() {
    for (frame, key) in [
        (
            json!({"ok": true, "kind": "lineages", "rows": [
                {"oid": "a", "short_oid": "b", "committed": 1, "files": []}]}),
            "name",
        ),
        (
            json!({"ok": true, "kind": "lineages", "rows": [
                {"name": "a", "short_oid": "b", "committed": 1, "files": []}]}),
            "oid",
        ),
        (
            json!({"ok": true, "kind": "lineages", "rows": [
                {"name": "a", "oid": "b", "short_oid": "c", "files": []}]}),
            "committed",
        ),
        (
            json!({"ok": true, "kind": "lineages", "rows": [
                {"name": "a", "oid": "b", "short_oid": "c", "committed": 1,
                 "files": [7]}]}),
            "a file is not a string",
        ),
        (
            json!({"ok": true, "kind": "lineages", "rows": ["not an object"]}),
            "not a JSON object",
        ),
    ] {
        let Read::Unreadable(why) = read(&frame) else {
            panic!("{key}: {frame}");
        };
        assert!(why.contains(key), "{key:?}: {why}");
    }
}
