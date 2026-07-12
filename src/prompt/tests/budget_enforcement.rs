//! Budget enforcement at the model-call boundary (ARCH §6, v0.7): a
//! conversation that exhausts `max_total_tokens` is stopped before the
//! next adapter invocation and gets the
//! `refs/lernie/budget-exhausted/<branch>` marker; an unbounded workflow
//! never triggers a stop.

use super::fixtures::*;
use crate::prompt::run;
use brazen::FinishReason;
use serde_json::json;

/// `budgets: {max_total_tokens: 8}` — exactly one `stream_of` step's
/// `Usage{input:5, output:3}` = 8 tokens, so the step-2 boundary check
/// trips at `8 >= 8`.
const WORKFLOW_WITH_TOKEN_BUDGET: &str = "events: {}\nbudgets:\n  max_total_tokens: 8\n";

fn tool_use_stream() -> Vec<u8> {
    // Finishes `tool_use` so the loop advances toward a second step.
    stream_of(
        FinishReason::ToolUse,
        &[Block::ToolUse {
            id: "toolu_01",
            name: "bash",
            input: json!({ "cmd": "ls" }),
        }],
    )
}

#[test]
fn exhausted_conversation_stops_before_next_model_call_and_marks_the_ref() {
    let repo = scaffold_repo_with_workflow(
        VALID_PER_REPO_PROVIDERS_YAML,
        WORKFLOW_WITH_TOKEN_BUDGET,
        Some("body"),
    );
    let harness = scaffold_harness_root();
    // Only two adapter replies are scripted (version guard + step 1). Had
    // the budget check failed to stop the loop, step 2 would invoke the
    // adapter a third time and the stub would panic — so "no third call"
    // is enforced structurally as well as asserted below.
    let adapter = StubAdapter::scripted([
        StubAdapter::reply_ok(&version_line()),
        StubAdapter::reply_ok(&tool_use_stream()),
    ]);
    let git = StubGit::ok();
    let (clock, id, dispatcher) = (FixedClock::default(), FixedIdGen, StubDispatcher::ok());
    let (sleeper, tool_executor) = (StubSleeper::default(), StubToolExecutor::ok());

    let branch = run(
        repo.path(),
        "go",
        &valid_deps(
            &adapter,
            &sleeper,
            &git,
            &clock,
            &id,
            &dispatcher,
            &tool_executor,
            harness.path(),
        ),
    )
    .unwrap();
    assert_eq!(branch, "ct-1-deadbeef");

    // Step 1 landed 8 tokens; the step-2 boundary check tripped (8 >= 8),
    // so the adapter was invoked exactly twice: version guard + step 1.
    assert_eq!(adapter.observed.borrow().len(), 2);
    assert!(repo.path().join("steps/ct-1-deadbeef/001").exists());
    // Step 2 was abandoned before its model call — no step-2 record.
    assert!(!repo.path().join("steps/ct-1-deadbeef/002").exists());

    // The git-native marker was written at the branch tip (§6).
    let runs = git.runs.borrow();
    assert!(
        runs.iter().any(|(_, args)| args
            == &vec![
                "update-ref".to_string(),
                "refs/lernie/budget-exhausted/ct-1-deadbeef".to_string(),
                "ct-1-deadbeef".to_string(),
            ]),
        "expected budget-exhausted update-ref; got {runs:?}"
    );
    // Terminal-by-exhaustion: no compaction dispatch, no rebase/merge.
    assert!(dispatcher.calls.borrow().is_empty());
    assert!(
        !runs.iter().any(|(_, args)| {
            let head = args.first().map(String::as_str);
            head == Some("rebase") || head == Some("merge")
        }),
        "exhausted conversation must not merge back; got {runs:?}"
    );
}

#[test]
fn unbounded_workflow_never_triggers_a_budget_stop() {
    // No `budgets:` block → every limit unbounded → the loop runs to a
    // normal terminal completion and dispatches the compactor (baseline).
    let repo = scaffold_repo(VALID_PER_REPO_PROVIDERS_YAML, Some("body"));
    let harness = scaffold_harness_root();
    let adapter = StubAdapter::happy(&happy_response_bytes());
    let git = StubGit::ok();
    let (clock, id, dispatcher) = (FixedClock::default(), FixedIdGen, StubDispatcher::ok());
    let (sleeper, tool_executor) = (StubSleeper::default(), StubToolExecutor::ok());

    run(
        repo.path(),
        "go",
        &valid_deps(
            &adapter,
            &sleeper,
            &git,
            &clock,
            &id,
            &dispatcher,
            &tool_executor,
            harness.path(),
        ),
    )
    .unwrap();

    // Reached the compactor dispatch + merge — no budget stop, no ref.
    assert_eq!(dispatcher.calls.borrow().len(), 1);
    let runs = git.runs.borrow();
    assert!(
        !runs.iter().any(|(_, args)| {
            args.iter()
                .any(|a| a.starts_with("refs/lernie/budget-exhausted/"))
        }),
        "no budget-exhausted ref expected under an unbounded workflow"
    );
}

#[test]
fn budget_ref_write_failure_surfaces_as_a_git_error() {
    // The marker `update-ref` is git op #13 in the exhaustion path (0
    // worktree add, 1 dispatch add, 2 dispatch commit, 3 step-1 drain
    // stray-probe, 4/5 user-message delivery add+commit, 6 step-1
    // rev-parse, 7/8 step-1 model-output transcript add+commit, 9/10 the tool
    // transcript add+commit, 11 step-2 drain stray-probe, 12 step-2
    // rev-parse, 13 mark_exhausted update-ref). Failing it surfaces the §6
    // exhaustion write's error arm.
    let repo = scaffold_repo_with_workflow(
        VALID_PER_REPO_PROVIDERS_YAML,
        WORKFLOW_WITH_TOKEN_BUDGET,
        Some("body"),
    );
    let harness = scaffold_harness_root();
    let adapter = StubAdapter::scripted([
        StubAdapter::reply_ok(&version_line()),
        StubAdapter::reply_ok(&tool_use_stream()),
    ]);
    let git = StubGit::failing_at(13);
    let (clock, id, dispatcher) = (FixedClock::default(), FixedIdGen, StubDispatcher::ok());
    let (sleeper, tool_executor) = (StubSleeper::default(), StubToolExecutor::ok());

    let err = run(
        repo.path(),
        "go",
        &valid_deps(
            &adapter,
            &sleeper,
            &git,
            &clock,
            &id,
            &dispatcher,
            &tool_executor,
            harness.path(),
        ),
    )
    .unwrap_err();
    assert!(
        matches!(
            err,
            crate::prompt::Error::Git {
                op: "budget-exhausted update-ref",
                ..
            }
        ),
        "got {err:?}"
    );
}
