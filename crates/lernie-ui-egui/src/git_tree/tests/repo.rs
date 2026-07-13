//! `GitTree::from_repo` end-to-end tests: real workspaces (ARCH §2.2),
//! real commits, assertions on the resulting view-model.

use super::fixture::Fixture;
use crate::git_tree::{GitTree, GitTreeError, ToolCallState};
use tempfile::tempdir;

#[test]
fn from_repo_errors_when_repo_missing() {
    let dir = tempdir().unwrap();
    let err = match GitTree::from_repo(&dir.path().join("nope")) {
        Ok(_) => panic!("expected error"),
        Err(e) => e,
    };
    assert!(
        matches!(err, GitTreeError::Git { .. } | GitTreeError::Spawn(_)),
        "got {err:?}"
    );
}

#[test]
fn from_repo_errors_when_repo_git_absent() {
    // A directory that exists but has no `repo.git` (i.e. not a
    // workspace) should fail with a git error rather than silently
    // returning an empty tree.
    let dir = tempdir().unwrap();
    let err = GitTree::from_repo(dir.path()).unwrap_err();
    assert!(
        matches!(err, GitTreeError::Git { .. } | GitTreeError::Spawn(_)),
        "got {err:?}"
    );
}

#[test]
fn from_repo_surfaces_the_config_lineage_as_the_trunk() {
    let fx = Fixture::new();
    fx.commit_other("workflow.yaml", "events: {}\n");
    let tree = GitTree::from_repo(&fx.path).unwrap();
    // Init config commit + the amendment, oldest to newest (§2.2).
    assert_eq!(tree.commits.len(), 2);
    assert_eq!(tree.commits[0].subject, "config: init [config/default]");
    assert_eq!(tree.commits[1].subject, "add workflow.yaml");
    assert_eq!(tree.commits[0].short_oid.len(), 8);
    assert!(tree.agents.is_empty());
}

#[test]
fn from_repo_agent_surfaces_with_steps_and_preview() {
    let fx = Fixture::new();
    fx.build_agent("20260422T120500Z-b002", "ping v03");
    let tree = GitTree::from_repo(&fx.path).unwrap();
    assert_eq!(tree.commits.len(), 1, "config lineage untouched");
    assert_eq!(tree.agents.len(), 1);
    let agent = &tree.agents[0];
    // The ref is `agents/<id>` (§2.3); the id is the identity.
    assert_eq!(agent.branch_name, "agents/20260422T120500Z-b002");
    assert_eq!(agent.agent_id, "20260422T120500Z-b002");
    assert_eq!(agent.preview.as_deref(), Some("ping v03"));
    // Dispatch commit + compaction merge past the config lineage, each
    // carrying its subject (§7.1 "commits surfaced").
    assert_eq!(agent.steps.len(), 2);
    assert!(agent.steps[0].subject.contains("dispatch"));
    // The compaction is a `--no-ff` merge; `--first-parent` surfaces the
    // merge commit (its summary rides the second parent, §2.6).
    assert!(
        agent.steps[1].subject.contains("Merge"),
        "{:?}",
        agent.steps[1].subject
    );
    assert_eq!(agent.tip_short_oid.len(), 8);
    // No inbox and no mark refs by default.
    assert_eq!(agent.pending_messages, 0);
    assert!(!agent.declined_transfer);
    assert!(!agent.budget_exhausted);
}

#[test]
fn from_repo_multiple_agents_appear() {
    let fx = Fixture::new();
    fx.build_agent("20260422T120000Z-old0", "first prompt");
    fx.build_agent("20260422T120500Z-new0", "second prompt");
    let tree = GitTree::from_repo(&fx.path).unwrap();
    assert_eq!(tree.commits.len(), 1);
    assert_eq!(tree.agents.len(), 2);
    let ids: Vec<&str> = tree.agents.iter().map(|a| a.agent_id.as_str()).collect();
    assert!(ids.contains(&"20260422T120000Z-old0"), "{ids:?}");
    assert!(ids.contains(&"20260422T120500Z-new0"), "{ids:?}");
}

