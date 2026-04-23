//! Spec-anchored parser tests for the non-streaming response shape.
//!
//! These tests pin the decision made in `docs/ARCHITECTURE.md` §4.4
//! "Response shape (non-streaming)" — the response object is the
//! Anthropic Messages-API wire shape, pass-through; each of `id`,
//! `model`, `stop_reason`, `content`, and `usage.input_tokens` is
//! required; prompt-caching fields use Anthropic's native names.
//!
//! Kept separate from [`super::errors`] so the spec anchor is easy to
//! find (and so that file stays under the repo's per-file line cap).

use super::fixtures::*;
use crate::prompt::{Error, run};

#[test]
fn run_rejects_response_missing_required_fields() {
    let cases: &[(&str, &[u8])] = &[
        (
            "missing id",
            br#"{"model":"m","stop_reason":"end_turn","content":[],"usage":{"input_tokens":1,"output_tokens":1}}"#,
        ),
        (
            "missing model",
            br#"{"id":"x","stop_reason":"end_turn","content":[],"usage":{"input_tokens":1,"output_tokens":1}}"#,
        ),
        (
            "missing stop_reason",
            br#"{"id":"x","model":"m","content":[],"usage":{"input_tokens":1,"output_tokens":1}}"#,
        ),
        (
            "missing content",
            br#"{"id":"x","model":"m","stop_reason":"end_turn","usage":{"input_tokens":1,"output_tokens":1}}"#,
        ),
        (
            "missing usage",
            br#"{"id":"x","model":"m","stop_reason":"end_turn","content":[]}"#,
        ),
        (
            "missing usage.input_tokens",
            br#"{"id":"x","model":"m","stop_reason":"end_turn","content":[],"usage":{"output_tokens":1}}"#,
        ),
    ];
    for (label, body) in cases {
        let repo = scaffold_repo(VALID_PROVIDERS_YAML, VALID_AGENTS_YAML, Some("body"));
        let adapter = StubAdapter::happy(body);
        let git = StubGit::ok();
        let clock = FixedClock::new();
        let id = FixedIdGen;
        let err = run(repo.path(), "hi", &valid_deps(&adapter, &git, &clock, &id)).unwrap_err();
        assert!(
            matches!(err, Error::AdapterJson(_)),
            "{label}: expected AdapterJson, got {err:?}"
        );
    }
}

#[test]
fn run_accepts_response_with_anthropic_native_cache_fields() {
    // Forward-compat check: unknown usage fields with Anthropic's native
    // names parse cleanly. The spec forbids renamed variants
    // (`cache_write_tokens` / `cache_read_tokens`), but the harness
    // parser enforcement here is purely that the native names do not
    // break the path — the naming rule is an adapter-side obligation.
    let body = br#"{
        "id":"msg_01","model":"claude-sonnet-4-7","stop_reason":"end_turn",
        "content":[{"type":"text","text":"hi"}],
        "usage":{
            "input_tokens":1,"output_tokens":1,
            "cache_creation_input_tokens":2,"cache_read_input_tokens":3
        }
    }"#;
    let repo = scaffold_repo(VALID_PROVIDERS_YAML, VALID_AGENTS_YAML, Some("body"));
    let adapter = StubAdapter::happy(body);
    let git = StubGit::ok();
    let clock = FixedClock::new();
    let id = FixedIdGen;
    run(repo.path(), "hi", &valid_deps(&adapter, &git, &clock, &id))
        .expect("native cache field names must parse");
}
