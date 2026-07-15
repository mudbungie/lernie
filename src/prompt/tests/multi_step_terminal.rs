//! Loop-termination cases for the step loop. Split out of
//! [`super::multi_step`] so that file stays under the per-file line cap;
//! the focus here is the `Finish` branch points (a non-`ToolUse` finish
//! terminates without tool work; a tool-executor failure aborts the
//! step).

use super::fixtures::*;
use crate::prompt::{Error, run};
use brazen::FinishReason;
use serde_json::json;

#[test]
fn loop_terminates_on_non_tool_use_finish() {
    // A `Finish{Length}` (max-tokens) terminates without tool work and
    // without a terminal compaction (§2.7 — the stage is deleted).
    let repo = scaffold_repo(VALID_PER_REPO_PROVIDERS_YAML, Some("body"));
    let harness = scaffold_harness_root();
    let r1 = stream_of(FinishReason::Length, &[Block::Text("done")]);
    let adapter = StubAdapter::happy(&r1);
    let git = StubGit::ok();
    let (clock, id) = (FixedClock::default(), FixedIdGen);
    let (sleeper, tool_executor) = (StubSleeper::default(), StubToolExecutor::ok());

    run(
        repo.path(),
        "hi",
        &valid_deps(
            &adapter,
            &sleeper,
            &git,
            &clock,
            &id,
            &tool_executor,
            harness.path(),
        ),
    )
    .unwrap();
    assert!(tool_executor.calls.borrow().is_empty());
}

#[test]
fn loop_surfaces_tool_executor_failure_as_tool_exec_error() {
    let repo = scaffold_repo(VALID_PER_REPO_PROVIDERS_YAML, Some("body"));
    let harness = scaffold_harness_root();
    let r1 = stream_of(
        FinishReason::ToolUse,
        &[Block::ToolUse {
            id: "toolu_01",
            name: "bash",
            input: json!({"cmd": "ls"}),
        }],
    );
    let adapter = StubAdapter::happy(&r1);
    let git = StubGit::ok();
    let (clock, id) = (FixedClock::default(), FixedIdGen);
    let (sleeper, tool_executor) = (StubSleeper::default(), StubToolExecutor::failing_on("bash"));

    let err = run(
        repo.path(),
        "hi",
        &valid_deps(
            &adapter,
            &sleeper,
            &git,
            &clock,
            &id,
            &tool_executor,
            harness.path(),
        ),
    )
    .unwrap_err();
    match err {
        Error::ToolExec { tool, .. } => assert_eq!(tool, "bash"),
        other => panic!("expected ToolExec, got {other:?}"),
    }
}
