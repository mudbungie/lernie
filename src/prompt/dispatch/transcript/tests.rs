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
    write_entry(dir.path(), "001-claude-fable-5.json");
    write_entry(dir.path(), "004-tool.json");
    write_entry(dir.path(), "002-claude-fable-5.json");
    // A stray non-conforming name contributes no counter.
    write_entry(dir.path(), "notes.txt");
    assert_eq!(next_seq(dir.path()).unwrap(), 5);
}

#[test]
fn deliver_message_renames_the_inbox_file_in_and_commits_it() {
    let dir = TempDir::new().unwrap();
    let worktree = dir.path();
    // A model-output entry already occupies 001, so the delivery lands at
    // 002 — the counter is shared across origins (§2.3).
    write_entry(worktree, "001-claude-fable-5.json");
    // A deposited inbox file with frontmatter + body, elsewhere on disk.
    let src = dir.path().join("bob-005.md");
    std::fs::write(&src, "---\nfrom: bob\n---\nhello").unwrap();

    let git = RecordGit::default();
    deliver_message(worktree, "conv-3", "bob", &src, &git).unwrap();

    // The file *moved* (rename, not copy): gone from its origin, present
    // in the transcript with frontmatter and body untouched (§2.11).
    assert!(!src.exists(), "inbox file renamed away");
    let entry = worktree.join("messages/002-bob.md");
    assert_eq!(
        std::fs::read_to_string(&entry).unwrap(),
        "---\nfrom: bob\n---\nhello"
    );

    let runs = git.runs.borrow();
    assert_eq!(runs[0].1, vec!["add", "messages/002-bob.md"]);
    assert!(runs[1].1[2].contains("transcript 002: bob"));
    assert!(runs[1].1[2].contains("[conv-3]"));
}

#[test]
fn commit_assistant_renames_staging_and_commits_at_the_model_id_origin() {
    let dir = TempDir::new().unwrap();
    let worktree = dir.path();
    // A sealed staging file waiting to be renamed in.
    let staging = worktree.join("staging.json");
    std::fs::write(&staging, br#"[{"type":"text","text":"hi"}]"#).unwrap();

    let git = RecordGit::default();
    // The origin token is the authoring model id (§2.3), not `assistant`.
    commit_assistant(worktree, "conv-1", "claude-fable-5", &staging, &git).unwrap();

    // Staging moved to messages/001-claude-fable-5.json, verbatim.
    assert!(!staging.exists(), "staging renamed away");
    let entry = worktree.join("messages/001-claude-fable-5.json");
    let blocks: Vec<Content> = serde_json::from_slice(&std::fs::read(&entry).unwrap()).unwrap();
    assert_eq!(blocks, vec![Content::Text("hi".into())]);

    let runs = git.runs.borrow();
    assert_eq!(runs.len(), 2);
    assert_eq!(runs[0].0, worktree);
    assert_eq!(runs[0].1, vec!["add", "messages/001-claude-fable-5.json"]);
    assert_eq!(runs[1].1[0], "commit");
    assert!(runs[1].1[2].contains("transcript 001: claude-fable-5"));
    assert!(runs[1].1[2].contains("[conv-1]"));
}

#[test]
fn commit_assistant_declines_a_model_id_colliding_with_the_reserved_tool_token() {
    // A model id equal to the one reserved `.json` origin token is
    // declined, not munged (§2.3 — decline illegal operations): were it
    // written, the assembler would mis-compose the model output as a
    // `tool_result`.
    let dir = TempDir::new().unwrap();
    let staging = dir.path().join("staging.json");
    std::fs::write(&staging, b"[]").unwrap();
    let git = RecordGit::default();
    let err = commit_assistant(dir.path(), "c", "tool", &staging, &git).unwrap_err();
    assert!(
        matches!(&err, Error::ReservedModelId(m) if m == "tool"),
        "got {err:?}"
    );
    // Declined before any git op and before the staging file moved.
    assert!(git.runs.borrow().is_empty());
    assert!(staging.exists(), "staging untouched on decline");
}

#[test]
fn commit_tool_writes_a_single_block_array_and_commits_at_the_next_seq() {
    let dir = TempDir::new().unwrap();
    let worktree = dir.path();
    // Prior model-output entry occupies 001, so the tool lands at 002.
    write_entry(worktree, "001-claude-fable-5.json");

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
    assert_eq!(
        entry_rel(7, "claude-fable-5"),
        "messages/007-claude-fable-5.json"
    );
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
    let staging = dir.path().join("staging.json");
    std::fs::write(&staging, b"[]").unwrap();
    let err = commit_assistant(dir.path(), "c", "claude-fable-5", &staging, &FailGit).unwrap_err();
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
    let staging = dir.path().join("staging.json");
    std::fs::write(&staging, b"[]").unwrap();
    let git = FailCommit { n: RefCell::new(0) };
    let err = commit_assistant(dir.path(), "c", "claude-fable-5", &staging, &git).unwrap_err();
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
