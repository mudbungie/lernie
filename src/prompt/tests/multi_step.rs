//! v0.3 ball #3 (with v0.3.1 layout): multi-step exchange-loop
//! tests. Drives the loop through [`StubToolExecutor`] to assert
//! §2.5 pairing, per-step on-disk shape, and the `stop_reason !=
//! "tool_use"` termination rule.

use super::fixtures::*;
use crate::prompt::{Error, run};

/// Non-streaming response body with `content` = `blocks_json` and
/// `stop_reason` = `stop`.
fn response_body(stop: &str, blocks_json: &str) -> String {
    format!(
        r#"{{"id":"msg_x","model":"claude-sonnet-4-7","stop_reason":"{stop}",
            "content":{blocks_json},
            "usage":{{"input_tokens":5,"output_tokens":3}}}}"#
    )
}

const TOOL_USE_BASH_LS: &str =
    r#"[{"type":"tool_use","id":"toolu_01","name":"bash","input":{"cmd":"ls"}}]"#;
const FINAL_TEXT: &str = r#"[{"type":"text","text":"done"}]"#;

#[test]
fn loop_runs_two_steps_when_first_response_is_tool_use() {
    // Step 1 returns tool_use → loop runs the executor stub, builds
    // step 2's request with the tool_result, calls the model again;
    // step 2 returns end_turn → loop terminates and the compactor +
    // merge-back fires once.
    let repo = scaffold_repo(VALID_PER_REPO_PROVIDERS_YAML, Some("body"));
    let harness = scaffold_harness_root();
    let adapter = StubAdapter::scripted([
        StubAdapter::reply_ok(STUB_DESCRIBE_JSON.as_bytes()),
        StubAdapter::reply_ok(response_body("tool_use", TOOL_USE_BASH_LS).as_bytes()),
        StubAdapter::reply_ok(response_body("end_turn", FINAL_TEXT).as_bytes()),
    ]);
    let git = StubGit::ok();
    let clock = FixedClock::default();
    let id = FixedIdGen;
    let dispatcher = StubDispatcher::ok();
    let tool_executor = StubToolExecutor::with_reply("bash", "files: a b");

    let branch = run(
        repo.path(),
        "list files",
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
    assert_eq!(branch, "ct-1-deadbeef");
    let worktree = repo.path().join("ct-1-deadbeef");

    // Executor saw exactly one call in step 1 with the emitted
    // tool_use's id/name/input. The step_dir end-segment carries the
    // step seq AND lives at the conv-repo root, outside any
    // worktree (§2.2 / §2.3).
    let tool_calls = tool_executor.calls.borrow().clone();
    assert_eq!(tool_calls.len(), 1);
    let (step_dir, id, name, input) = &tool_calls[0];
    assert_eq!(step_dir, &repo.path().join("steps/ct-1-deadbeef/001"));
    assert_eq!(
        (id.as_str(), name.as_str(), &input["cmd"]),
        ("toolu_01", "bash", &serde_json::json!("ls"))
    );

    // Step records live at the conv-repo root (§2.2). Step 1 request:
    // bare user string. Step 2 request: §2.5 pairing — assistant
    // tool_use + user tool_result.
    assert!(
        !worktree.join("steps").exists(),
        "step records must not land inside any worktree (§2.2)"
    );
    let step1_dir = repo.path().join("steps/ct-1-deadbeef/001");
    let step2_dir = repo.path().join("steps/ct-1-deadbeef/002");
    let req1: serde_json::Value =
        serde_json::from_slice(&std::fs::read(step1_dir.join("request.json")).unwrap()).unwrap();
    assert_eq!(req1["messages"].as_array().unwrap().len(), 1);
    assert_eq!(req1["messages"][0]["content"], "list files");
    let req2: serde_json::Value =
        serde_json::from_slice(&std::fs::read(step2_dir.join("request.json")).unwrap()).unwrap();
    let msgs = req2["messages"].as_array().unwrap();
    assert_eq!(msgs.len(), 3);
    assert_eq!(msgs[1]["role"], "assistant");
    assert_eq!(msgs[1]["content"][0]["id"], "toolu_01");
    assert_eq!(msgs[2]["role"], "user");
    assert_eq!(msgs[2]["content"][0]["tool_use_id"], "toolu_01");
    assert_eq!(msgs[2]["content"][0]["content"], "files: a b");

    // Step 2 has no dispatch artifact — goal/soul live on the branch
    // tip from step 1 (§2.10 — step ≥2 has no pre-call commit).
    assert!(worktree.join("goal.md").exists());
    let resp2: serde_json::Value =
        serde_json::from_slice(&std::fs::read(step2_dir.join("response.json")).unwrap()).unwrap();
    assert_eq!(resp2["stop_reason"], "end_turn");
    assert_eq!(dispatcher.calls.borrow().len(), 1);

    // Git op log: 3 (step 1: worktree add + dispatch add + commit) +
    // 2 (rev-parse per step, 2 steps) + 6 (merge-back) = 11. No
    // per-step request/response commits, no per-tool-call commits —
    // step records are diagnostic-only and live outside every
    // worktree (§2.3, §3.3 amended).
    let runs = git.runs.borrow();
    assert_eq!(runs.len(), 11);
    // Step 1 ops: worktree add (0), dispatch add goal+soul (1),
    // dispatch commit (2), rev-parse for step 1's meta (3).
    assert_eq!(runs[1].1, vec!["add", "goal.md", "soul.md"]);
    assert!(runs[2].1[2].contains("step 001: dispatch"));
    assert_eq!(runs[3].1, vec!["rev-parse", "HEAD"]);
    // Step 2: only rev-parse (no dispatch commit).
    assert_eq!(runs[4].1, vec!["rev-parse", "HEAD"]);
    // 5..11 are merge-back.
    assert_eq!(runs[5].1, vec!["rebase", "main"]);
}

#[test]
fn loop_runs_three_steps_when_two_responses_in_a_row_are_tool_use() {
    // Step 1: tool_use; Step 2: tool_use; Step 3: end_turn. Verifies
    // the loop continues iterating, each new request rolls in the
    // prior assistant + tool_result, and the step seq lands on 003.
    let repo = scaffold_repo(VALID_PER_REPO_PROVIDERS_YAML, Some("body"));
    let harness = scaffold_harness_root();
    let tool_use_2 = r#"[{"type":"tool_use","id":"toolu_02","name":"bash","input":{"cmd":"pwd"}}]"#;
    let adapter = StubAdapter::scripted([
        StubAdapter::reply_ok(STUB_DESCRIBE_JSON.as_bytes()),
        StubAdapter::reply_ok(response_body("tool_use", TOOL_USE_BASH_LS).as_bytes()),
        StubAdapter::reply_ok(response_body("tool_use", tool_use_2).as_bytes()),
        StubAdapter::reply_ok(response_body("end_turn", FINAL_TEXT).as_bytes()),
    ]);
    let git = StubGit::ok();
    let clock = FixedClock::default();
    let id = FixedIdGen;
    let dispatcher = StubDispatcher::ok();
    let tool_executor = StubToolExecutor::ok();

    run(
        repo.path(),
        "go",
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

    // Step records sit at the conv-repo root (§2.2 / §2.3).
    let step3_resp = repo.path().join("steps/ct-1-deadbeef/003/response.json");
    assert!(step3_resp.exists());
    assert!(!repo.path().join("steps/ct-1-deadbeef/004").exists());

    let calls = tool_executor.calls.borrow().clone();
    assert_eq!(calls.len(), 2);
    // Step seq is encoded in the per-call step_dir tail.
    assert!(calls[0].0.ends_with("steps/ct-1-deadbeef/001"));
    assert!(calls[1].0.ends_with("steps/ct-1-deadbeef/002"));
    assert_eq!(calls[0].1, "toolu_01");
    assert_eq!(calls[1].1, "toolu_02");
}

#[test]
fn loop_runs_each_tool_use_block_in_one_step_in_emission_order() {
    // One step, two tool_use blocks → two executor calls in the
    // emitted order, two tool_result blocks on the next user message.
    let repo = scaffold_repo(VALID_PER_REPO_PROVIDERS_YAML, Some("body"));
    let harness = scaffold_harness_root();
    let two_tool_use = r#"[
        {"type":"tool_use","id":"toolu_a","name":"bash","input":{"cmd":"ls"}},
        {"type":"tool_use","id":"toolu_b","name":"read_file","input":{"path":"/x"}}
    ]"#;
    let adapter = StubAdapter::scripted([
        StubAdapter::reply_ok(STUB_DESCRIBE_JSON.as_bytes()),
        StubAdapter::reply_ok(response_body("tool_use", two_tool_use).as_bytes()),
        StubAdapter::reply_ok(response_body("end_turn", FINAL_TEXT).as_bytes()),
    ]);
    let git = StubGit::ok();
    let clock = FixedClock::default();
    let id = FixedIdGen;
    let dispatcher = StubDispatcher::ok();
    let tool_executor = StubToolExecutor::ok();

    run(
        repo.path(),
        "do two things",
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

    let calls = tool_executor.calls.borrow().clone();
    assert_eq!(calls.len(), 2);
    let pair = |c: &(_, String, String, _)| (c.1.clone(), c.2.clone());
    assert_eq!(pair(&calls[0]), ("toolu_a".into(), "bash".into()));
    assert_eq!(pair(&calls[1]), ("toolu_b".into(), "read_file".into()));

    let req2: serde_json::Value = serde_json::from_slice(
        &std::fs::read(repo.path().join("steps/ct-1-deadbeef/002/request.json")).unwrap(),
    )
    .unwrap();
    let user_blocks = req2["messages"][2]["content"].as_array().unwrap();
    assert_eq!(user_blocks.len(), 2);
    assert_eq!(user_blocks[0]["tool_use_id"], "toolu_a");
    assert_eq!(user_blocks[1]["tool_use_id"], "toolu_b");
}

#[test]
fn loop_terminates_on_max_tokens_stop_reason() {
    // Any non-tool_use stop_reason terminates; the branch still
    // reaches the compactor + merge-back exactly once.
    let repo = scaffold_repo(VALID_PER_REPO_PROVIDERS_YAML, Some("body"));
    let harness = scaffold_harness_root();
    let adapter = StubAdapter::scripted([
        StubAdapter::reply_ok(STUB_DESCRIBE_JSON.as_bytes()),
        StubAdapter::reply_ok(response_body("max_tokens", FINAL_TEXT).as_bytes()),
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
    let adapter = StubAdapter::scripted([
        StubAdapter::reply_ok(STUB_DESCRIBE_JSON.as_bytes()),
        StubAdapter::reply_ok(response_body("tool_use", TOOL_USE_BASH_LS).as_bytes()),
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
