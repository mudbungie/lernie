//! Per-step streaming `complete` invocation (ARCH §4.4, §3.5).
//!
//! Drives the adapter with `stream: true` and tails its stdout
//! line-by-line. Each event line is appended to
//! `<conv-repo>/steps/<conv-id>/<NNN>/response.json` (JSONL of §4.4
//! events) **and** fed through the in-memory [`Assembler`] so the
//! step loop has the assistant blocks ready for the next request.
//!
//! End-of-stream is the writer closing the response.json fd: the
//! [`std::fs::File`] is held in this function's scope and dropped on
//! return, which is what surfaces `IN_CLOSE_WRITE` to filesystem
//! watchers (§3.5). The harness never reads `response.json` back at
//! runtime — the assembler's [`Completion`] is the only consumer of
//! the model's output (§2.3 Diagnostic-only contract).

use super::assembler::{Assembler, AssemblyError, Completion};
use crate::prompt::Error;
use crate::prompt::adapter::AdapterRunner;
use crate::provider::wire::StreamEvent;
use serde_json::Value;
use std::ffi::OsString;
use std::fs::File;
use std::io::Write;
use std::path::Path;

/// Drive one streaming model call. The adapter binary is invoked with
/// `complete`, the request bytes are piped to its stdin, and each
/// stdout line is teed to `response_path` (JSONL) and the in-memory
/// assembler. Returns the assembled completion on `message_stop`;
/// half-streams and in-band `error` events surface as harness errors.
pub(super) fn run_complete(
    adapter: &dyn AdapterRunner,
    binary: &OsString,
    endpoint_envs: &[(&str, &str)],
    request_bytes: &[u8],
    response_path: &Path,
) -> Result<Completion, Error> {
    if let Some(parent) = response_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let mut response_file = File::create(response_path)?;
    let mut assembler = Assembler::new();
    let mut feed_err: Option<Error> = None;

    adapter
        .run(
            binary,
            &["complete"],
            endpoint_envs,
            request_bytes,
            &mut |line| {
                response_file.write_all(line)?;
                response_file.write_all(b"\n")?;
                if feed_err.is_some() || assembler.is_terminal() {
                    return Ok(());
                }
                match serde_json::from_slice::<StreamEvent>(line) {
                    Ok(event) => assembler.feed(event),
                    Err(e) => feed_err = Some(Error::AdapterJson(e)),
                }
                Ok(())
            },
        )
        .map_err(Error::AdapterSpawn)?;

    // Drop closes the fd; that close is the §3.5 IN_CLOSE_WRITE
    // completion signal for any frontend tailing this file. Drop
    // before raising any error so the file is observable on disk.
    drop(response_file);

    if let Some(err) = feed_err {
        return Err(err);
    }
    assembler.into_completion().map_err(into_prompt_error)
}

/// Build the wire request body. Held here so the per-step body in
/// [`super`] stays narrow.
pub(super) fn build_request(
    model_id: &str,
    system: &str,
    messages: &[Value],
    max_tokens: u32,
) -> Value {
    serde_json::json!({
        "model": model_id,
        "max_tokens": max_tokens,
        "system": system,
        "messages": messages,
        "stream": true,
    })
}

fn into_prompt_error(e: AssemblyError) -> Error {
    match e {
        AssemblyError::Adapter {
            kind,
            message,
            http_status,
        } => Error::AdapterError {
            kind,
            message,
            http_status,
        },
        AssemblyError::HalfStream => Error::AdapterError {
            kind: "fatal".into(),
            message: "stream ended without message_stop".into(),
            http_status: None,
        },
        AssemblyError::MissingStopReason => Error::AdapterError {
            kind: "fatal".into(),
            message: "message_stop missing stop_reason".into(),
            http_status: None,
        },
        AssemblyError::ToolInputJson(e) => Error::AdapterJson(e),
    }
}
