//! Happy-path test: full orchestration with valid inputs, over the
//! brazen `bz` data plane (ARCH §4.4).
//!
//! Asserts the conversation branch is spawned, the dispatch commit lays
//! goal.md + soul.md, the diagnostic step record lands at the conv-repo
//! root outside the worktree (§2.2 / §2.3), the request is a typed
//! canonical request, and the response is brazen `v=1` NDJSON with a
//! terminal `end`.

use super::fixtures::*;
use super::stubs::STUB_SHA;
use crate::prompt::run;
use crate::prompt::step::StepMeta;
use std::ffi::OsStr;

#[test]
fn run_happy_path_writes_branch_worktree_and_two_commits() {
    let repo = scaffold_repo(VALID_PER_REPO_PROVIDERS_YAML, Some("system body"));
    let harness = scaffold_harness_root();
    let adapter = StubAdapter::happy(&happy_response_bytes());
    let git = StubGit::ok();
    let (clock, id) = (FixedClock::default(), FixedIdGen);
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
            &tool_executor,
            harness.path(),
        ),
    )
    .unwrap();
    assert_eq!(branch, "ct-1-deadbeef");

    let worktree = worktree_path(repo.path());
    let repo_git = crate::workspace::repo_git(repo.path());

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
    // The initial user message entered through the front door (§2.11):
    // deposited into the agent's own inbox, then delivered by the step-1
    // drain. Its `from:` / `deposited_at:` frontmatter travels with the
    // file and is model-visible by design (§2.11) — `deposited_at` is the
    // first `now_iso8601` tick (`iso-1`).
    assert_eq!(
        request["messages"][0]["content"][0]["text"],
        "---\nfrom: user\ndeposited_at: iso-1\n---\nhello"
    );
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
    // stub git's revision captures return the fixed stub sha.
    // The deposit's `deposited_at` consumed `iso-1`, so the step-1 model
    // call bookends at `iso-2` / `iso-3`.
    let meta: StepMeta =
        serde_json::from_slice(&std::fs::read(step_dir.join("meta.json")).unwrap()).unwrap();
    assert_eq!(meta.commit, STUB_SHA);
    assert_eq!(meta.started_at, "iso-2");
    assert_eq!(meta.ended_at, "iso-3");

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


    // Git sequence: 4 (control resolution from the config commit, §2.2:
    // config-head rev-parse + three `show` reads, all against repo.git)
    // + 1 (branch spawn off config/default) + 3 (dispatch commit:
    // control-file removal, add, commit — §2.3 step 2) + 1 (drain
    // stray-probe, §2.11) + 2 (user-message delivery commit, §2.11) + 1
    // (rev-parse) + 2 (model-output transcript entry add + commit) + 1
    // (terminal result-deposit rev-parse, §2.6). Merge-back is gone
    // (§2.6): the root branch persists on its own ref. The version guard
    // runs no git.
    let runs = git.runs.borrow();
    assert_eq!(runs.len(), 15);
    for (dest, _args) in &runs[0..5] {
        assert_eq!(dest, &repo_git, "control + spawn run against repo.git");
    }
    assert_eq!(
        runs[0].1,
        vec!["rev-parse", "--verify", "refs/heads/config/default"]
    );
    assert_eq!(
        runs[1].1,
        vec!["show", &format!("{STUB_SHA}:providers.yaml")]
    );
    assert_eq!(
        runs[2].1,
        vec!["show", &format!("{STUB_SHA}:workflow.yaml")]
    );
    assert_eq!(
        runs[3].1,
        vec!["show", &format!("{STUB_SHA}:souls/worker.md")]
    );
    let args4 = &runs[4].1;
    assert_eq!(
        args4[..4],
        ["worktree", "add", "-b", "agents/ct-1-deadbeef"]
    );
    assert_eq!(args4[4], worktree.to_string_lossy().to_string());
    assert_eq!(args4[5], "config/default");
    for (dest, _args) in &runs[5..15] {
        assert_eq!(dest, &worktree, "post-spawn git runs inside the worktree");
    }
    // Dispatch commit (§2.3 step 2): the config commit's control files
    // leave the agent's tree (§2.2), then goal + soul commit.
    assert_eq!(
        runs[5].1,
        vec![
            "rm",
            "-r",
            "-q",
            "--ignore-unmatch",
            "--",
            "manifest.yaml",
            "workflow.yaml",
            "providers.yaml",
            "version",
            "souls"
        ]
    );
    assert_eq!(runs[6].1, vec!["add", "goal.md", "soul.md"]);
    assert_eq!(runs[7].1[0], "commit");
    assert!(runs[7].1[2].contains("step 001: dispatch"));
    assert!(runs[7].1[2].contains("[ct-1-deadbeef]"));
    // The step-1 drain (§2.11 *Delivery*): a stray-recovery probe over
    // messages/ (clean here — no add/commit), then the initial user
    // message delivered from the inbox as the first transcript entry,
    // before step 1's read state is captured.
    assert_eq!(runs[8].1, vec!["status", "--porcelain", "--", "messages"]);
    assert_eq!(runs[9].1, vec!["add", "messages/001-user.md"]);
    assert!(runs[10].1[2].contains("transcript 001: user"));
    assert_eq!(runs[11].1, vec!["rev-parse", "HEAD"]);

    // The transcript writer commits the model-output entry (§2.3): the
    // sealed staging file is renamed to messages/002-<model-id>.json —
    // the origin token is the model that authored it (§2.3) — and
    // committed.
    assert_eq!(
        runs[12].1,
        vec!["add", "messages/002-claude-sonnet-4-7.json"]
    );
    assert_eq!(runs[13].1[0], "commit");
    assert!(runs[13].1[2].contains("transcript 002: claude-sonnet-4-7"));
    assert!(runs[13].1[2].contains("[ct-1-deadbeef]"));
    // The renamed entry is on disk in the worktree and holds the
    // canonical model-output blocks (the "hi there" text block).
    let entry = worktree.join("messages/002-claude-sonnet-4-7.json");
    let blocks: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&entry).unwrap()).unwrap();
    assert_eq!(blocks[0]["type"], "text");
    assert_eq!(blocks[0]["text"], "hi there");
    // The staging file left by rename — no debris under steps/.
    assert!(!step_dir.join("staging.json").exists());

    // The terminal result deposit (§2.6, §2.3 step 5) reads the branch
    // tip as its terminal ref (`rev-parse HEAD`); the deposit itself is a
    // structural no-op for a root (no parent inbox, §2.4), so it lands no
    // git op of its own and no merge-back follows.
    assert_eq!(runs[14].0, worktree);
    assert_eq!(runs[14].1, vec!["rev-parse", "HEAD"]);
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

    // Exactly one adapter call — the model call — against the override
    // binary; no `--version` guard call.
    let calls = adapter.observed.borrow().clone();
    assert_eq!(calls.len(), 1);
    assert_eq!(calls[0].0, OsStr::new("/opt/alt-bz"));
    assert_eq!(calls[0].1, vec!["--json", "--provider", "anthropic"]);
}
