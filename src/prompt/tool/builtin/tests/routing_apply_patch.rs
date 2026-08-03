//! The `apply_patch` routing arm: the dispatcher reaches the inner
//! module, and inner failures surface as [`Error::ApplyPatch`].

use super::super::Error;
use super::route;
use std::io::Cursor;

#[test]
fn apply_patch_routed_to_inner_module() {
    let tmp = tempfile::TempDir::new().unwrap();
    let path = tmp.path().join("new.txt");
    let patch = format!(
        "*** Begin Patch\n*** Add File: {}\n+hi\n*** End Patch",
        path.display()
    );
    let input = serde_json::json!({ "input": patch }).to_string();
    let mut stdin = Cursor::new(input.into_bytes());
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    let code = route("apply_patch", &mut stdin, &mut stdout, &mut stderr).unwrap();
    assert_eq!(code, 0);
    assert_eq!(std::fs::read_to_string(path).unwrap(), "hi\n");
    assert!(stdout.starts_with(b"{\"status\":\"applied\""));
}

#[test]
fn apply_patch_error_is_carried_through_dispatcher() {
    let mut stdin = Cursor::new(b"not json".to_vec());
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    let err = route("apply_patch", &mut stdin, &mut stdout, &mut stderr).unwrap_err();
    assert!(matches!(err, Error::ApplyPatch(_)), "{err}");
}
