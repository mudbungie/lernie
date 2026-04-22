//! Happy-path test: full orchestration with valid inputs. Asserts the
//! exchange record is written with the expected fields, the adapter gets
//! the right stdin, and the three git invocations land in order.

use super::fixtures::*;
use crate::prompt::run;
use std::ffi::OsStr;

#[test]
fn run_happy_path_writes_exchange_commits_and_returns_sha() {
    let repo = scaffold_repo(VALID_PROVIDERS_YAML, VALID_AGENTS_YAML, Some("system body"));
    let adapter = StubAdapter::returning_ok(HAPPY_RESPONSE_JSON.as_bytes());
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

    // Adapter was invoked with the right binary name, endpoint, and
    // request-shaped stdin.
    let (binary, args, stdin) = adapter.observed.borrow().clone().unwrap();
    assert_eq!(binary, OsStr::new("lernie-provider-anthropic"));
    assert_eq!(
        args,
        vec!["complete", "--endpoint", "https://api.anthropic.com"]
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
