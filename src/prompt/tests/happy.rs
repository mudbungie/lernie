//! Happy-path test: full v0.3 orchestration with valid inputs.
//!
//! Asserts the conversation branch is spawned, the goal and soul are
//! committed at the worktree root, the snapshot commit lands before
//! the model call, and the response lands as a follow-up commit
//! after. Compaction itself is exercised by the compactor module's
//! own tests; here we only assert the dispatcher was called with the
//! right repo + branch (ARCH §3.4).

use super::fixtures::*;
use crate::prompt::run;
use crate::template::ROOT_WORKTREE;
use std::ffi::OsStr;

#[test]
fn run_happy_path_writes_branch_worktree_and_two_commits() {
    let repo = scaffold_repo(VALID_PER_REPO_PROVIDERS_YAML, Some("system body"));
    let harness = scaffold_harness_root();
    let adapter = StubAdapter::happy(HAPPY_RESPONSE_JSON.as_bytes());
    let git = StubGit::ok();
    let clock = FixedClock::default();
    let id = FixedIdGen;
    let dispatcher = StubDispatcher::ok();

    let branch = run(
        repo.path(),
        "hello",
        &valid_deps(&adapter, &git, &clock, &id, &dispatcher, harness.path()),
    )
    .unwrap();
    // Branch name is the bare conv-id — no `ex/` prefix in v0.3
    // (ARCH §2.3).
    assert_eq!(branch, "ct-1-deadbeef");

    // Conversation worktree is a sibling of `root/`, named by the
    // conv-id (ARCH §2.2).
    let worktree = repo.path().join("ct-1-deadbeef");
    let primary_worktree = repo.path().join(ROOT_WORKTREE);

    // Goal pinned at `goal.md` on the branch's worktree (§2.8).
    let goal = std::fs::read_to_string(worktree.join("goal.md")).unwrap();
    assert_eq!(goal, "hello");
    // Soul committed at `soul.md` on the branch's worktree (§4.3).
    let soul = std::fs::read_to_string(worktree.join("soul.md")).unwrap();
    assert_eq!(soul, "system body");

    // Snapshot commit's artifact: request.json at
    // steps/<conv-id>/001/request.json (§2.3, §2.10).
    let step_dir = worktree.join("steps/ct-1-deadbeef/001");
    let request: serde_json::Value =
        serde_json::from_slice(&std::fs::read(step_dir.join("request.json")).unwrap()).unwrap();
    assert_eq!(request["model"], "claude-sonnet-4-7");
    // Goal is prepended to the soul so it sits at the head of context
    // (§2.8). Soul still follows.
    assert_eq!(
        request["system"].as_str().unwrap(),
        "<goal>\nhello\n</goal>\n\nsystem body"
    );
    assert_eq!(request["messages"][0]["role"], "user");
    assert_eq!(request["messages"][0]["content"], "hello");

    // Follow-up commit's artifact: response.json with normalized
    // harness-owned fields (not the raw Anthropic response). The
    // assistant message is committed as structured content blocks
    // (§3.3) — text + tool_use survive in `content`.
    let response: serde_json::Value =
        serde_json::from_slice(&std::fs::read(step_dir.join("response.json")).unwrap()).unwrap();
    assert_eq!(response["content"][0]["type"], "text");
    assert_eq!(response["content"][0]["text"], "hi there");
    assert_eq!(response["model_id"], "claude-sonnet-4-7");
    assert_eq!(response["provider"], "anthropic");
    assert_eq!(response["stop_reason"], "end_turn");
    assert_eq!(response["usage"]["input_tokens"], 3);
    assert_eq!(response["usage"]["output_tokens"], 2);
    // ISO clock is called once before adapter.complete, once after.
    assert_eq!(response["started_at"], "iso-1");
    assert_eq!(response["ended_at"], "iso-2");

    // Adapter was called twice: describe (no envs, no stdin) then
    // complete (endpoint env var set, request JSON on stdin).
    let calls = adapter.observed.borrow().clone();
    assert_eq!(calls.len(), 2, "expected describe + complete");
    let (binary, args, envs, stdin) = calls[0].clone();
    assert_eq!(binary, OsStr::new("lernie-provider-anthropic"));
    assert_eq!(args, vec!["describe"]);
    assert!(envs.is_empty());
    assert!(stdin.is_empty());
    let (_, args, envs, stdin) = calls[1].clone();
    assert_eq!(args, vec!["complete"]);
    assert_eq!(
        envs,
        vec![(
            "LERNIE_PROVIDER_ANTHROPIC_ENDPOINT".to_string(),
            "https://api.anthropic.com".to_string()
        )]
    );
    // Complete's stdin matches the committed request.json byte-for-
    // byte in content (pretty-printing differs, so compare parsed).
    let wire: serde_json::Value = serde_json::from_slice(&stdin).unwrap();
    assert_eq!(wire, request);

    // Compactor was dispatched via the CLI surface (§3.4): the
    // dispatcher saw the repo + conversation branch exactly once.
    let dispatches = dispatcher.calls.borrow().clone();
    assert_eq!(dispatches.len(), 1);
    assert_eq!(dispatches[0].0, repo.path());
    assert_eq!(dispatches[0].1, "ct-1-deadbeef");

    // Git sequence (cmp internals are now behind the dispatcher
    // boundary): 5 for the conversation branch (worktree add,
    // snapshot add+commit, response add+commit), then 3 for
    // merge-to-main (rebase, merge, remove conv worktree). No
    // sidecar-file write: the git ref database is the single source
    // of truth for branch state (PRINCIPLES.md "Single source of
    // truth").
    let runs = git.runs.borrow();
    assert_eq!(runs.len(), 8);

    // [0..5] conversation-branch setup (pre-dispatch).
    // worktree add runs inside the primary worktree (root/), since
    // that is where the .git directory lives (§2.2).
    let (dest0, args0) = &runs[0];
    assert_eq!(dest0, &primary_worktree);
    assert_eq!(args0[..4], ["worktree", "add", "-b", "ct-1-deadbeef"]);
    assert_eq!(args0[4], worktree.to_string_lossy().to_string());
    assert_eq!(args0[5], "main");
    for (dest, _args) in &runs[1..5] {
        assert_eq!(dest, &worktree, "post-spawn git runs inside conv worktree");
    }
    assert_eq!(runs[1].1[0], "add");
    // Snapshot add: goal + soul + request, all in one git add so the
    // dispatch commit's tree carries the full §2.8/§4.3 surface.
    assert_eq!(runs[1].1[1], "goal.md");
    assert_eq!(runs[1].1[2], "soul.md");
    assert_eq!(runs[1].1[3], "steps/ct-1-deadbeef/001/request.json");
    assert_eq!(runs[2].1[0], "commit");
    assert!(runs[2].1[2].contains("step 001: dispatch"));
    assert!(runs[2].1[2].contains("[ct-1-deadbeef]"));
    assert_eq!(runs[3].1[0], "add");
    assert_eq!(runs[3].1[1], "steps/ct-1-deadbeef/001/response.json");
    assert_eq!(runs[4].1[0], "commit");
    assert!(runs[4].1[2].contains("step 001: response"));

    // [5..8] merge conv into main: rebase conv onto main (inside
    // conv wt), merge --no-ff conv into main (inside primary
    // worktree, which is where main is checked out), remove conv
    // worktree.
    assert_eq!(runs[5].0, worktree);
    assert_eq!(runs[5].1, vec!["rebase", "main"]);
    assert_eq!(runs[6].0, primary_worktree);
    assert_eq!(runs[6].1, vec!["merge", "--no-ff", "ct-1-deadbeef"]);
    assert_eq!(runs[7].0, primary_worktree);
    assert_eq!(runs[7].1[0], "worktree");
    assert_eq!(runs[7].1[1], "remove");
    assert_eq!(runs[7].1[2], worktree.to_string_lossy().to_string());
}

#[test]
fn run_describe_without_endpoint_env_field_forwards_no_envs() {
    // An adapter that does not advertise endpoint_env opts out of the
    // harness-set endpoint; `complete` is invoked with an empty env
    // list and the adapter falls back to its built-in default.
    let describe = br#"{"name":"anthropic","schema_version":2,"capabilities":[],
                       "models":[],"auth_env":[]}"#;
    let repo = scaffold_repo(VALID_PER_REPO_PROVIDERS_YAML, Some("body"));
    let harness = scaffold_harness_root();
    let adapter = StubAdapter::scripted([
        StubAdapter::reply_ok(describe),
        StubAdapter::reply_ok(HAPPY_RESPONSE_JSON.as_bytes()),
    ]);
    let git = StubGit::ok();
    let clock = FixedClock::default();
    let id = FixedIdGen;
    let dispatcher = StubDispatcher::ok();

    run(
        repo.path(),
        "hi",
        &valid_deps(&adapter, &git, &clock, &id, &dispatcher, harness.path()),
    )
    .unwrap();

    let (_, args, envs, _) = adapter.last();
    assert_eq!(args, vec!["complete"]);
    assert!(envs.is_empty(), "no endpoint_env → no envs forwarded");
}
