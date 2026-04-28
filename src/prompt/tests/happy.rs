//! Happy-path test: full v0.3.1 orchestration with valid inputs.
//!
//! Asserts the conversation branch is spawned, the dispatch commit
//! lays goal.md + soul.md (only — no `request.json` in the
//! committed tree per amended §2.10), and the diagnostic step record
//! (request.json, response.json, meta.json) lands at the conv-repo
//! root outside the worktree (§2.2 / §2.3). v0.3.1 P3: response.json
//! is JSONL of §4.4 stream events, written event-by-event by the
//! harness as the adapter emits them; the assertions below pin that
//! shape. Compaction itself is exercised by the compactor module's
//! own tests; here we only assert the dispatcher was called with the
//! right repo + branch (ARCH §3.4).

use super::fixtures::*;
use crate::prompt::run;
use crate::prompt::step::StepMeta;
use crate::template::ROOT_WORKTREE;
use std::ffi::OsStr;

#[test]
fn run_happy_path_writes_branch_worktree_and_two_commits() {
    let repo = scaffold_repo(VALID_PER_REPO_PROVIDERS_YAML, Some("system body"));
    let harness = scaffold_harness_root();
    let adapter = StubAdapter::happy(&happy_response_bytes());
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
    // Streaming-on by default per v0.3.1 P3 (§4.4 always-on contract).
    assert_eq!(request["stream"], true);

    // response.json is JSONL of §4.4 stream events (one event per
    // `\n`-terminated line), appended by the harness as the adapter
    // emits them. Closing the write fd is the §3.5 IN_CLOSE_WRITE
    // completion signal — the file's terminal line is `message_stop`.
    let lines = parse_jsonl(&std::fs::read(step_dir.join("response.json")).unwrap());
    assert!(lines.len() >= 2, "expected JSONL stream, got {lines:?}");
    assert_eq!(lines.first().unwrap()["type"], "message_start");
    let text_delta = lines
        .iter()
        .find(|e| e["type"] == "text_delta")
        .expect("expected at least one text_delta");
    assert_eq!(text_delta["text"], "hi there");
    let stop = lines.last().unwrap();
    assert_eq!(stop["type"], "message_stop");
    assert_eq!(stop["stop_reason"], "end_turn");

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
    // dispatcher saw role=compactor, the repo, the conversation
    // branch, and no `--goal` (compactor uses built-in boilerplate).
    let dispatches = dispatcher.calls.borrow().clone();
    assert_eq!(dispatches.len(), 1);
    assert_eq!(dispatches[0].0, "compactor");
    assert_eq!(dispatches[0].1, repo.path());
    assert_eq!(dispatches[0].2, "ct-1-deadbeef");
    assert_eq!(dispatches[0].3, None);

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
    let stream = happy_response_bytes();
    let adapter = StubAdapter::scripted([
        StubAdapter::reply_ok(describe),
        StubAdapter::reply_ok(&stream),
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
