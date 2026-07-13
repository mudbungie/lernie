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
    assert!(tree.in_flight.is_empty());
}

#[test]
fn from_repo_agent_branch_surfaces_with_steps_and_preview() {
    let fx = Fixture::new();
    fx.build_agent("20260422T120500Z-b002", "ping v03");
    let tree = GitTree::from_repo(&fx.path).unwrap();
    assert_eq!(tree.commits.len(), 1, "config lineage untouched");
    assert_eq!(tree.in_flight.len(), 1);
    let branch = &tree.in_flight[0];
    // The ref is `agents/<id>` (§2.3); the id is the identity.
    assert_eq!(branch.branch_name, "agents/20260422T120500Z-b002");
    assert_eq!(branch.conv_id, "20260422T120500Z-b002");
    assert_eq!(branch.preview.as_deref(), Some("ping v03"));
    // Dispatch commit + compaction merge past the config lineage.
    assert_eq!(branch.steps.len(), 2);
    assert_eq!(branch.tip_short_oid.len(), 8);
}

#[test]
fn from_repo_multiple_agents_appear() {
    let fx = Fixture::new();
    fx.build_agent("20260422T120000Z-old0", "first prompt");
    fx.build_agent("20260422T120500Z-new0", "second prompt");
    let tree = GitTree::from_repo(&fx.path).unwrap();
    assert_eq!(tree.commits.len(), 1);
    assert_eq!(tree.in_flight.len(), 2);
    let ids: Vec<&str> = tree.in_flight.iter().map(|b| b.conv_id.as_str()).collect();
    assert!(ids.contains(&"20260422T120000Z-old0"), "{ids:?}");
    assert!(ids.contains(&"20260422T120500Z-new0"), "{ids:?}");
}

#[test]
fn from_repo_agent_without_request_json_drops_preview() {
    let fx = Fixture::new();
    fx.build_agent("20260422T130000Z-xxxx", "seed");
    // Remove the step record: preview and streaming text derive from
    // disk (§2.3), so both go silent.
    std::fs::remove_dir_all(fx.path.join("steps/20260422T130000Z-xxxx")).unwrap();
    let tree = GitTree::from_repo(&fx.path).unwrap();
    assert_eq!(tree.in_flight.len(), 1);
    assert!(tree.in_flight[0].preview.is_none());
    assert!(tree.in_flight[0].streaming_text.is_none());
}

#[test]
fn from_repo_in_flight_surfaces_partial_response_text() {
    // bl-0619: live-streaming text view-model. The harness writes
    // `<workspace>/steps/<agent-id>/<NNN>/response.json` as JSONL of
    // §4.4 stream events while the model is producing output; the
    // frontend reads it on every tick and folds `text_delta` events
    // into the in-flight branch's `streaming_text`.
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
    assert_eq!(tree.in_flight.len(), 1);
    assert_eq!(
        tree.in_flight[0].streaming_text.as_deref(),
        Some("Rust tracks ownership")
    );
}

#[test]
fn from_repo_in_flight_picks_latest_step_response_text() {
    // Multi-step loop: step 001 has a complete response, step 002 is
    // mid-stream. Streaming text reflects the latest step only — earlier
    // steps' bodies surface through the per-step commit view, not the
    // live-streaming pane.
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
        tree.in_flight[0].streaming_text.as_deref(),
        Some("second step partial")
    );
}

#[test]
fn from_repo_in_flight_with_response_but_no_text_deltas_yet() {
    // Response file exists but only `message_start` has landed — no
    // text_delta events yet. The view-model should still be `None`
    // (nothing user-visible to render) and detection must not crash.
    let fx = Fixture::new();
    fx.build_agent("20260427T120300Z-prep", "still preparing");
    fx.write_response_events(
        "20260427T120300Z-prep",
        1,
        &[r#"{"type":"message_start","message":{"id":"m1"}}"#],
    );
    let tree = GitTree::from_repo(&fx.path).unwrap();
    assert!(tree.in_flight[0].streaming_text.is_none());
}

#[test]
fn from_repo_in_flight_surfaces_in_flight_and_complete_tool_calls() {
    // bl-23d9: pulsing tool indicators. Latest step's tools/<id>/ dir
    // with input.json + no output.json is in-flight; both files present
    // is complete. Detection is filesystem-only (ARCH §3.3, §3.5).
    let fx = Fixture::new();
    fx.build_agent("20260427T140000Z-tool", "run two tools");
    fx.write_tool_call("20260427T140000Z-tool", 1, "toolu_done", Some(b"{}"));
    fx.write_tool_call("20260427T140000Z-tool", 1, "toolu_live", None);
    let tree = GitTree::from_repo(&fx.path).unwrap();
    let calls = &tree.in_flight[0].tool_calls;
    assert_eq!(calls.len(), 2);
    // Sorted by tool_id: "toolu_done" < "toolu_live".
    assert_eq!(calls[0].tool_id, "toolu_done");
    assert_eq!(calls[0].state, ToolCallState::Complete);
    assert_eq!(calls[1].tool_id, "toolu_live");
    assert_eq!(calls[1].state, ToolCallState::InFlight);
}

#[test]
fn from_repo_in_flight_without_tool_calls_yields_empty_vec() {
    // No tools/ dir on disk — branch surfaces but tool_calls is empty.
    let fx = Fixture::new();
    fx.build_agent("20260427T140100Z-bare", "no tools yet");
    // Drop the tools-free step record's response so only the request
    // remains; tool detection walks the latest step dir.
    let tree = GitTree::from_repo(&fx.path).unwrap();
    assert!(tree.in_flight[0].tool_calls.is_empty());
}
