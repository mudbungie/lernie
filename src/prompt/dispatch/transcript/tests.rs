//! Unit tests for transcript-entry commit (ARCH §2.3, §3.3).
//!
//! `next_seq` reads a real `messages/` directory; the commit helpers do
//! real filesystem moves and route their `git` verbs through a recording
//! stub, so the on-disk entry and the exact `add`/`commit` argv are both
//! observable without a live repo.

use super::*;
use brazen::Content;
use serde_json::Value;
use std::cell::RefCell;
use std::io;
use std::path::PathBuf;
use tempfile::TempDir;

/// Recording [`GitRunner`] — the commit helpers' only side channel.
#[derive(Default)]
struct RecordGit {
    runs: RefCell<Vec<(PathBuf, Vec<String>)>>,
}
impl GitRunner for RecordGit {
    fn run(&self, dest: &Path, args: &[&str]) -> io::Result<()> {
        self.runs.borrow_mut().push((
            dest.to_path_buf(),
            args.iter().map(|s| (*s).to_owned()).collect(),
        ));
        Ok(())
    }
    fn run_capture(&self, dest: &Path, args: &[&str]) -> io::Result<String> {
        self.run(dest, args)?;
        Ok(String::new())
    }
}

fn write_entry(worktree: &Path, name: &str) {
    let dir = worktree.join(MESSAGES_DIR);
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join(name), b"[]").unwrap();
}

#[test]
fn next_seq_is_one_when_messages_dir_is_absent() {
    let dir = TempDir::new().unwrap();
    assert_eq!(next_seq(dir.path()).unwrap(), 1);
}

#[test]
fn next_seq_is_one_when_messages_dir_is_empty() {
    let dir = TempDir::new().unwrap();
    std::fs::create_dir_all(dir.path().join(MESSAGES_DIR)).unwrap();
    assert_eq!(next_seq(dir.path()).unwrap(), 1);
}

#[test]
fn next_seq_is_max_prefix_plus_one_ignoring_unparsable_names() {
    let dir = TempDir::new().unwrap();
    write_entry(dir.path(), "001-assistant.json");
    write_entry(dir.path(), "004-tool.json");
    write_entry(dir.path(), "002-assistant.json");
    // A stray non-conforming name contributes no counter.
    write_entry(dir.path(), "notes.txt");
    assert_eq!(next_seq(dir.path()).unwrap(), 5);
}

