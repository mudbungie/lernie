//! The `cd` routing arm (ARCH §3.3 *Working directory*): the dispatcher
//! hands the call to the inner module and carries its decline back
//! through [`Error::Cd`].

use super::super::{Error, run_with};
use super::{StubSender, StubSpawner, stub_env};
use std::io::Cursor;

#[test]
fn cd_routed_to_inner_module() {
    let (_h, ws) = crate::workspace::fixture::workspace();
    let worktree = crate::workspace::fixture::spawn_root(&ws, "p1");
    let input = serde_json::json!({ "path": worktree }).to_string();
    let mut stdin = Cursor::new(input.into_bytes());
    let (mut stdout, mut stderr) = (Vec::new(), Vec::new());
    let code = run_with(
        "cd",
        &mut stdin,
        &mut stdout,
        &mut stderr,
        &stub_env(&ws, "p1"),
        &StubSpawner,
        &StubSender,
    )
    .unwrap();
    assert_eq!(code, 0);
    let payload: serde_json::Value = serde_json::from_slice(&stdout).unwrap();
    let canonical = std::fs::canonicalize(&worktree).unwrap();
    assert_eq!(payload["cwd"], canonical.to_string_lossy().as_ref());
}

#[test]
fn cd_error_is_carried_through_dispatcher() {
    // A path that names nothing — cd::Error::Resolve via `#[from]`.
    let repo = tempfile::TempDir::new().unwrap();
    let input = serde_json::json!({ "path": "/no/such/place/at/all" }).to_string();
    let mut stdin = Cursor::new(input.into_bytes());
    let (mut stdout, mut stderr) = (Vec::new(), Vec::new());
    let err = run_with(
        "cd",
        &mut stdin,
        &mut stdout,
        &mut stderr,
        &stub_env(repo.path(), "p1"),
        &StubSpawner,
        &StubSender,
    )
    .unwrap_err();
    assert!(matches!(err, Error::Cd(_)), "{err}");
}
