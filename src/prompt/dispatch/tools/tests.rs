//! Unit tests for the role-tools composer (ARCH §3.3, §4.3).

use super::*;
use crate::prompt::Error;

/// The ordinary role: no procedure injects a toolset for it (§2.7).
const WORKER: &str = crate::prompt::WORKER_ROLE;
use serde_json::json;
use std::fs;
use tempfile::TempDir;

/// Write `descriptions/tools/<name>.json` in `worktree` with `body`.
fn write_schema(worktree: &Path, name: &str, body: &str) {
    let dir = worktree.join(TOOLS_DESC_DIR);
    fs::create_dir_all(&dir).unwrap();
    fs::write(dir.join(format!("{name}.json")), body).unwrap();
}

/// Write `descriptions/skills/<name>.md` in `worktree` with a frontmatter
/// `body` (the shape the descriptions-always producer emits).
fn write_skill(worktree: &Path, name: &str, frontmatter_body: &str) {
    let dir = worktree.join(SKILLS_DESC_DIR);
    fs::create_dir_all(&dir).unwrap();
    fs::write(dir.join(format!("{name}.md")), frontmatter_body).unwrap();
}

/// Destructure a composed `Tool::Custom` into (name, description, input_schema).
/// The composer only ever emits `Custom` tools (§3.3), so a `Provider` here is a bug.
fn custom(t: &Tool) -> (&str, Option<&str>, &Value) {
    match t {
        Tool::Custom {
            name,
            description,
            input_schema,
            ..
        } => (name.as_str(), description.as_deref(), input_schema),
        _ => panic!("composed tools are Custom"),
    }
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

    let tools = compose(wt.path(), WORKER, &["bash".to_string()], &[]).unwrap();

    assert_eq!(tools.len(), 1);
    let (name, description, input_schema) = custom(&tools[0]);
    assert_eq!(name, "bash");
    // §3.3: the schema file is the `input_schema`, verbatim.
    assert_eq!(
        *input_schema,
        serde_json::from_str::<Value>(BASH_SCHEMA).unwrap()
    );
    // A schema present without its skill frontmatter composes with a
    // `None` description — the transient producer-ordering state §3.3
    // sanctions — rather than being dropped.
    assert_eq!(description, None);
}

#[test]
fn skill_frontmatter_populates_the_tool_description() {
    let wt = TempDir::new().unwrap();
    write_schema(wt.path(), "bash", BASH_SCHEMA);
    write_skill(
        wt.path(),
        "bash",
        "name: bash\ndescription: Run a shell command.\n",
    );

    let tools = compose(wt.path(), WORKER, &["bash".to_string()], &[]).unwrap();
    // §3.3 point 3: the tool entry's `description` is its skill's
    // frontmatter `description`.
    let (_, description, input_schema) = custom(&tools[0]);
    assert_eq!(description, Some("Run a shell command."));
    assert_eq!(
        *input_schema,
        serde_json::from_str::<Value>(BASH_SCHEMA).unwrap()
    );
}

#[test]
fn malformed_skill_frontmatter_is_a_hard_error() {
    let wt = TempDir::new().unwrap();
    write_schema(wt.path(), "bash", BASH_SCHEMA);
    // Present but not a valid frontmatter mapping (missing fields).
    write_skill(wt.path(), "bash", "not: a valid frontmatter\n");
    let err = compose(wt.path(), WORKER, &["bash".to_string()], &[]).unwrap_err();
    match err {
        Error::SkillFrontmatter { name, .. } => assert_eq!(name, "bash"),
        other => panic!("expected SkillFrontmatter, got {other:?}"),
    }
}

#[test]
fn unreadable_skill_frontmatter_surfaces_io_error() {
    let wt = TempDir::new().unwrap();
    write_schema(wt.path(), "bash", BASH_SCHEMA);
    // A directory where `bash.md` is expected: `read_to_string` fails
    // with a non-NotFound error, surfaced rather than treated as absent.
    fs::create_dir_all(wt.path().join(SKILLS_DESC_DIR).join("bash.md")).unwrap();
    let err = compose(wt.path(), WORKER, &["bash".to_string()], &[]).unwrap_err();
    match err {
        Error::SkillFrontmatterIo { name, .. } => assert_eq!(name, "bash"),
        other => panic!("expected SkillFrontmatterIo, got {other:?}"),
    }
}

#[test]
fn empty_declaration_yields_empty_tools() {
    let wt = TempDir::new().unwrap();
    write_schema(wt.path(), "bash", BASH_SCHEMA);
    assert!(compose(wt.path(), WORKER, &[], &[]).unwrap().is_empty());
}

