//! Unit tests for the dispatch-time unsettled-tool-step prune (ARCH
//! §2.3, §2.5). The fork-level proof — a real child forked mid-tool-step
//! whose first assembled history the wire accepts — is [`fork`].
//!
//! Lives in a sibling file rather than an inline `mod tests` so the
//! production module stays under the 300-line repo cap.

mod fork;

use super::*;
use serde_json::json;
use std::cell::RefCell;
use std::io;
use std::path::PathBuf;

#[derive(Default)]
struct StubGit {
    runs: RefCell<Vec<(PathBuf, Vec<String>)>>,
    fail: bool,
}

impl GitRunner for StubGit {
    fn run(&self, dest: &Path, args: &[&str]) -> io::Result<()> {
        self.runs.borrow_mut().push((
            dest.to_path_buf(),
            args.iter().map(|s| (*s).to_owned()).collect(),
        ));
        if self.fail {
            Err(io::Error::other("stub git fail"))
        } else {
            Ok(())
        }
    }
    fn run_capture(&self, _dest: &Path, _args: &[&str]) -> io::Result<String> {
        unreachable!("the prune never issues capturing git ops")
    }
}

/// A worktree carrying the given transcript entries verbatim.
fn tree(entries: &[(&str, String)]) -> tempfile::TempDir {
    let dir = tempfile::TempDir::new().unwrap();
    std::fs::create_dir_all(dir.path().join(MESSAGES_DIR)).unwrap();
    for (name, body) in entries {
        std::fs::write(dir.path().join(MESSAGES_DIR).join(name), body).unwrap();
    }
    dir
}

/// A canonical entry body (§2.3): a JSON array of `Content` blocks.
fn entry(blocks: &[Content]) -> String {
    serde_json::to_string(blocks).unwrap()
}

fn tool_use(id: &str) -> Content {
    Content::ToolUse {
        id: id.into(),
        name: "dispatch".into(),
        input: json!({"role": "worker", "goal": "go"}),
        signature: None,
    }
}

fn tool_result(id: &str) -> Content {
    Content::ToolResult {
        tool_use_id: id.into(),
        content: vec![Content::Text("p1-child".into())],
        is_error: false,
    }
}

#[test]
fn a_tree_with_no_transcript_issues_no_git_op() {
    // Every fresh root: forked off a config commit, nothing inherited.
    let wt = tempfile::TempDir::new().unwrap();
    let git = StubGit::default();
    prune_unsettled(wt.path(), &git).unwrap();
    assert!(git.runs.borrow().is_empty());
}

#[test]
fn a_settled_tail_issues_no_git_op() {
    // The compactor's and verifier's fork points (§2.7, §6): every
    // `tool_use` already answered, so there is nothing to make honest.
    let wt = tree(&[
        ("001-user.md", "do a thing".into()),
        ("002-claude-sonnet-5.json", entry(&[tool_use("toolu_a")])),
        ("003-tool.json", entry(&[tool_result("toolu_a")])),
    ]);
    let git = StubGit::default();
    prune_unsettled(wt.path(), &git).unwrap();
    assert!(git.runs.borrow().is_empty());
}

#[test]
fn a_transcript_of_delivered_messages_alone_issues_no_git_op() {
    // No model output means no `tool_use` can be outstanding — the
    // general path with empty inputs, not a bootstrap special case.
    let wt = tree(&[("001-user.md", "hi".into())]);
    let git = StubGit::default();
    prune_unsettled(wt.path(), &git).unwrap();
    assert!(git.runs.borrow().is_empty());
}

#[test]
fn a_final_response_tail_issues_no_git_op() {
    let wt = tree(&[
        ("001-user.md", "hi".into()),
        (
            "002-claude-sonnet-5.json",
            entry(&[Content::Text("final".into())]),
        ),
    ]);
    let git = StubGit::default();
    prune_unsettled(wt.path(), &git).unwrap();
    assert!(git.runs.borrow().is_empty());
}

#[test]
fn the_unsettled_step_and_its_partial_results_are_staged_for_removal() {
    // The bl-4231 shape at its widest: a step that emitted two tool
    // calls, the first settled, the second (the dispatch) still running
    // when the child forked. The partial result orphans the moment its
    // `tool_use` leaves, so it goes with the entry that emitted it.
    let wt = tree(&[
        ("001-user.md", "do a thing".into()),
        (
            "002-gpt-5.4.json",
            entry(&[
                Content::Text("dispatching".into()),
                tool_use("call_a"),
                tool_use("call_b"),
            ]),
        ),
        ("003-tool.json", entry(&[tool_result("call_a")])),
    ]);
    let git = StubGit::default();
    prune_unsettled(wt.path(), &git).unwrap();

    let runs = git.runs.borrow();
    assert_eq!(runs.len(), 1);
    assert_eq!(runs[0].0, wt.path());
    assert_eq!(
        runs[0].1,
        vec![
            "rm",
            "-q",
            "--",
            "messages/002-gpt-5.4.json",
            "messages/003-tool.json",
        ],
        "the delivered message ahead of the step survives; the step does not"
    );
}

#[test]
fn the_cut_follows_the_filename_counter_not_the_readdir_order() {
    // Order lives in the name (§2.3): a two-digit-wide neighbour and a
    // non-conforming file are both handled by that one rule — the latter
    // contributes no entry at all.
    let wt = tree(&[
        ("009-tool.json", entry(&[tool_result("call_old")])),
        ("008-claude-sonnet-5.json", entry(&[tool_use("call_old")])),
        ("010-gpt-5.4.json", entry(&[tool_use("call_new")])),
        ("README", "not an entry".into()),
    ]);
    let git = StubGit::default();
    prune_unsettled(wt.path(), &git).unwrap();

    let runs = git.runs.borrow();
    assert_eq!(
        runs[0].1,
        vec!["rm", "-q", "--", "messages/010-gpt-5.4.json"],
        "only the trailing unsettled step is cut; the settled pair below it stays"
    );
}

#[test]
fn an_unreadable_transcript_dir_surfaces_rather_than_pruning_blind() {
    // A regular file where `messages/` must be: `read_dir` fails with
    // something other than NotFound, and a prune that cannot enumerate
    // must not silently decide the tail is settled.
    let wt = tempfile::TempDir::new().unwrap();
    std::fs::write(wt.path().join(MESSAGES_DIR), b"not a directory").unwrap();
    let git = StubGit::default();
    let err = prune_unsettled(wt.path(), &git).unwrap_err();
    assert!(matches!(err, Error::Io(_)), "got {err:?}");
}

#[test]
fn an_unreadable_entry_surfaces_rather_than_pruning_blind() {
    let wt = tree(&[("001-claude-sonnet-5.json", entry(&[tool_use("call_a")]))]);
    std::fs::remove_file(wt.path().join("messages/001-claude-sonnet-5.json")).unwrap();
    std::fs::create_dir(wt.path().join("messages/001-claude-sonnet-5.json")).unwrap();
    let git = StubGit::default();
    let err = prune_unsettled(wt.path(), &git).unwrap_err();
    assert!(matches!(err, Error::Io(_)), "got {err:?}");
}

#[test]
fn a_failing_git_rm_surfaces_as_a_named_git_error() {
    let wt = tree(&[("001-claude-sonnet-5.json", entry(&[tool_use("call_a")]))]);
    let git = StubGit {
        fail: true,
        ..StubGit::default()
    };
    let err = prune_unsettled(wt.path(), &git).unwrap_err();
    assert!(
        matches!(&err, Error::Git { op, .. } if *op == "rm unsettled tool step"),
        "got {err:?}"
    );
}