#[test]
fn from_repo_surfaces_pending_message_count() {
    // §7.1 pending-message indicator: files in the agent's inbox count;
    // the atomic-rename temp dotfile is excluded.
    let fx = Fixture::new();
    fx.build_agent("20260422T121000Z-msg0", "with mail");
    fx.deposit_message("20260422T121000Z-msg0", "user-001.md", "hi");
    fx.deposit_message("20260422T121000Z-msg0", "p1-002.md", "steer");
    fx.deposit_message("20260422T121000Z-msg0", ".user-003.md.tmp", "partial");
    let tree = GitTree::from_repo(&fx.path).unwrap();
    assert_eq!(tree.agents[0].pending_messages, 2);
}

#[test]
fn from_repo_surfaces_declined_transfer_and_budget_marks() {
    // §2.6 / §6 ref-derived marks, keyed by raw agent id.
    let fx = Fixture::new();
    fx.build_agent("20260422T121500Z-mrk0", "marked");
    fx.mark_ref("refs/lernie/conflicted/20260422T121500Z-mrk0");
    fx.mark_ref("refs/lernie/budget-exhausted/20260422T121500Z-mrk0");
    let tree = GitTree::from_repo(&fx.path).unwrap();
    assert!(tree.agents[0].declined_transfer);
    assert!(tree.agents[0].budget_exhausted);
}

#[test]
fn from_repo_marks_only_the_named_agent() {
    // A mark on one agent does not bleed onto a sibling.
    let fx = Fixture::new();
    fx.build_agent("20260422T122000Z-aaa0", "marked");
    fx.build_agent("20260422T122000Z-bbb0", "clean");
    fx.mark_ref("refs/lernie/budget-exhausted/20260422T122000Z-aaa0");
    let tree = GitTree::from_repo(&fx.path).unwrap();
    let marked = tree
        .agents
        .iter()
        .find(|a| a.agent_id == "20260422T122000Z-aaa0")
        .unwrap();
    let clean = tree
        .agents
        .iter()
        .find(|a| a.agent_id == "20260422T122000Z-bbb0")
        .unwrap();
    assert!(marked.budget_exhausted);
    assert!(!clean.budget_exhausted);
}

#[test]
fn from_repo_enumerates_a_child_agent_for_the_descent_tree() {
    // A child fork appears as its own agent row; the descent tree is
    // derived from the ids at render time (§2.3, §7.1).
    let fx = Fixture::new();
    fx.build_agent("20260422T123000Z-par0", "parent");
    fx.build_child("20260422T123000Z-par0", "20260422T123000Z-par0-c1");
    let tree = GitTree::from_repo(&fx.path).unwrap();
    let ids: Vec<&str> = tree.agents.iter().map(|a| a.agent_id.as_str()).collect();
    assert!(ids.contains(&"20260422T123000Z-par0"), "{ids:?}");
    assert!(ids.contains(&"20260422T123000Z-par0-c1"), "{ids:?}");
}

#[test]
fn from_repo_agent_without_request_json_drops_preview() {
    let fx = Fixture::new();
    fx.build_agent("20260422T130000Z-xxxx", "seed");
    // Remove the step record: preview and streaming text derive from
    // disk (§2.3), so both go silent.
    std::fs::remove_dir_all(fx.path.join("steps/20260422T130000Z-xxxx")).unwrap();
    let tree = GitTree::from_repo(&fx.path).unwrap();
    assert_eq!(tree.agents.len(), 1);
    assert!(tree.agents[0].preview.is_none());
    assert!(tree.agents[0].streaming_text.is_none());
}

