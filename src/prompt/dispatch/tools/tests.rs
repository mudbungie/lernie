//! Unit tests for the role-tools composer (ARCH §3.3, §4.3).

use super::*;
use crate::prompt::Error;
use serde_json::json;
use std::fs;
use tempfile::TempDir;

/// Write `descriptions/tools/<name>.json` in `worktree` with `body`.
fn write_schema(worktree: &Path, name: &str, body: &str) {
    let dir = worktree.join(TOOLS_DESC_DIR);
    fs::create_dir_all(&dir).unwrap();
    fs::write(dir.join(format!("{name}.json")), body).unwrap();
}

const BASH_SCHEMA: &str = r#"{
  "type": "object",
  "properties": { "command": { "type": "string" } },
  "required": ["command"]
}"#;

#[test]
fn declared_tool_with_schema_carries_it_verbatim() {
    let wt = TempDir::new().unwrap();
    write_schema(wt.path(), "bash", BASH_SCHEMA);

    let tools = compose(wt.path(), &["bash".to_string()]).unwrap();

    assert_eq!(tools.len(), 1);
    assert_eq!(tools[0].name, "bash");
    // §3.3: the schema file is the `input_schema`, verbatim.
    assert_eq!(
        tools[0].input_schema,
        serde_json::from_str::<Value>(BASH_SCHEMA).unwrap()
    );
    // description sourcing (SKILL.md frontmatter) is not yet wired.
    assert_eq!(tools[0].description, None);
}

#[test]
fn empty_declaration_yields_empty_tools() {
    let wt = TempDir::new().unwrap();
    write_schema(wt.path(), "bash", BASH_SCHEMA);
    assert!(compose(wt.path(), &[]).unwrap().is_empty());
}

#[test]
fn declared_tool_without_schema_is_dropped() {
    let wt = TempDir::new().unwrap();
    // No descriptions/tools dir at all — the intersection is empty.
    assert!(
        compose(wt.path(), &["bash".to_string()])
            .unwrap()
            .is_empty()
    );
}

#[test]
fn intersection_keeps_only_available_schemas_in_declared_order() {
    let wt = TempDir::new().unwrap();
    write_schema(wt.path(), "read_file", r#"{"type":"object"}"#);
    write_schema(wt.path(), "bash", BASH_SCHEMA);

    let declared = [
        "bash".to_string(),
        "missing".to_string(),
        "read_file".to_string(),
    ];
    let tools = compose(wt.path(), &declared).unwrap();

    // `missing` has no schema and is dropped; the rest keep declared order.
    let names: Vec<&str> = tools.iter().map(|t| t.name.as_str()).collect();
    assert_eq!(names, vec!["bash", "read_file"]);
}

#[test]
fn present_but_malformed_schema_is_a_hard_error() {
    let wt = TempDir::new().unwrap();
    write_schema(wt.path(), "bash", "{ not json");
    let err = compose(wt.path(), &["bash".to_string()]).unwrap_err();
    match err {
        Error::ToolSchemaJson { name, .. } => assert_eq!(name, "bash"),
        other => panic!("expected ToolSchemaJson, got {other:?}"),
    }
}

#[test]
fn unreadable_schema_file_surfaces_io_error() {
    let wt = TempDir::new().unwrap();
    // A directory where a `<name>.json` file is expected: `read` fails
    // with a non-NotFound error, which is surfaced (not dropped).
    fs::create_dir_all(wt.path().join(TOOLS_DESC_DIR).join("bash.json")).unwrap();
    let err = compose(wt.path(), &["bash".to_string()]).unwrap_err();
    match err {
        Error::ToolSchemaIo { name, .. } => assert_eq!(name, "bash"),
        other => panic!("expected ToolSchemaIo, got {other:?}"),
    }
}

#[test]
fn schema_value_shape_is_preserved() {
    let wt = TempDir::new().unwrap();
    write_schema(
        wt.path(),
        "x",
        r#"{"type":"object","properties":{"a":{"type":"number"}}}"#,
    );
    let tools = compose(wt.path(), &["x".to_string()]).unwrap();
    assert_eq!(
        tools[0].input_schema,
        json!({"type":"object","properties":{"a":{"type":"number"}}})
    );
}
