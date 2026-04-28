//! Spec-anchored parser tests for the §4.4 streaming wire shape on
//! the harness side.
//!
//! These tests pin the per-line decisions made in
//! `docs/ARCHITECTURE.md` §4.4 "Response shape (streaming)" — every
//! complete invocation produces JSON Lines, the terminal event is
//! `message_stop` (or `error`), and `kind`/`message` on an in-band
//! error event are required fields.
//!
//! Kept separate from [`super::errors`] so the spec anchor stays
//! findable (and so each file stays under the repo's per-file line
//! cap).

use super::fixtures::*;
use crate::prompt::Error;

#[test]
fn run_rejects_message_stop_with_required_field_omitted() {
    // Per-stream cases the harness must reject. Each body is one
    // JSONL line. The harness surfaces every one as AdapterJson
    // because StreamEvent's tag-based deserialization rejects
    // missing required fields.
    let cases: &[(&str, &[u8])] = &[
        (
            "message_stop missing usage",
            b"{\"type\":\"message_stop\",\"stop_reason\":\"end_turn\",\"api_calls\":1}\n",
        ),
        (
            "message_stop missing api_calls",
            b"{\"type\":\"message_stop\",\"stop_reason\":\"end_turn\",\"usage\":{\"input_tokens\":1,\"output_tokens\":1}}\n",
        ),
        (
            "error missing kind",
            b"{\"type\":\"error\",\"message\":\"boom\"}\n",
        ),
        (
            "error missing message",
            b"{\"type\":\"error\",\"kind\":\"fatal\"}\n",
        ),
    ];
    for (label, body) in cases {
        let repo = scaffold_repo(VALID_PER_REPO_PROVIDERS_YAML, Some("body"));
        let adapter = StubAdapter::happy(body);
        let err = run_with_stubs(repo.path(), "hi", &adapter, &StubGit::ok()).unwrap_err();
        assert!(
            matches!(err, Error::AdapterJson(_)),
            "{label}: expected AdapterJson, got {err:?}"
        );
    }
}

#[test]
fn run_accepts_message_stop_without_optional_stop_reason_when_message_start_carries_it() {
    // §4.4 lets `stop_reason` arrive on `message_start.message`,
    // `message_delta`, or the terminal `message_stop`. A stop event
    // with no `stop_reason` is fine if message_start already carried
    // one.
    let body = concat!(
        r#"{"type":"message_start","message":{"id":"m","model":"x","stop_reason":"end_turn","usage":{"input_tokens":1,"output_tokens":0}}}"#,
        "\n",
        r#"{"type":"message_stop","usage":{"input_tokens":1,"output_tokens":2},"api_calls":1}"#,
        "\n",
    );
    let repo = scaffold_repo(VALID_PER_REPO_PROVIDERS_YAML, Some("body"));
    let adapter = StubAdapter::happy(body.as_bytes());
    run_with_stubs(repo.path(), "hi", &adapter, &StubGit::ok())
        .expect("stop_reason from message_start must be honored");
}
