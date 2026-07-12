//! Unit tests for transcript assembly (ARCH §2.3, §5).
//!
//! Assembly is a pure function of the read-state tree, so every test
//! builds a `messages/` directory by hand and asserts the composed wire
//! history — the same operation running, retry, and replay all invoke
//! (§2.3 *Crash and recovery*). The "replay" test is exactly this: a
//! `messages/` tree recorded to disk, assembled cold.

use super::*;
use brazen::{Content, Role};
use serde_json::json;
use tempfile::TempDir;

/// Write a raw transcript entry file under `<worktree>/messages/`.
fn write(worktree: &Path, name: &str, bytes: &[u8]) {
    let dir = worktree.join(MESSAGES_DIR);
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join(name), bytes).unwrap();
}

/// Write a `.json` entry from a canonical block list.
fn write_json(worktree: &Path, name: &str, blocks: &[Content]) {
    write(worktree, name, &serde_json::to_vec(blocks).unwrap());
}

#[test]
fn absent_messages_dir_assembles_no_messages() {
    let dir = TempDir::new().unwrap();
    assert!(assemble(dir.path()).unwrap().is_empty());
}

#[test]
fn empty_messages_dir_assembles_no_messages() {
    let dir = TempDir::new().unwrap();
    std::fs::create_dir_all(dir.path().join(MESSAGES_DIR)).unwrap();
    assert!(assemble(dir.path()).unwrap().is_empty());
}

#[test]
fn messages_path_that_is_a_file_surfaces_io_error() {
    // A file where `messages/` is expected → read_dir fails non-NotFound.
    let dir = TempDir::new().unwrap();
    std::fs::write(dir.path().join(MESSAGES_DIR), b"not a dir").unwrap();
    assert!(matches!(assemble(dir.path()).unwrap_err(), Error::Io(_)));
}

#[test]
fn user_message_composes_as_a_user_text_block_verbatim() {
    let dir = TempDir::new().unwrap();
    write(dir.path(), "001-user.md", b"list files");
    let msgs = assemble(dir.path()).unwrap();
    assert_eq!(msgs.len(), 1);
    assert_eq!(msgs[0].role, Role::User);
    assert_eq!(msgs[0].content, vec![Content::Text("list files".into())]);
}

#[test]
fn replay_from_a_recorded_tree_yields_the_alternating_wire_history() {
    // A recorded multi-step transcript assembled cold: user →
    // assistant(tool_use) → the tool result, exactly the §2.5 pairing.
    let dir = TempDir::new().unwrap();
    write(dir.path(), "001-user.md", b"go");
    write_json(
        dir.path(),
        "002-claude-fable-5.json",
        &[Content::ToolUse {
            id: "toolu_1".into(),
            name: "bash".into(),
            input: json!({"cmd": "ls"}),
        }],
    );
    write_json(
        dir.path(),
        "003-tool.json",
        &[Content::ToolResult {
            tool_use_id: "toolu_1".into(),
            content: vec![Content::Text("out".into())],
            is_error: false,
        }],
    );

    let msgs = assemble(dir.path()).unwrap();
    assert_eq!(msgs.len(), 3);
    assert_eq!(msgs[0].role, Role::User);
    assert_eq!(msgs[1].role, Role::Assistant);
    assert!(matches!(msgs[1].content[0], Content::ToolUse { .. }));
    assert_eq!(msgs[2].role, Role::User);
    assert!(matches!(msgs[2].content[0], Content::ToolResult { .. }));
}

#[test]
fn consecutive_same_side_entries_group_into_one_message() {
    // Two tool results after one assistant step fold into a single
    // user message carrying both `tool_result` blocks (§2.3 grouping).
    let dir = TempDir::new().unwrap();
    write(dir.path(), "001-user.md", b"do two");
    write_json(
        dir.path(),
        "002-claude-fable-5.json",
        &[
            Content::ToolUse {
                id: "a".into(),
                name: "bash".into(),
                input: json!({}),
            },
            Content::ToolUse {
                id: "b".into(),
                name: "read_file".into(),
                input: json!({}),
            },
        ],
    );
    write_json(dir.path(), "003-tool.json", &[tool_result("a")]);
    write_json(dir.path(), "004-tool.json", &[tool_result("b")]);

    let msgs = assemble(dir.path()).unwrap();
    assert_eq!(msgs.len(), 3);
    // The two tool entries grouped into one user message, in seq order.
    assert_eq!(msgs[2].role, Role::User);
    assert_eq!(msgs[2].content.len(), 2);
    assert!(
        matches!(&msgs[2].content[0], Content::ToolResult { tool_use_id, .. } if tool_use_id == "a")
    );
    assert!(
        matches!(&msgs[2].content[1], Content::ToolResult { tool_use_id, .. } if tool_use_id == "b")
    );
}

#[test]
fn entries_sort_by_numeric_prefix_and_ignore_non_conforming_names() {
    // Out-of-order on disk, plus a stray non-conforming name: assembly
    // sorts by the NNN prefix and drops the stray.
    let dir = TempDir::new().unwrap();
    write_json(
        dir.path(),
        "002-claude-fable-5.json",
        &[Content::Text("second".into())],
    );
    write(dir.path(), "001-user.md", b"first");
    write(dir.path(), "notes.txt", b"ignored");

    let msgs = assemble(dir.path()).unwrap();
    assert_eq!(msgs.len(), 2);
    assert_eq!(msgs[0].content, vec![Content::Text("first".into())]);
    assert_eq!(msgs[1].role, Role::Assistant);
    assert_eq!(msgs[1].content, vec![Content::Text("second".into())]);
}

fn tool_result(id: &str) -> Content {
    Content::ToolResult {
        tool_use_id: id.into(),
        content: vec![Content::Text("out".into())],
        is_error: false,
    }
}
