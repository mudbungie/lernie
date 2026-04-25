//! Built-in tools — the in-process implementations behind
//! `lernie tool <name>` (ARCH §3.3, §12 v0.3 toolset).
//!
//! Each tool is a pure function over [`Read`]/[`Write`] so unit tests
//! drive it without touching real stdio. The `lernie tool` subcommand
//! is a thin shim that locks the process's stdio handles and delegates
//! to [`run`]; the §3.3 stdio contract (stdin = `tool_use.input` JSON,
//! stdout = raw result bytes, exit code = is_error) is enforced here.
//!
//! v0.3 ships two built-ins: [`read_file`] and [`bash`]. Adding a new
//! one is a match arm in [`run`] plus a sibling module.

use std::io::{Read, Write};
use thiserror::Error;

pub mod read_file;

/// Reasons [`run`] can fail. Each in-process tool surfaces its own
/// error variant; an unknown tool name is the dispatcher-level case.
#[derive(Debug, Error)]
pub enum Error {
    /// The lernie binary was invoked as `lernie tool <name>` for a
    /// `<name>` that isn't a built-in. The harness only routes here
    /// after external resolution misses (§3.3), so this is "no tool
    /// of that name exists at all".
    #[error("unknown built-in tool: {0:?}")]
    Unknown(String),
    /// `read_file` failed; carries the inner reason for the operator's
    /// `eprintln!`. The §3.3 stdio contract concats stderr after
    /// stdout into `tool_result.content` when exit code is non-zero,
    /// so the message reaches the model verbatim.
    #[error(transparent)]
    ReadFile(#[from] read_file::Error),
}

/// Dispatch one in-process tool call. `name` is the tool name as the
/// model spelled it (and as the harness passed via `lernie tool
/// <name>`); `stdin` carries the `tool_use.input` JSON; `stdout`
/// receives the raw bytes the executor will surface as
/// `tool_result.content`.
pub fn run<R: Read, W: Write>(name: &str, stdin: &mut R, stdout: &mut W) -> Result<(), Error> {
    if name == "read_file" {
        return read_file::run(stdin, stdout).map_err(Error::ReadFile);
    }
    Err(Error::Unknown(name.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn unknown_tool_name_surfaces_unknown_variant() {
        let mut stdin = Cursor::new(Vec::<u8>::new());
        let mut stdout = Vec::new();
        let err = run("not_a_tool", &mut stdin, &mut stdout).unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains("not_a_tool"), "{msg}");
        assert!(msg.contains("unknown"), "{msg}");
    }

    #[test]
    fn read_file_routed_to_inner_module() {
        // A minimal-but-valid input that drives the inner module's
        // happy path. Exercising the dispatch arm for read_file.
        let tmp = tempfile::NamedTempFile::new().unwrap();
        std::fs::write(tmp.path(), b"hi").unwrap();
        let input = serde_json::json!({ "path": tmp.path() }).to_string();
        let mut stdin = Cursor::new(input.into_bytes());
        let mut stdout = Vec::new();
        run("read_file", &mut stdin, &mut stdout).unwrap();
        assert_eq!(stdout, b"hi");
    }

    #[test]
    fn read_file_error_is_carried_through_dispatcher() {
        // Bad JSON on stdin — read_file::Error::InvalidJson — should
        // surface through the From conversion as Error::ReadFile.
        let mut stdin = Cursor::new(b"not json".to_vec());
        let mut stdout = Vec::new();
        let err = run("read_file", &mut stdin, &mut stdout).unwrap_err();
        assert!(matches!(err, Error::ReadFile(_)), "{err}");
    }
}
