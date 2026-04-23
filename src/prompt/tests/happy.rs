//! Happy-path test: full v0.2 orchestration with valid inputs.
//!
//! Asserts the exchange branch is spawned, the goal is pinned, the
//! snapshot commit lands before the model call, and the response
//! lands as a follow-up commit after.

use super::fixtures::*;
use crate::prompt::run;
use std::ffi::OsStr;

#[test]
fn run_happy_path_writes_branch_worktree_and_two_commits() {
    let repo = scaffold_repo(VALID_PROVIDERS_YAML, VALID_AGENTS_YAML, Some("system body"));
    let adapter = StubAdapter::happy(HAPPY_RESPONSE_JSON.as_bytes());
    let git = StubGit::ok();
    let clock = FixedClock::new();
    let id = FixedIdGen;

    let branch = run(
        repo.path(),
        "hello",
        &valid_deps(&adapter, &git, &clock, &id),
    )
    .unwrap();
    assert_eq!(branch, "ex/ct-1-deadbeef");

    let worktree = repo.path().join(".lernie/worktrees/ex/ct-1-deadbeef");

    // Goal pinned at `.agent/goal.md` on the branch's worktree (§2.8).
    let goal = std::fs::read_to_string(worktree.join(".agent/goal.md")).unwrap();
    assert_eq!(goal, "hello");

    // Snapshot commit's artifact: request.json at
    // exchanges/<exchange-id>/steps/001/request.json (§2.3, §2.10).
    let step_dir = worktree.join("exchanges/ct-1-deadbeef/steps/001");
    let request: serde_json::Value =
        serde_json::from_slice(&std::fs::read(step_dir.join("request.json")).unwrap()).unwrap();
    assert_eq!(request["model"], "claude-sonnet-4-7");
    // Goal is prepended to system so it sits at the head of context
    // (§2.8). Base role prompt still follows.
    assert_eq!(
        request["system"].as_str().unwrap(),
        "<goal>\nhello\n</goal>\n\nsystem body"
    );
    assert_eq!(request["messages"][0]["role"], "user");
    assert_eq!(request["messages"][0]["content"], "hello");

    // Follow-up commit's artifact: response.json with normalized
    // harness-owned fields (not the raw Anthropic response).
    let response: serde_json::Value =
        serde_json::from_slice(&std::fs::read(step_dir.join("response.json")).unwrap()).unwrap();
    assert_eq!(response["assistant_response"], "hi there");
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

    // Git sequence: 5 for the exchange branch (worktree add, snapshot
    // add+commit, response add+commit), 6 for compaction (spawn cmp,
    // add+commit summary, rebase-merge-remove back into ex), 3 for
    // merge-to-main (rebase, merge, remove ex worktree) — 14 total.
    // No sidecar-file write: the git ref database is the single
    // source of truth for branch state (PRINCIPLES.md "Single source
    // of truth").
    let runs = git.runs.borrow();
    assert_eq!(runs.len(), 14);

    // [0..5] exchange-branch setup (pre-compaction).
    let (dest0, args0) = &runs[0];
    assert_eq!(dest0, repo.path());
    assert_eq!(args0[..4], ["worktree", "add", "-b", "ex/ct-1-deadbeef"]);
    assert_eq!(args0[4], worktree.to_string_lossy().to_string());
    assert_eq!(args0[5], "main");
    for (dest, _args) in &runs[1..5] {
        assert_eq!(dest, &worktree, "post-spawn git runs inside worktree");
    }
    assert_eq!(runs[1].1[0], "add");
    assert_eq!(runs[1].1[1], ".agent/goal.md");
    assert_eq!(
        runs[1].1[2],
        "exchanges/ct-1-deadbeef/steps/001/request.json"
    );
    assert_eq!(runs[2].1[0], "commit");
    assert!(runs[2].1[2].contains("step 001: dispatch"));
    assert!(runs[2].1[2].contains("ex ct-1-deadbeef"));
    assert_eq!(runs[3].1[0], "add");
    assert_eq!(
        runs[3].1[1],
        "exchanges/ct-1-deadbeef/steps/001/response.json"
    );
    assert_eq!(runs[4].1[0], "commit");
    assert!(runs[4].1[2].contains("step 001: response"));

    // [5] compactor branch spawned off the exchange tip.
    let cmp_branch = "inv/ct-1-deadbeef/ct-2-deadbeef";
    let cmp_worktree = repo
        .path()
        .join(".lernie/worktrees/inv/ct-1-deadbeef/ct-2-deadbeef");
    assert_eq!(runs[5].0, repo.path());
    assert_eq!(runs[5].1[..4], ["worktree", "add", "-b", cmp_branch]);
    assert_eq!(runs[5].1[4], cmp_worktree.to_string_lossy().to_string());
    assert_eq!(runs[5].1[5], "ex/ct-1-deadbeef");

    // [6..8] compactor adds + commits the summary inside cmp wt.
    assert_eq!(runs[6].0, cmp_worktree);
    assert_eq!(runs[6].1[0], "add");
    assert_eq!(runs[6].1[1], ".agent/compactions/001.md");
    assert_eq!(runs[7].0, cmp_worktree);
    assert_eq!(runs[7].1[0], "commit");
    assert!(runs[7].1[2].contains("compaction: terminal summary"));

    // [8] cmp rebases onto ex. [9] ex merges cmp with --no-ff. [10]
    // cmp worktree removed.
    assert_eq!(runs[8].0, cmp_worktree);
    assert_eq!(runs[8].1, vec!["rebase", "ex/ct-1-deadbeef"]);
    assert_eq!(runs[9].0, worktree);
    assert_eq!(runs[9].1, vec!["merge", "--no-ff", cmp_branch]);
    assert_eq!(runs[10].0, repo.path());
    assert_eq!(runs[10].1[0], "worktree");
    assert_eq!(runs[10].1[1], "remove");
    assert_eq!(runs[10].1[2], cmp_worktree.to_string_lossy().to_string());

    // [11..14] merge ex into main: rebase ex onto main (inside ex
    // wt), merge --no-ff ex into main (inside repo root, which is
    // main's worktree), remove ex worktree.
    assert_eq!(runs[11].0, worktree);
    assert_eq!(runs[11].1, vec!["rebase", "main"]);
    assert_eq!(runs[12].0, repo.path());
    assert_eq!(runs[12].1, vec!["merge", "--no-ff", "ex/ct-1-deadbeef"]);
    assert_eq!(runs[13].0, repo.path());
    assert_eq!(runs[13].1[0], "worktree");
    assert_eq!(runs[13].1[1], "remove");
    assert_eq!(runs[13].1[2], worktree.to_string_lossy().to_string());

    // Compaction summary was written with the expected body.
    let summary = std::fs::read_to_string(cmp_worktree.join(".agent/compactions/001.md")).unwrap();
    assert_eq!(summary, "exchange ct-1-deadbeef: hi there\n");
}

#[test]
fn run_describe_without_endpoint_env_field_forwards_no_envs() {
    // An adapter that does not advertise endpoint_env opts out of the
    // harness-set endpoint; `complete` is invoked with an empty env
    // list and the adapter falls back to its built-in default.
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