#[test]
fn commit_assistant_renames_staging_and_commits_the_entry() {
    let dir = TempDir::new().unwrap();
    let worktree = dir.path();
    // A sealed staging file waiting to be renamed in.
    let staging = worktree.join("assistant.staging.json");
    std::fs::write(&staging, br#"[{"type":"text","text":"hi"}]"#).unwrap();

    let git = RecordGit::default();
    commit_assistant(worktree, "conv-1", &staging, &git).unwrap();

    // Staging moved to messages/001-assistant.json, verbatim.
    assert!(!staging.exists(), "staging renamed away");
    let entry = worktree.join("messages/001-assistant.json");
    let blocks: Vec<Content> = serde_json::from_slice(&std::fs::read(&entry).unwrap()).unwrap();
    assert_eq!(blocks, vec![Content::Text("hi".into())]);

    let runs = git.runs.borrow();
    assert_eq!(runs.len(), 2);
    assert_eq!(runs[0].0, worktree);
    assert_eq!(runs[0].1, vec!["add", "messages/001-assistant.json"]);
    assert_eq!(runs[1].1[0], "commit");
    assert!(runs[1].1[2].contains("transcript 001: assistant"));
    assert!(runs[1].1[2].contains("[conv-1]"));
}

#[test]
fn commit_tool_writes_a_single_block_array_and_commits_at_the_next_seq() {
    let dir = TempDir::new().unwrap();
    let worktree = dir.path();
    // Prior assistant entry occupies 001, so the tool lands at 002.
    write_entry(worktree, "001-assistant.json");

    let tool_result = Content::ToolResult {
        tool_use_id: "toolu_9".into(),
        content: vec![Content::Text("out".into())],
        is_error: false,
    };
    let git = RecordGit::default();
    commit_tool(worktree, "conv-2", &tool_result, &git).unwrap();

    let entry = worktree.join("messages/002-tool.json");
    let raw: Value = serde_json::from_slice(&std::fs::read(&entry).unwrap()).unwrap();
    // A one-element array of the canonical tool_result block.
    assert_eq!(raw.as_array().unwrap().len(), 1);
    assert_eq!(raw[0]["type"], "tool_result");
    assert_eq!(raw[0]["tool_use_id"], "toolu_9");
    let blocks: Vec<Content> = serde_json::from_value(raw).unwrap();
    assert_eq!(blocks, vec![tool_result]);

    let runs = git.runs.borrow();
    assert_eq!(runs[0].1, vec!["add", "messages/002-tool.json"]);
    assert!(runs[1].1[2].contains("transcript 002: tool [conv-2]"));
}

#[test]
fn entry_rel_zero_pads_to_three_digits() {
    assert_eq!(entry_rel(7, "assistant"), "messages/007-assistant.json");
    assert_eq!(entry_rel(42, "tool"), "messages/042-tool.json");
}

#[test]
fn commit_add_failure_surfaces_as_a_git_error() {
    // A git that fails every run: the `add` verb surfaces `Error::Git`.
    #[derive(Default)]
    struct FailGit;
    impl GitRunner for FailGit {
        fn run(&self, _: &Path, _: &[&str]) -> io::Result<()> {
            Err(io::Error::other("boom"))
        }
        fn run_capture(&self, _: &Path, _: &[&str]) -> io::Result<String> {
            Err(io::Error::other("boom"))
        }
    }
    let dir = TempDir::new().unwrap();
    let staging = dir.path().join("assistant.staging.json");
    std::fs::write(&staging, b"[]").unwrap();
    let err = commit_assistant(dir.path(), "c", &staging, &FailGit).unwrap_err();
    assert!(
        matches!(
            err,
            Error::Git {
                op: "transcript add",
                ..
            }
        ),
        "got {err:?}"
    );
}

#[test]
fn commit_verb_failure_surfaces_as_a_git_error() {
    // add succeeds (run 0), commit fails (run 1): the `commit` verb
    // surfaces `Error::Git { op: "transcript commit" }`.
    struct FailCommit {
        n: RefCell<usize>,
    }
    impl GitRunner for FailCommit {
        fn run(&self, _: &Path, _: &[&str]) -> io::Result<()> {
            let mut n = self.n.borrow_mut();
            *n += 1;
            if *n >= 2 {
                Err(io::Error::other("boom"))
            } else {
                Ok(())
            }
        }
        fn run_capture(&self, _: &Path, _: &[&str]) -> io::Result<String> {
            Ok(String::new())
        }
    }
    let dir = TempDir::new().unwrap();
    let staging = dir.path().join("assistant.staging.json");
    std::fs::write(&staging, b"[]").unwrap();
    let git = FailCommit { n: RefCell::new(0) };
    let err = commit_assistant(dir.path(), "c", &staging, &git).unwrap_err();
    assert!(
        matches!(
            err,
            Error::Git {
                op: "transcript commit",
                ..
            }
        ),
        "got {err:?}"
    );
}

#[test]
fn next_seq_surfaces_a_non_not_found_io_error() {
    // A file where `messages/` is expected: read_dir fails with a
    // non-NotFound error, which surfaces rather than defaulting to 1.
    let dir = TempDir::new().unwrap();
    std::fs::write(dir.path().join(MESSAGES_DIR), b"not a dir").unwrap();
    let err = next_seq(dir.path()).unwrap_err();
    assert!(matches!(err, Error::Io(_)), "got {err:?}");
}
