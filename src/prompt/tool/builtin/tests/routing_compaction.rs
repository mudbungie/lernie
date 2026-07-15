//! Routing tests for the compactor built-in tools (ARCH §2.7): each name
//! routes through [`super::super::run_with`] into the compaction module,
//! and its errors surface via `#[from]` into [`Error::Compaction`].

use super::super::{Error, dispatch, run_with};
use super::{StubSender, StubSpawner, stub_env};
use std::io::Cursor;

#[test]
fn write_summary_routed_to_inner_module() {
    // A real worktree under the stub env's repo: the summary write lands
    // and routes back a `written` status through the dispatcher.
    let repo = tempfile::TempDir::new().unwrap();
    std::fs::create_dir_all(crate::workspace::agent_worktree(repo.path(), "a1")).unwrap();
    let env = stub_env(repo.path(), "a1");
    let input = serde_json::json!({"content":"digest\n"}).to_string();
    let mut stdin = Cursor::new(input.into_bytes());
    let (mut stdout, mut stderr) = (Vec::new(), Vec::new());
    let code = run_with(
        "write_summary",
        &mut stdin,
        &mut stdout,
        &mut stderr,
        &env,
        &StubSpawner,
        &StubSender,
    )
    .unwrap();
    assert_eq!(code, 0);
    let v: serde_json::Value = serde_json::from_slice(&stdout).unwrap();
    assert_eq!(v["status"], "written");
}

#[test]
fn mark_for_deletion_error_is_carried_through_dispatcher() {
    // No env → compaction::Error::MissingEnv via #[from] into
    // Error::Compaction, routed through the `mark_for_deletion` arm.
    struct EmptyEnv;
    impl dispatch::EnvLookup for EmptyEnv {
        fn get(&self, _key: &str) -> Option<std::ffi::OsString> {
            None
        }
    }
    let input = serde_json::json!({"path":"messages/001-user.md"}).to_string();
    let mut stdin = Cursor::new(input.into_bytes());
    let (mut stdout, mut stderr) = (Vec::new(), Vec::new());
    let err = run_with(
        "mark_for_deletion",
        &mut stdin,
        &mut stdout,
        &mut stderr,
        &EmptyEnv,
        &StubSpawner,
        &StubSender,
    )
    .unwrap_err();
    assert!(matches!(err, Error::Compaction(_)), "{err}");
}
