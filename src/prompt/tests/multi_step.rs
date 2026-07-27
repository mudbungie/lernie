//! Multi-step exchange-loop tests. Drives the loop through
//! [`StubToolExecutor`] to assert §2.5 pairing, per-step on-disk shape,
//! and the `Finish{!ToolUse}` termination rule over brazen `v=1` events.

use super::fixtures::*;
use crate::prompt::run;
use brazen::FinishReason;
use serde_json::{Value, json};

fn tool_use_stream(id: &str, name: &str, cmd_key: &str, cmd_val: &str) -> Vec<u8> {
    stream_of(
        FinishReason::ToolUse,
        &[Block::ToolUse {
            id,
            name,
            input: json!({ cmd_key: cmd_val }),
        }],
    )
}

fn final_stream() -> Vec<u8> {
    stream_of(FinishReason::Stop, &[Block::Text("done")])
}

fn last_line_type(bytes: &[u8]) -> String {
    let lines = parse_jsonl(bytes);
    lines.last().unwrap()["type"].as_str().unwrap().to_string()
}

fn finish_reason(bytes: &[u8]) -> String {
    parse_jsonl(bytes)
        .into_iter()
        .find(|e| e["type"] == "finish")
        .unwrap()["reason"]
        .as_str()
        .unwrap()
        .to_string()
}