#[test]
fn from_repo_surfaces_partial_response_text() {
    // Live-streaming text view-model. The harness writes
    // `<workspace>/steps/<agent-id>/<NNN>/response.json` as JSONL of §4.4
    // stream events while the model produces output; the frontend reads it
    // on every tick and folds `text_delta` events into `streaming_text`.
    let fx = Fixture::new();
    fx.build_agent("20260427T120100Z-strm", "summarize Rust ownership");
    fx.write_response_events(
        "20260427T120100Z-strm",
        1,
        &[
            r#"{"type":"message_start","message":{"id":"m1"}}"#,
            r#"{"type":"content_block_start","index":0,"content_block":{"type":"text","text":""}}"#,
            r#"{"type":"content_delta","index":0,"delta":{"text_delta":"Rust"}}"#,
            r#"{"type":"content_delta","index":0,"delta":{"text_delta":" tracks"}}"#,
            r#"{"type":"content_delta","index":0,"delta":{"text_delta":" ownership"}}"#,
        ],
    );
    let tree = GitTree::from_repo(&fx.path).unwrap();
    assert_eq!(tree.agents.len(), 1);
    assert_eq!(
        tree.agents[0].streaming_text.as_deref(),
        Some("Rust tracks ownership")
    );
}

#[test]
fn from_repo_picks_latest_step_response_text() {
    // Multi-step loop: step 001 has a complete response, step 002 is
    // mid-stream. Streaming text reflects the latest step only.
    let fx = Fixture::new();
    fx.build_agent("20260427T120200Z-loop", "step into the loop");
    fx.write_response_events(
        "20260427T120200Z-loop",
        1,
        &[r#"{"type":"content_delta","index":0,"delta":{"text_delta":"first step body"}}"#],
    );
    fx.write_response_events(
        "20260427T120200Z-loop",
        2,
        &[r#"{"type":"content_delta","index":0,"delta":{"text_delta":"second step partial"}}"#],
    );
    let tree = GitTree::from_repo(&fx.path).unwrap();
    assert_eq!(
        tree.agents[0].streaming_text.as_deref(),
        Some("second step partial")
    );
}

#[test]
fn from_repo_with_response_but_no_text_deltas_yet() {
    // Response file exists but only `message_start` has landed — no
    // text_delta events yet. The view-model should still be `None`.
    let fx = Fixture::new();
    fx.build_agent("20260427T120300Z-prep", "still preparing");
    fx.write_response_events(
        "20260427T120300Z-prep",
        1,
        &[r#"{"type":"message_start","message":{"id":"m1"}}"#],
    );
    let tree = GitTree::from_repo(&fx.path).unwrap();
    assert!(tree.agents[0].streaming_text.is_none());
}

#[test]
fn from_repo_surfaces_in_flight_and_complete_tool_calls() {
    // Pulsing tool indicators. Latest step's tools/<id>/ dir with
    // input.json + no output.json is in-flight; both files present is
    // complete. Detection is filesystem-only (ARCH §3.3, §3.5).
    let fx = Fixture::new();
    fx.build_agent("20260427T140000Z-tool", "run two tools");
    fx.write_tool_call("20260427T140000Z-tool", 1, "toolu_done", Some(b"{}"));
    fx.write_tool_call("20260427T140000Z-tool", 1, "toolu_live", None);
    let tree = GitTree::from_repo(&fx.path).unwrap();
    let calls = &tree.agents[0].tool_calls;
    assert_eq!(calls.len(), 2);
    // Sorted by tool_id: "toolu_done" < "toolu_live".
    assert_eq!(calls[0].tool_id, "toolu_done");
    assert_eq!(calls[0].state, ToolCallState::Complete);
    assert_eq!(calls[1].tool_id, "toolu_live");
    assert_eq!(calls[1].state, ToolCallState::InFlight);
}

#[test]
fn from_repo_without_tool_calls_yields_empty_vec() {
    // No tools/ dir on disk — agent surfaces but tool_calls is empty.
    let fx = Fixture::new();
    fx.build_agent("20260427T140100Z-bare", "no tools yet");
    let tree = GitTree::from_repo(&fx.path).unwrap();
    assert!(tree.agents[0].tool_calls.is_empty());
}
