//! Routing tests for the `bash` built-in (ARCH §3.3): the name routes
//! through [`super::super::run`] into the bash module, and its errors
//! surface via `#[from]` into [`Error::Bash`].

use super::super::Error;
use super::route;
use std::io::Cursor;

#[test]
fn bash_routed_to_inner_module() {
    // Drives the dispatch arm for bash through a trivial command.
    let input = serde_json::json!({ "command": "printf hi" }).to_string();
    let mut stdin = Cursor::new(input.into_bytes());
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    let code = route("bash", &mut stdin, &mut stdout, &mut stderr).unwrap();
    assert_eq!(code, 0);
    assert_eq!(stdout, b"hi");
}

#[test]
fn bash_error_is_carried_through_dispatcher() {
    // Bad JSON on stdin — bash::Error::InvalidJson — should surface
    // through the From conversion as Error::Bash.
    let mut stdin = Cursor::new(b"not json".to_vec());
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    let err = route("bash", &mut stdin, &mut stdout, &mut stderr).unwrap_err();
    assert!(matches!(err, Error::Bash(_)), "{err}");
}