#[test]
fn loop_runs_two_steps_when_first_completion_is_tool_use() {
    let repo = scaffold_repo(VALID_PER_REPO_PROVIDERS_YAML, Some("body"));
    let harness = scaffold_harness_root();
    let r1 = tool_use_stream("toolu_01", "bash", "cmd", "ls");
    let r2 = final_stream();
    let adapter = StubAdapter::scripted([
        StubAdapter::reply_ok(&version_line()),
        StubAdapter::reply_ok(&r1),
        StubAdapter::reply_ok(&r2),
    ]);
    let git = StubGit::ok();
    let (clock, id) = (FixedClock::default(), FixedIdGen);
    let (sleeper, tool_executor) = (
        StubSleeper::default(),
        StubToolExecutor::with_reply("bash", "files: a b"),
    );

    let branch = run(
        repo.path(),
        "list files",
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
    assert_eq!(branch, "ct-1-deadbeef");
    let worktree = worktree_path(repo.path());

    // Executor saw one call in step 1 with the emitted tool_use.
    let tool_calls = tool_executor.invocations.borrow().clone();
    assert_eq!(tool_calls.len(), 1);
    let (step_dir, tid, name, input) = &tool_calls[0];
    assert_eq!(step_dir, &repo.path().join("steps/ct-1-deadbeef/001"));
    assert_eq!(
        (tid.as_str(), name.as_str(), &input["cmd"]),
        ("toolu_01", "bash", &json!("ls"))
    );

    assert!(!worktree.join("steps").exists());
    let step1_dir = repo.path().join("steps/ct-1-deadbeef/001");
    let step2_dir = repo.path().join("steps/ct-1-deadbeef/002");

    // Step 1 request: one user message, the front-door delivery of the
    // initial message (§2.11) — its deposit frontmatter travels with the
    // body and is model-visible (`deposited_at` is the first clock tick).
    let req1: Value =
        serde_json::from_slice(&std::fs::read(step1_dir.join("request.json")).unwrap()).unwrap();
    assert_eq!(req1["messages"].as_array().unwrap().len(), 1);
    assert_eq!(
        req1["messages"][0]["content"][0]["text"],
        "---\nfrom: user\ndeposited_at: iso-1\n---\nlist files"
    );

    // Step 2 request: §2.5 pairing — assistant tool_use + tool-side
    // tool_result (canonical `Role::Tool`, whose content is a canonical
    // `Content` array; the provider protocol projects the role, §2.3).
    let req2: Value =
        serde_json::from_slice(&std::fs::read(step2_dir.join("request.json")).unwrap()).unwrap();
    let msgs = req2["messages"].as_array().unwrap();
    assert_eq!(msgs.len(), 3);
    assert_eq!(msgs[1]["role"], "assistant");
    assert_eq!(msgs[1]["content"][0]["id"], "toolu_01");
    assert_eq!(msgs[2]["role"], "tool");
    assert_eq!(msgs[2]["content"][0]["tool_use_id"], "toolu_01");
    assert_eq!(msgs[2]["content"][0]["content"][0]["text"], "files: a b");

    assert!(worktree.join("goal.md").exists());
    // Step 2's response terminal is brazen's `end`; the finish reason
    // is `stop`.
    let resp2 = std::fs::read(step2_dir.join("response.json")).unwrap();
    assert_eq!(last_line_type(&resp2), "end");
    assert_eq!(finish_reason(&resp2), "stop");

    // Git op log: 6 (control resolution, §2.2 — config-head rev-parse
    // plus five `show` reads, `version` first for the §10 schema-version
    // guard, manifest.yaml before the soul, §5.2) + 4 (step 1 setup:
    // spawn, control rm, add, commit) + 1 (step-1 drain stray-probe) + 2
    // (user-message delivery add+commit) + 1 (step 1 rev-parse) + 2
    // (step-1 model-output transcript entry add+commit) + 2 (the tool
    // transcript entry add+commit) + 1 (step-2 drain stray-probe) + 1
    // (step 2 rev-parse) + 2 (step-2 model-output entry add+commit) + 1
    // (terminal result-deposit rev-parse, §2.6) = 23. Merge-back is gone
    // (§2.6). The version guard runs no git.
    let runs = git.runs.borrow();
    assert_eq!(runs.len(), 23);
    assert_eq!(runs[8].1, vec!["add", "goal.md", "soul.md"]);
    assert!(runs[9].1[2].contains("step 001: dispatch"));
    // Step-1 drain (§2.11): the clean stray-probe, then the initial user
    // message delivered from the inbox as the first transcript entry (001).
    assert_eq!(runs[10].1, vec!["status", "--porcelain", "--", "messages"]);
    assert_eq!(runs[11].1, vec!["add", "messages/001-user.md"]);
    assert!(runs[12].1[2].contains("transcript 001: user"));
    assert_eq!(runs[13].1, vec!["rev-parse", "HEAD"]);
    // Step 1's transcript: model-output entry (002), then the tool result
    // entry (003) — the §2.3 ordering (model output before its tool
    // results). Counters are max-present-plus-one from the messages/
    // listing, so they never collide with the step number. The model
    // output's origin token is the authoring model id (§2.3).
    assert_eq!(runs[14].1, vec!["add", "messages/002-claude-sonnet-5.json"]);
    assert!(runs[15].1[2].contains("transcript 002: claude-sonnet-5"));
    // A tool commit stages the whole worktree (`git add -A`, §2.3) so any
    // worktree side effect the tool produced lands with its result entry.
    assert_eq!(runs[16].1, vec!["add", "-A"]);
    assert!(runs[17].1[2].contains("transcript 003: tool"));
    // Step 2 opens with its own boundary drain (empty inbox → stray-probe
    // only), then the branch-tip capture (advanced by step 1's transcript
    // commits), then commits its own model-output entry (004).
    assert_eq!(runs[18].1, vec!["status", "--porcelain", "--", "messages"]);
    assert_eq!(runs[19].1, vec!["rev-parse", "HEAD"]);
    assert_eq!(runs[20].1, vec!["add", "messages/004-claude-sonnet-5.json"]);
    assert!(runs[21].1[2].contains("transcript 004: claude-sonnet-5"));
    // The terminal result deposit reads the branch tip (§2.6); no
    // merge-back follows.
    assert_eq!(runs[22].1, vec!["rev-parse", "HEAD"]);

    // The tool entry on disk is the canonical tool_result block.
    let tool_entry = worktree.join("messages/003-tool.json");
    let blocks: Value = serde_json::from_slice(&std::fs::read(&tool_entry).unwrap()).unwrap();
    assert_eq!(blocks[0]["type"], "tool_result");
    assert_eq!(blocks[0]["tool_use_id"], "toolu_01");
    assert_eq!(blocks[0]["content"][0]["text"], "files: a b");
}

#[test]
fn loop_runs_three_steps_when_two_completions_in_a_row_are_tool_use() {
    let repo = scaffold_repo(VALID_PER_REPO_PROVIDERS_YAML, Some("body"));
    let harness = scaffold_harness_root();
    let r1 = tool_use_stream("toolu_01", "bash", "cmd", "ls");
    let r2 = tool_use_stream("toolu_02", "bash", "cmd", "pwd");
    let r3 = final_stream();
    let adapter = StubAdapter::scripted([
        StubAdapter::reply_ok(&version_line()),
        StubAdapter::reply_ok(&r1),
        StubAdapter::reply_ok(&r2),
        StubAdapter::reply_ok(&r3),
    ]);
    let git = StubGit::ok();
    let (clock, id) = (FixedClock::default(), FixedIdGen);
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
            &tool_executor,
            harness.path(),
        ),
    )
    .unwrap();

    let step3_resp = repo.path().join("steps/ct-1-deadbeef/003/response.json");
    assert!(step3_resp.exists());
    assert!(!repo.path().join("steps/ct-1-deadbeef/004").exists());

    let invocations = tool_executor.invocations.borrow().clone();
    assert_eq!(invocations.len(), 2);
    assert!(invocations[0].0.ends_with("steps/ct-1-deadbeef/001"));
    assert!(invocations[1].0.ends_with("steps/ct-1-deadbeef/002"));
    assert_eq!(invocations[0].1, "toolu_01");
    assert_eq!(invocations[1].1, "toolu_02");
}

