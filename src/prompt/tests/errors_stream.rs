//! v0.3.1 P3 stream-shape error paths for [`crate::prompt::run`].
//!
//! Pins the streaming-wire decisions in `docs/ARCHITECTURE.md` §4.4
//! "Response shape (streaming)" / §3.5 (writer-closes-fd completion):
//! malformed JSONL surfaces as [`Error::AdapterJson`]; assembler
//! contract violations (half-stream, missing stop_reason, malformed
//! tool input) surface as either an [`Error::AdapterError`] synthesized
//! by the harness or `AdapterJson` for the parse failure.
//!
//! Lives in its own file so the original [`super::errors`] stays under
//! the repo's per-file line cap; the cases here are spec-anchored to
//! the §4.4 streaming surface specifically.

use super::fixtures::*;
use crate::prompt::Error;

#[test]
fn run_surfaces_malformed_complete_jsonl_line() {
    // A complete-side line that isn't valid JSON surfaces as
    // AdapterJson — the streaming wire is JSONL, one event per line,
    // so a parse failure on any line is a contract violation.
    let repo = scaffold_repo(VALID_PER_REPO_PROVIDERS_YAML, Some("body"));
    let adapter = StubAdapter::happy(b"{ not json\n");
    let err = run_with_stubs(repo.path(), "hi", &adapter, &StubGit::ok()).unwrap_err();
    assert!(matches!(err, Error::AdapterJson(_)));
}

#[test]
fn run_surfaces_event_missing_required_type_tag() {
    // A JSONL line that is valid JSON but does not tag itself as a
    // known §4.4 event type fails StreamEvent deserialization and
    // surfaces as AdapterJson.
    let repo = scaffold_repo(VALID_PER_REPO_PROVIDERS_YAML, Some("body"));
    let adapter = StubAdapter::happy(b"{\"unexpected\":\"shape\"}\n");
    let err = run_with_stubs(repo.path(), "hi", &adapter, &StubGit::ok()).unwrap_err();
    assert!(matches!(err, Error::AdapterJson(_)));
}

#[test]
fn run_surfaces_half_stream_with_synthetic_fatal_message() {
    // Adapter exits without writing any terminal event — the
    // assembler's HalfStream surfaces as a synthesized fatal
    // AdapterError so a half-stream is never silently dropped.
    let repo = scaffold_repo(VALID_PER_REPO_PROVIDERS_YAML, Some("body"));
    let adapter = StubAdapter::happy(b"");
    let err = run_with_stubs(repo.path(), "hi", &adapter, &StubGit::ok()).unwrap_err();
    match err {
        Error::AdapterError {
            kind,
            message,
            http_status,
        } => {
            assert_eq!(kind, "fatal");
            assert!(message.contains("stream ended without message_stop"));
            assert_eq!(http_status, None);
        }
        other => panic!("expected AdapterError, got {other:?}"),
    }
}

#[test]
fn run_surfaces_message_stop_missing_stop_reason() {
    // §4.4 requires a stop_reason on message_stop (or earlier); a
    // stop event with none surfaces as a synthetic fatal.
    let repo = scaffold_repo(VALID_PER_REPO_PROVIDERS_YAML, Some("body"));
    let body = b"{\"type\":\"message_stop\",\"usage\":{\"input_tokens\":1,\"output_tokens\":1},\"api_calls\":1}\n";
    let adapter = StubAdapter::happy(body);
    let err = run_with_stubs(repo.path(), "hi", &adapter, &StubGit::ok()).unwrap_err();
    match err {
        Error::AdapterError { kind, message, .. } => {
            assert_eq!(kind, "fatal");
            assert!(message.contains("missing stop_reason"));
        }
        other => panic!("expected AdapterError, got {other:?}"),
    }
}

#[test]
fn run_surfaces_invalid_tool_input_partial_json() {
    // A tool_use stream whose accumulated `partial_json` is not valid
    // JSON surfaces as AdapterJson — the harness can't construct the
    // next step's tool_use input from a malformed payload.
    let repo = scaffold_repo(VALID_PER_REPO_PROVIDERS_YAML, Some("body"));
    let body = concat!(
        r#"{"type":"content_block_start","index":0,"content_block":{"type":"tool_use","id":"t1","name":"bash","input":{}}}"#,
        "\n",
        r#"{"type":"tool_use_delta","index":0,"partial_json":"{ not json"}"#,
        "\n",
        r#"{"type":"message_stop","stop_reason":"tool_use","usage":{"input_tokens":1,"output_tokens":1},"api_calls":1}"#,
        "\n",
    );
    let adapter = StubAdapter::happy(body.as_bytes());
    let err = run_with_stubs(repo.path(), "hi", &adapter, &StubGit::ok()).unwrap_err();
    assert!(matches!(err, Error::AdapterJson(_)));
}

#[test]
fn run_short_circuits_subsequent_lines_after_first_parse_error() {
    // After the first malformed JSONL line, later lines still write
    // through to response.json (faithful diagnostic record) but the
    // assembler is no longer fed — the first parse error is the one
    // surfaced.
    let repo = scaffold_repo(VALID_PER_REPO_PROVIDERS_YAML, Some("body"));
    let body = b"{ not json\n{\"type\":\"message_stop\",\"stop_reason\":\"end_turn\",\"usage\":{\"input_tokens\":1,\"output_tokens\":1},\"api_calls\":1}\n";
    let adapter = StubAdapter::happy(body);
    let err = run_with_stubs(repo.path(), "hi", &adapter, &StubGit::ok()).unwrap_err();
    assert!(matches!(err, Error::AdapterJson(_)));
}

#[test]
fn run_writes_post_terminal_lines_to_disk_without_re_feeding_assembler() {
    // A buggy adapter that emits stray events after message_stop must
    // not break the run: the harness appends them to response.json
    // (faithful diagnostic record) but skips feeding the (already
    // terminal) assembler. The conversation completes normally.
    let body = concat!(
        r#"{"type":"message_start","message":{"id":"m","model":"x","usage":{"input_tokens":1,"output_tokens":0}}}"#,
        "\n",
        r#"{"type":"content_block_start","index":0,"content_block":{"type":"text","text":""}}"#,
        "\n",
        r#"{"type":"text_delta","index":0,"text":"hi"}"#,
        "\n",
        r#"{"type":"content_block_stop","index":0}"#,
        "\n",
        r#"{"type":"message_stop","stop_reason":"end_turn","usage":{"input_tokens":1,"output_tokens":1},"api_calls":1}"#,
        "\n",
        r#"{"type":"text_delta","index":0,"text":"stray post-terminal"}"#,
        "\n",
    );
    let repo = scaffold_repo(VALID_PER_REPO_PROVIDERS_YAML, Some("body"));
    let adapter = StubAdapter::happy(body.as_bytes());
    run_with_stubs(repo.path(), "hi", &adapter, &StubGit::ok())
        .expect("post-terminal lines must not error the run");
}
