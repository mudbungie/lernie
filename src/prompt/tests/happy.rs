//! Happy-path test: full orchestration with valid inputs. Asserts the
//! exchange record is written with the expected fields, the adapter sees a
//! `describe` then a `complete` call (the latter with the endpoint env
//! var set, no `--endpoint` argv), and the three git invocations land in
//! order.

use super::fixtures::*;
use crate::prompt::run;
use std::ffi::OsStr;

#[test]
fn run_happy_path_writes_exchange_commits_and_returns_sha() {
    let repo = scaffold_repo(VALID_PROVIDERS_YAML, VALID_AGENTS_YAML, Some("system body"));
    let adapter = StubAdapter::happy(HAPPY_RESPONSE_JSON.as_bytes());
    let git = StubGit::ok();
    let clock = FixedClock::new();
    let id = FixedIdGen;

    let sha = run(
        repo.path(),
        "hello",
        &valid_deps(&adapter, &git, &clock, &id),
    )
    .unwrap();
    assert_eq!(sha, "sha-123");

    // Exchange record landed.
    let written = std::fs::read_dir(repo.path().join("exchanges"))
        .unwrap()
        .next()
        .unwrap()
        .unwrap();
    assert!(
        written
            .file_name()
            .to_string_lossy()
            .ends_with("-deadbeef.json"),
        "unexpected: {:?}",
        written.file_name()
    );
    let body: serde_json::Value =
        serde_json::from_slice(&std::fs::read(written.path()).unwrap()).unwrap();
    assert_eq!(body["user_message"], "hello");
    assert_eq!(body["assistant_response"], "hi there");
    assert_eq!(body["model_id"], "claude-sonnet-4-7");
    assert_eq!(body["provider"], "anthropic");
    assert_eq!(body["usage"]["input_tokens"], 3);
    assert_eq!(body["stop_reason"], "end_turn");
    // started_at vs ended_at — clock is called once before adapter.run,
    // once after, so they must differ.
    assert_ne!(body["started_at"], body["ended_at"]);

    // Two adapter invocations: describe (no envs, no stdin) then
    // complete (endpoint env var set from providers.yaml, no --endpoint
    // on argv, request JSON on stdin).
    let calls = adapter.observed.borrow().clone();
    assert_eq!(calls.len(), 2, "expected describe + complete");

    let (binary, args, envs, stdin) = calls[0].clone();
    assert_eq!(binary, OsStr::new("lernie-provider-anthropic"));
    assert_eq!(args, vec!["describe"]);
    assert!(envs.is_empty(), "describe should carry no envs");
    assert!(stdin.is_empty(), "describe takes no stdin");

    let (binary, args, envs, stdin) = calls[1].clone();
    assert_eq!(binary, OsStr::new("lernie-provider-anthropic"));
    assert_eq!(args, vec!["complete"]);
    assert_eq!(
        envs,
        vec![(
            "LERNIE_PROVIDER_ANTHROPIC_ENDPOINT".to_string(),
            "https://api.anthropic.com".to_string()
        )]
    );
    let req: serde_json::Value = serde_json::from_slice(&stdin).unwrap();
    assert_eq!(req["model"], "claude-sonnet-4-7");
    assert_eq!(req["system"], "system body");
    assert_eq!(req["messages"][0]["role"], "user");
    assert_eq!(req["messages"][0]["content"], "hello");

    // Three git invocations in the right order.
    let runs = git.runs.borrow();
    assert_eq!(runs.len(), 3);
    assert_eq!(runs[0][0], "add");
    assert_eq!(runs[1][0], "commit");
    assert!(runs[1][2].contains("exchange deadbeef"));
    assert!(runs[1][2].contains("ARCH §12"));
    assert_eq!(runs[2], vec!["rev-parse", "HEAD"]);
}

#[test]
fn run_describe_without_endpoint_env_field_forwards_no_envs() {
    // An adapter that does not advertise endpoint_env opts out of the
    // harness-set endpoint; `complete` is invoked with an empty env list
    // and the adapter falls back to its built-in default.
    let describe = br#"{"name":"anthropic","schema_version":2,"capabilities":[],
                       "models":[],"auth_env":[]}"#;
    let repo = scaffold_repo(VALID_PROVIDERS_YAML, VALID_AGENTS_YAML, Some("body"));
    let adapter = StubAdapter::scripted([
        StubAdapter::reply_ok(describe),
        StubAdapter::reply_ok(HAPPY_RESPONSE_JSON.as_bytes()),
    ]);
    let git = StubGit::ok();
    let clock = FixedClock::new();
    let id = FixedIdGen;

    run(repo.path(), "hi", &valid_deps(&adapter, &git, &clock, &id)).unwrap();

    let (_, args, envs, _) = adapter.last();
    assert_eq!(args, vec!["complete"]);
    assert!(envs.is_empty(), "no endpoint_env → no envs forwarded");
}