#[test]
fn loop_runs_each_tool_use_block_in_one_step_in_emission_order() {
    let repo = scaffold_repo(VALID_PER_REPO_PROVIDERS_YAML, Some("body"));
    let harness = scaffold_harness_root();
    let two_tool_use = stream_of(
        FinishReason::ToolUse,
        &[
            Block::ToolUse {
                id: "toolu_a",
                name: "bash",
                input: json!({"cmd": "ls"}),
            },
            Block::ToolUse {
                id: "toolu_b",
                name: "read_file",
                input: json!({"path": "/x"}),
            },
        ],
    );
    let adapter = StubAdapter::scripted([
        StubAdapter::reply_ok(&version_line()),
        StubAdapter::reply_ok(&two_tool_use),
        StubAdapter::reply_ok(&final_stream()),
    ]);
    let git = StubGit::ok();
    let (clock, id) = (FixedClock::default(), FixedIdGen);
    let (sleeper, tool_executor) = (StubSleeper::default(), StubToolExecutor::ok());

    run(
        repo.path(),
        "do two things",
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

    let invocations = tool_executor.invocations.borrow().clone();
    assert_eq!(invocations.len(), 2);
    let pair = |c: &(_, String, String, _)| (c.1.clone(), c.2.clone());
    assert_eq!(pair(&invocations[0]), ("toolu_a".into(), "bash".into()));
    assert_eq!(
        pair(&invocations[1]),
        ("toolu_b".into(), "read_file".into())
    );

    let req2: Value = serde_json::from_slice(
        &std::fs::read(repo.path().join("steps/ct-1-deadbeef/002/request.json")).unwrap(),
    )
    .unwrap();
    let user_blocks = req2["messages"][2]["content"].as_array().unwrap();
    assert_eq!(user_blocks.len(), 2);
    assert_eq!(user_blocks[0]["tool_use_id"], "toolu_a");
    assert_eq!(user_blocks[1]["tool_use_id"], "toolu_b");
}

// Loop-termination cases (tool-executor failure) live in
// [`super::multi_step_terminal`].