#[test]
fn declared_tool_without_schema_is_dropped() {
    let wt = TempDir::new().unwrap();
    // No descriptions/tools dir at all — the intersection is empty.
    assert!(
        compose(wt.path(), WORKER, &["bash".to_string()], &[])
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
    let tools = compose(wt.path(), WORKER, &declared, &[]).unwrap();

    // `missing` has no schema and is dropped; the rest keep declared order.
    let names: Vec<&str> = tools.iter().map(|t| custom(t).0).collect();
    assert_eq!(names, vec!["bash", "read_file"]);
}

#[test]
fn present_but_malformed_schema_is_a_hard_error() {
    let wt = TempDir::new().unwrap();
    write_schema(wt.path(), "bash", "{ not json");
    let err = compose(wt.path(), WORKER, &["bash".to_string()], &[]).unwrap_err();
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
    let err = compose(wt.path(), WORKER, &["bash".to_string()], &[]).unwrap_err();
    match err {
        Error::ToolSchemaIo { name, .. } => assert_eq!(name, "bash"),
        other => panic!("expected ToolSchemaIo, got {other:?}"),
    }
}

/// An assistant message calling each of `names` — the assembled shape
/// whose `tool_use` blocks the closure reads (§2.3).
fn history_calling(names: &[&str]) -> Vec<Message> {
    vec![Message {
        role: brazen::Role::Assistant,
        content: names
            .iter()
            .enumerate()
            .map(|(i, name)| Content::ToolUse {
                id: format!("toolu_{i}"),
                name: (*name).to_string(),
                input: json!({}),
                signature: None,
            })
            .collect(),
    }]
}

#[test]
fn a_tool_the_history_names_is_declared_with_its_committed_schema() {
    // The compactor case (§2.7): the inherited transcript calls `bash`,
    // which no `tools:` list of the role declares — but its schema rides
    // the inherited `descriptions/**`, so the entry is the real one.
    let wt = TempDir::new().unwrap();
    write_schema(wt.path(), "bash", BASH_SCHEMA);
    write_skill(
        wt.path(),
        "bash",
        "name: bash\ndescription: Run a shell command.\n",
    );

    let tools = compose(wt.path(), WORKER, &[], &history_calling(&["bash"])).unwrap();

    assert_eq!(tools.len(), 1);
    let (name, description, input_schema) = custom(&tools[0]);
    assert_eq!(name, "bash");
    assert_eq!(description, Some("Run a shell command."));
    assert_eq!(
        *input_schema,
        serde_json::from_str::<Value>(BASH_SCHEMA).unwrap()
    );
}

#[test]
fn a_tool_the_history_names_with_no_schema_still_gets_a_declaration() {
    // A name the model invented: the exchange landed in the transcript
    // regardless, so the request must still account for it. Unlike an
    // elected name (dropped), it composes with a stand-in schema.
    let wt = TempDir::new().unwrap();
    let tools = compose(wt.path(), WORKER, &[], &history_calling(&["frobnicate"])).unwrap();

    let (name, description, input_schema) = custom(&tools[0]);
    assert_eq!(name, "frobnicate");
    assert_eq!(description, None);
    assert_eq!(*input_schema, json!({"type":"object"}));
}

#[test]
fn an_already_declared_tool_is_never_declared_twice() {
    let wt = TempDir::new().unwrap();
    write_schema(wt.path(), "bash", BASH_SCHEMA);
    // Named twice by the history, and already elected: still one entry.
    let declared = ["bash".to_string()];
    let tools = compose(
        wt.path(),
        WORKER,
        &declared,
        &history_calling(&["bash", "bash"]),
    )
    .unwrap();
    let names: Vec<&str> = tools.iter().map(|t| custom(t).0).collect();
    assert_eq!(names, vec!["bash"]);
}

#[test]
fn a_compactor_gets_its_injected_pair_and_never_a_duplicate_of_it() {
    // §2.7: the pair is injected for this role alone, and a compactor's
    // own later history naming `write_summary` must not re-declare it.
    let wt = TempDir::new().unwrap();
    let history = history_calling(&["write_summary", "bash"]);
    let tools = compose(wt.path(), "compactor", &[], &history).unwrap();
    let names: Vec<&str> = tools.iter().map(|t| custom(t).0).collect();
    assert_eq!(names, vec!["write_summary", "mark_for_deletion", "bash"]);
}

#[test]
fn a_history_with_no_tool_use_leaves_the_declaration_alone() {
    // The plain no-tools path: nothing to close over, nothing appended.
    let wt = TempDir::new().unwrap();
    write_schema(wt.path(), "bash", BASH_SCHEMA);
    let history = vec![Message {
        role: brazen::Role::User,
        content: vec![Content::Text("just talking".into())],
    }];
    let declared = ["bash".to_string()];
    let tools = compose(wt.path(), WORKER, &declared, &history).unwrap();
    let names: Vec<&str> = tools.iter().map(|t| custom(t).0).collect();
    assert_eq!(names, vec!["bash"]);
}

#[test]
fn a_malformed_schema_for_a_history_named_tool_is_a_hard_error() {
    let wt = TempDir::new().unwrap();
    write_schema(wt.path(), "bash", "{ not json");
    let err = compose(wt.path(), WORKER, &[], &history_calling(&["bash"])).unwrap_err();
    match err {
        Error::ToolSchemaJson { name, .. } => assert_eq!(name, "bash"),
        other => panic!("expected ToolSchemaJson, got {other:?}"),
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
    let tools = compose(wt.path(), WORKER, &["x".to_string()], &[]).unwrap();
    assert_eq!(
        *custom(&tools[0]).2,
        json!({"type":"object","properties":{"a":{"type":"number"}}})
    );
}
