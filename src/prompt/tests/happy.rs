//! Happy-path test: full orchestration with valid inputs, over the
//! brazen `bz` data plane (ARCH §4.4).
//!
//! Asserts the conversation branch is spawned, the dispatch commit lays
//! goal.md + soul.md, the diagnostic step record lands at the conv-repo
//! root outside the worktree (§2.2 / §2.3), the request is a typed
//! canonical request, and the response is brazen `v=1` NDJSON with a
//! terminal `end`.

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
    let (clock, id, dispatcher) = (FixedClock::default(), FixedIdGen, StubDispatcher::ok());
    let (sleeper, tool_executor) = (StubSleeper::default(), StubToolExecutor::ok());

    let branch = run(
        repo.path(),
        "hello",
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

    let worktree = repo.path().join("ct-1-deadbeef");
    let primary_worktree = repo.path().join(ROOT_WORKTREE);

    let goal = std::fs::read_to_string(worktree.join("goal.md")).unwrap();
    assert_eq!(goal, "hello");
    let soul = std::fs::read_to_string(worktree.join("soul.md")).unwrap();
    assert_eq!(soul, "system body");

    assert!(!worktree.join("steps").exists());
    let step_dir = repo.path().join("steps/ct-1-deadbeef/001");
    let request: serde_json::Value =
        serde_json::from_slice(&std::fs::read(step_dir.join("request.json")).unwrap()).unwrap();
    assert_eq!(request["model"], "claude-sonnet-4-7");
    // Goal is prepended to the soul and rides as a canonical
    // `Content::Text` in `system[0]` (§2.8, §4.4 typed request).
    assert_eq!(
        request["system"][0]["text"].as_str().unwrap(),
        "<goal>\nhello\n</goal>\n\nsystem body"
    );
    assert_eq!(request["messages"][0]["role"], "user");
    assert_eq!(request["messages"][0]["content"][0]["text"], "hello");
    assert_eq!(request["max_tokens"], 4096);
    // `stream` is not set by lernie — brazen's default governs (§4.4).
    // The typed request serializes an unset Option as JSON `null`.
    assert!(request["stream"].is_null());

    // response.json is brazen `v=1` NDJSON: first line message_start,
    // and the terminal line is `{"type":"end"}` (§4.4).
    let lines = parse_jsonl(&std::fs::read(step_dir.join("response.json")).unwrap());
    assert!(lines.len() >= 2, "expected event stream, got {lines:?}");
    assert_eq!(lines.first().unwrap()["type"], "message_start");
    let text = lines
        .iter()
        .find(|e| e["type"] == "content_delta")
        .expect("expected a content_delta");
    assert_eq!(text["delta"]["text_delta"], "hi there");
    assert_eq!(lines.last().unwrap()["type"], "end");
    let finish = lines.iter().find(|e| e["type"] == "finish").unwrap();
    assert_eq!(finish["reason"], "stop");

    // meta.json carries the branch-tip sha at step-start (§2.10). The
    // stub git's run_capture returns "", so `commit` is empty here.
    let meta: StepMeta =
        serde_json::from_slice(&std::fs::read(step_dir.join("meta.json")).unwrap()).unwrap();
    assert_eq!(meta.commit, "");
    assert_eq!(meta.started_at, "iso-1");
    assert_eq!(meta.ended_at, "iso-2");

    // Adapter called twice: the version guard (`bz --version`) then the
    // model call (`bz --json --provider anthropic`, request on stdin).
    let calls = adapter.observed.borrow().clone();
    assert_eq!(calls.len(), 2, "version guard + model call");
    let (binary, args, stdin) = calls[0].clone();
    assert_eq!(binary, OsStr::new("bz"));
    assert_eq!(args, vec!["--version"]);
    assert!(stdin.is_empty());
    let (binary, args, stdin) = calls[1].clone();
    assert_eq!(binary, OsStr::new("bz"));
    assert_eq!(args, vec!["--json", "--provider", "anthropic"]);
    // The model-call stdin matches request.json byte-for-byte in
    // content (pretty-print differs, so compare parsed).
    let wire: serde_json::Value = serde_json::from_slice(&stdin).unwrap();
    assert_eq!(wire, request);

    // Compactor dispatched via the CLI surface (§3.4).
    let dispatches = dispatcher.calls.borrow().clone();
    assert_eq!(dispatches.len(), 1);
    assert_eq!(dispatches[0].0, "compactor");
    assert_eq!(dispatches[0].1, repo.path());
    assert_eq!(dispatches[0].2, "ct-1-deadbeef");
    assert_eq!(dispatches[0].3, None);

    // Git sequence: 4 (conversation branch setup + rev-parse) + 6
    // (merge-back). The version guard runs no git.
    let runs = git.runs.borrow();
    assert_eq!(runs.len(), 10);
    let (dest0, args0) = &runs[0];
    assert_eq!(dest0, &primary_worktree);
    assert_eq!(args0[..4], ["worktree", "add", "-b", "ct-1-deadbeef"]);
    assert_eq!(args0[4], worktree.to_string_lossy().to_string());
    assert_eq!(args0[5], "main");
    for (dest, _args) in &runs[1..4] {
        assert_eq!(dest, &worktree, "post-spawn git runs inside conv worktree");
    }
    assert_eq!(runs[1].1, vec!["add", "goal.md", "soul.md"]);
    assert_eq!(runs[2].1[0], "commit");
    assert!(runs[2].1[2].contains("step 001: dispatch"));
    assert!(runs[2].1[2].contains("[ct-1-deadbeef]"));
    assert_eq!(runs[3].1, vec!["rev-parse", "HEAD"]);

    assert_eq!(runs[4].0, worktree);
    assert_eq!(runs[4].1, vec!["rebase", "main"]);
    assert_eq!(runs[5].1[0], "rm");
    assert_eq!(runs[6].1[0], "ls-tree");
    assert_eq!(runs[7].1, vec!["diff", "--cached", "--name-only"]);
    assert_eq!(runs[8].0, primary_worktree);
    assert_eq!(runs[8].1, vec!["merge", "--no-ff", "ct-1-deadbeef"]);
    assert_eq!(runs[9].0, primary_worktree);
    assert_eq!(runs[9].1[0], "worktree");
    assert_eq!(runs[9].1[1], "remove");
    assert_eq!(runs[9].1[2], worktree.to_string_lossy().to_string());
}

#[test]
fn run_under_adapter_override_skips_version_guard_and_uses_the_override() {
    // With an `adapter:` override in models.yaml the version guard is
    // skipped (§4.2); the MessageStart.v handshake governs. The stub
    // adapter is scripted with just the model stream — no `--version`.
    let repo = scaffold_repo(VALID_PER_REPO_PROVIDERS_YAML, Some("body"));
    let harness = scaffold_harness_root_with_adapter("/opt/alt-bz");
    let adapter = StubAdapter::scripted([StubAdapter::reply_ok(&happy_response_bytes())]);
    let git = StubGit::ok();
    let (clock, id, dispatcher) = (FixedClock::default(), FixedIdGen, StubDispatcher::ok());
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
            &dispatcher,
            &tool_executor,
            harness.path(),
        ),
    )
    .unwrap();

    // Exactly one adapter call — the model call — against the override
    // binary; no `--version` guard call.
    let calls = adapter.observed.borrow().clone();
    assert_eq!(calls.len(), 1);
    assert_eq!(calls[0].0, OsStr::new("/opt/alt-bz"));
    assert_eq!(calls[0].1, vec!["--json", "--provider", "anthropic"]);
}
