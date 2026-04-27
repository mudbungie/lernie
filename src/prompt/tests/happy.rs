//! Happy-path test: full v0.3.1 orchestration with valid inputs.
//!
//! Asserts the conversation branch is spawned, the dispatch commit
//! lays goal.md + soul.md (only — no `request.json` in the
//! committed tree per amended §2.10), and the diagnostic step record
//! (request.json, response.json, meta.json) lands at the conv-repo
//! root outside the worktree (§2.2 / §2.3). Compaction itself is
//! exercised by the compactor module's own tests; here we only
//! assert the dispatcher was called with the right repo + branch
//! (ARCH §3.4).

use super::fixtures::*;
use crate::prompt::run;
use crate::prompt::step::StepMeta;
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
    let tool_executor = StubToolExecutor::ok();

    let branch = run(
        repo.path(),
        "hello",
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

    // Step record lives at the conv-repo root, *outside* the
    // worktree (§2.2 / §2.3 — diagnostic-only contract). The
    // worktree no longer contains a `steps/` tree.
    assert!(
        !worktree.join("steps").exists(),
        "step records must not land inside any worktree (§2.2)"
    );
    let step_dir = repo.path().join("steps/ct-1-deadbeef/001");
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

    // response.json lives next to request.json with the harness-owned
    // normalized shape (not the raw Anthropic response). The
    // assistant message is recorded as structured content blocks
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

    // meta.json carries the branch-tip sha at step-start (§2.10
    // replay state) plus matching timestamps. The stub git's
    // run_capture returns "", so the `commit` field is the empty
    // string here — production reads `git rev-parse HEAD`.
    let meta: StepMeta =
        serde_json::from_slice(&std::fs::read(step_dir.join("meta.json")).unwrap()).unwrap();
    assert_eq!(meta.commit, "");
    assert_eq!(meta.started_at, "iso-1");
    assert_eq!(meta.ended_at, "iso-2");

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
    // Complete's stdin matches the on-disk request.json byte-for-
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
    // boundary): 4 for the conversation branch (worktree add,
    // dispatch add, dispatch commit, rev-parse capturing the tip
    // sha for meta.json), then 6 for merge-to-main (rebase,
    // merge=ours rm + ls-tree + diff with the stub returning empty
    // captures so neither the alignment checkout nor the alignment
    // commit fires, then merge --no-ff and remove conv worktree).
    // No per-step request/response commits — those records are
    // diagnostic-only and live outside the worktree (§2.3).
    let runs = git.runs.borrow();
    assert_eq!(runs.len(), 10);

    // [0..4] conversation-branch setup (pre-dispatch).
    // worktree add runs inside the primary worktree (root/), since
    // that is where the .git directory lives (§2.2).
    let (dest0, args0) = &runs[0];
    assert_eq!(dest0, &primary_worktree);
    assert_eq!(args0[..4], ["worktree", "add", "-b", "ct-1-deadbeef"]);
    assert_eq!(args0[4], worktree.to_string_lossy().to_string());
    assert_eq!(args0[5], "main");
    for (dest, _args) in &runs[1..4] {
        assert_eq!(dest, &worktree, "post-spawn git runs inside conv worktree");
    }
    // Dispatch add stages goal + soul ONLY (§2.10 — request.json is
    // diagnostic and not committed).
    assert_eq!(runs[1].1, vec!["add", "goal.md", "soul.md"]);
    assert_eq!(runs[2].1[0], "commit");
    assert!(runs[2].1[2].contains("step 001: dispatch"));
    assert!(runs[2].1[2].contains("[ct-1-deadbeef]"));
    // Branch-tip capture for meta.json.
    assert_eq!(runs[3].1, vec!["rev-parse", "HEAD"]);

    // [4..10] merge conv into main: rebase conv onto main, then the
    // merge=ours alignment (rm + ls-tree + diff — captures empty so
    // the conditional checkout and alignment commit are skipped),
    // then merge --no-ff and remove the conv worktree. The alignment
    // runs in the conv worktree; the merge and worktree-remove run
    // in the primary worktree, where main is checked out (§2.2).
    assert_eq!(runs[4].0, worktree);
    assert_eq!(runs[4].1, vec!["rebase", "main"]);
    assert_eq!(runs[5].0, worktree);
    assert_eq!(runs[5].1[0], "rm");
    assert_eq!(runs[6].0, worktree);
    assert_eq!(runs[6].1[0], "ls-tree");
    assert_eq!(runs[7].0, worktree);
    assert_eq!(runs[7].1, vec!["diff", "--cached", "--name-only"]);
    assert_eq!(runs[8].0, primary_worktree);
    assert_eq!(runs[8].1, vec!["merge", "--no-ff", "ct-1-deadbeef"]);
    assert_eq!(runs[9].0, primary_worktree);
    assert_eq!(runs[9].1[0], "worktree");
    assert_eq!(runs[9].1[1], "remove");
    assert_eq!(runs[9].1[2], worktree.to_string_lossy().to_string());
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

    let (_, args, envs, _) = adapter.last();
    assert_eq!(args, vec!["complete"]);
    assert!(envs.is_empty(), "no endpoint_env → no envs forwarded");
}
