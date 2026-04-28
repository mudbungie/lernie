//! Loop-termination cases for the v0.3 step loop. Split out of
//! [`super::multi_step`] so that file stays under the repo's per-file
//! line cap; the focus here is the `stop_reason` branch points
//! (`max_tokens` terminates without tool work; tool-executor failure
//! aborts before the compactor runs).

use super::fixtures::*;
use crate::prompt::{Error, run};
use serde_json::json;

fn response_body(stop: &str, blocks: serde_json::Value) -> Vec<u8> {
    streaming_response(stop, &blocks)
}

fn final_text() -> serde_json::Value {
    json!([{"type":"text","text":"done"}])
}

fn tool_use_bash_ls() -> serde_json::Value {
    json!([{"type":"tool_use","id":"toolu_01","name":"bash","input":{"cmd":"ls"}}])
}

#[test]
fn loop_terminates_on_max_tokens_stop_reason() {
    // Any non-tool_use stop_reason terminates; the branch still
    // reaches the compactor + merge-back exactly once.
    let repo = scaffold_repo(VALID_PER_REPO_PROVIDERS_YAML, Some("body"));
    let harness = scaffold_harness_root();
    let r1 = response_body("max_tokens", final_text());
    let adapter = StubAdapter::scripted([
        StubAdapter::reply_ok(STUB_DESCRIBE_JSON.as_bytes()),
        StubAdapter::reply_ok(&r1),
    ]);
    let git = StubGit::ok();
    let clock = FixedClock::default();
    let id = FixedIdGen;
    let dispatcher = StubDispatcher::ok();
    let tool_executor = StubToolExecutor::ok();

    run(
        repo.path(),
        "hi",
        &valid_deps(
            &adapter,
            &git,
            &clock,
            &id,
            &dispatcher,
            &tool_executor,
            harness.path(),
        ),
    )
    .unwrap();
    assert!(tool_executor.calls.borrow().is_empty());
    assert_eq!(dispatcher.calls.borrow().len(), 1);
}

#[test]
fn loop_surfaces_tool_executor_failure_as_tool_exec_error() {
    // Executor errors propagate as Error::ToolExec carrying the tool
    // name — the branch is left unmerged (no compactor dispatch).
    let repo = scaffold_repo(VALID_PER_REPO_PROVIDERS_YAML, Some("body"));
    let harness = scaffold_harness_root();
    let r1 = response_body("tool_use", tool_use_bash_ls());
    let adapter = StubAdapter::scripted([
        StubAdapter::reply_ok(STUB_DESCRIBE_JSON.as_bytes()),
        StubAdapter::reply_ok(&r1),
    ]);
    let git = StubGit::ok();
    let clock = FixedClock::default();
    let id = FixedIdGen;
    let dispatcher = StubDispatcher::ok();
    let tool_executor = StubToolExecutor::failing_on("bash");

    let err = run(
        repo.path(),
        "hi",
        &valid_deps(
            &adapter,
            &git,
            &clock,
            &id,
            &dispatcher,
            &tool_executor,
            harness.path(),
        ),
    )
    .unwrap_err();
    match err {
        Error::ToolExec { tool, .. } => assert_eq!(tool, "bash"),
        other => panic!("expected ToolExec, got {other:?}"),
    }
    assert!(dispatcher.calls.borrow().is_empty());
    // Error::ToolExec formats with context for operators.
    let e = Error::ToolExec {
        tool: "bash".into(),
        source: crate::prompt::ExecError::Spawn {
            name: "bash".into(),
            source: std::io::Error::other("x"),
        },
    };
    assert!(e.to_string().contains("tool bash"));
}
