//! Stream-shape error paths for [`crate::prompt::run`] over brazen
//! `v=1` events (ARCH §4.4 / §3.5).
//!
//! Malformed JSONL surfaces as [`Error::AdapterJson`]; a stream with no
//! trailing `end` (killed mid-stream) surfaces as
//! [`Error::AdapterHalfStream`]; a tool_use block whose `json_delta`
//! buffer is not valid JSON surfaces as `AdapterJson`.

use super::fixtures::*;
use crate::prompt::Error;

#[test]
fn run_surfaces_malformed_jsonl_line() {
    let repo = scaffold_repo(VALID_PER_REPO_PROVIDERS_YAML, Some("body"));
    let adapter = StubAdapter::happy(b"{ not json\n");
    let err = run_with_stubs(repo.path(), "hi", &adapter, &StubGit::ok()).unwrap_err();
    assert!(matches!(err, Error::AdapterJson(_)));
}

#[test]
fn run_surfaces_half_stream_as_adapter_half_stream() {
    // The model call produced no trailing `end` (empty stream here) —
    // the writer died mid-stream (§2.9).
    let repo = scaffold_repo(VALID_PER_REPO_PROVIDERS_YAML, Some("body"));
    let adapter = StubAdapter::happy(b"");
    let err = run_with_stubs(repo.path(), "hi", &adapter, &StubGit::ok()).unwrap_err();
    assert!(matches!(err, Error::AdapterHalfStream));
}

#[test]
fn run_surfaces_invalid_tool_input_json_delta() {
    // A tool_use block whose accumulated `json_delta` is malformed —
    // the harness cannot build the next step's tool input.
    let repo = scaffold_repo(VALID_PER_REPO_PROVIDERS_YAML, Some("body"));
    let body = concat!(
        r#"{"type":"message_start","v":1,"role":"assistant"}"#,
        "\n",
        r#"{"type":"content_start","index":0,"kind":{"tool_use":{"id":"t1","name":"bash"}}}"#,
        "\n",
        r#"{"type":"content_delta","index":0,"delta":{"json_delta":"{ not json"}}"#,
        "\n",
        r#"{"type":"finish","reason":"tool_use"}"#,
        "\n",
        r#"{"type":"end"}"#,
        "\n",
    );
    let adapter = StubAdapter::happy(body.as_bytes());
    let err = run_with_stubs(repo.path(), "hi", &adapter, &StubGit::ok()).unwrap_err();
    assert!(matches!(err, Error::AdapterJson(_)));
}

#[test]
fn run_surfaces_invalid_tool_input_json_delta_at_content_stop() {
    // Same malformed `json_delta`, but the block is `content_stop`'d —
    // so the transcript writer's staging sink (§2.3) parses it and
    // surfaces the `AdapterJson` at the block's stop. The model call
    // commits no model-output entry; the branch tip does not move past
    // the user-message delivery.
    let repo = scaffold_repo(VALID_PER_REPO_PROVIDERS_YAML, Some("body"));
    let body = concat!(
        r#"{"type":"message_start","v":1,"role":"assistant"}"#,
        "\n",
        r#"{"type":"content_start","index":0,"kind":{"tool_use":{"id":"t1","name":"bash"}}}"#,
        "\n",
        r#"{"type":"content_delta","index":0,"delta":{"json_delta":"{ not json"}}"#,
        "\n",
        r#"{"type":"content_stop","index":0}"#,
        "\n",
        r#"{"type":"finish","reason":"tool_use"}"#,
        "\n",
        r#"{"type":"end"}"#,
        "\n",
    );
    let adapter = StubAdapter::happy(body.as_bytes());
    let git = StubGit::ok();
    let err = run_with_stubs(repo.path(), "hi", &adapter, &git).unwrap_err();
    assert!(matches!(err, Error::AdapterJson(_)));
    // No model-output transcript entry was committed: the staging sink
    // erred before the seal-and-rename, so no `git add
    // messages/…-<model-id>.json` ran. (The user-message delivery entry
    // committed earlier, before the model call — that is the on-ramp,
    // not the step's own output.)
    assert!(
        !git.runs
            .borrow()
            .iter()
            .any(|(_, args)| args.first().map(String::as_str) == Some("add")
                && args
                    .get(1)
                    .is_some_and(|a| a.ends_with("-claude-sonnet-5.json"))),
        "no model-output transcript commit on a failed model call"
    );
}

#[test]
fn run_short_circuits_after_first_parse_error() {
    // After the first malformed line, later lines still tee to
    // response.json but the assembler stops being fed — the first parse
    // error is the surfaced one.
    let repo = scaffold_repo(VALID_PER_REPO_PROVIDERS_YAML, Some("body"));
    let body = concat!(
        "{ not json\n",
        r#"{"type":"finish","reason":"stop"}"#,
        "\n",
        r#"{"type":"end"}"#,
        "\n",
    );
    let adapter = StubAdapter::happy(body.as_bytes());
    let err = run_with_stubs(repo.path(), "hi", &adapter, &StubGit::ok()).unwrap_err();
    assert!(matches!(err, Error::AdapterJson(_)));
}

#[test]
fn run_tolerates_post_terminal_stray_lines() {
    // A buggy adapter emitting events after `end` must not break the
    // run: they tee to disk but the (already terminal) assembler
    // ignores them.
    let body = concat!(
        r#"{"type":"message_start","v":1,"role":"assistant"}"#,
        "\n",
        r#"{"type":"content_start","index":0,"kind":{"text":{}}}"#,
        "\n",
        r#"{"type":"content_delta","index":0,"delta":{"text_delta":"hi"}}"#,
        "\n",
        r#"{"type":"finish","reason":"stop"}"#,
        "\n",
        r#"{"type":"end"}"#,
        "\n",
        r#"{"type":"content_delta","index":0,"delta":{"text_delta":"stray"}}"#,
        "\n",
    );
    let repo = scaffold_repo(VALID_PER_REPO_PROVIDERS_YAML, Some("body"));
    let adapter = StubAdapter::happy(body.as_bytes());
    run_with_stubs(repo.path(), "hi", &adapter, &StubGit::ok())
        .expect("post-terminal lines must not error the run");
}
