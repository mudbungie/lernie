//! The working-directory mark at the spawn boundary (ARCH §3.3 *Working
//! directory*): the agent's worktree is the **default** cwd, and a `cd`
//! the agent made moves every later tool call. The mark itself is
//! covered against real git in `workspace::cwd`; here the git read is
//! stubbed so the executor's three arms are structural.

use super::super::{SpawnTool, ToolCall, ToolExecutor};
use super::fixtures::{FixedClock, HarnessRoot, StepDir, after_header, driver_target};
use crate::template::GitRunner;
use serde_json::json;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::atomic::AtomicBool;

/// [`GitRunner`] answering the mark read with a fixed path — the shape
/// `workspace::cwd::read` parses (`git cat-file blob <ref>`).
struct MarkedAt(String);
impl GitRunner for MarkedAt {
    fn run(&self, _dest: &Path, _args: &[&str]) -> io::Result<()> {
        unreachable!("the mark read only captures")
    }
    fn run_capture(&self, _dest: &Path, _args: &[&str]) -> io::Result<String> {
        Ok(self.0.clone())
    }
}

/// Run a `pwd` fixture tool and return the directory it reported.
fn where_the_tool_ran(git: Box<dyn GitRunner>, step: &StepDir) -> PathBuf {
    let root = HarnessRoot::new();
    root.install("whereami", "pwd");
    let clock = FixedClock::default();
    let exec = SpawnTool::new(root.path(), &clock, driver_target()).with_git(git);
    let outcome = exec
        .execute(
            ToolCall {
                id: "toolu_cwd",
                name: "whereami",
                input: &json!({}),
            },
            &step.path,
            &AtomicBool::new(false),
        )
        .unwrap();
    PathBuf::from(
        String::from_utf8(after_header(&outcome.content).to_vec())
            .unwrap()
            .trim(),
    )
}

#[test]
fn a_mark_naming_a_live_directory_is_where_the_tool_runs() {
    let step = StepDir::new();
    let elsewhere = tempfile::TempDir::new().unwrap();
    let ran_in = where_the_tool_ran(
        Box::new(MarkedAt(elsewhere.path().to_string_lossy().into_owned())),
        &step,
    );
    assert_eq!(ran_in, std::fs::canonicalize(elsewhere.path()).unwrap());
}

#[test]
fn a_mark_whose_directory_is_gone_falls_back_to_the_worktree() {
    // `cd` is itself a tool call, so declining here would strand the
    // agent somewhere it can never leave. The default answers instead.
    let step = StepDir::new();
    let ran_in = where_the_tool_ran(
        Box::new(MarkedAt("/no/such/place/at/all".to_string())),
        &step,
    );
    assert_eq!(ran_in, std::fs::canonicalize(&step.worktree).unwrap());
}

#[test]
fn an_unset_mark_leaves_the_worktree_as_the_cwd() {
    // The empty mark is the state of every agent that never called `cd`
    // — the general path with the fact absent, not a special case.
    let step = StepDir::new();
    let ran_in = where_the_tool_ran(Box::new(MarkedAt(String::new())), &step);
    assert_eq!(ran_in, std::fs::canonicalize(&step.worktree).unwrap());
}
