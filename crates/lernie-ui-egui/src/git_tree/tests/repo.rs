//! `GitTree::from_repo` end-to-end tests: real conv-repos (ARCH §2.2),
//! real commits, assertions on the resulting view-model.

use super::fixture::{Fixture, run_git};
use crate::git_tree::{GitTree, GitTreeError};
use std::fs;
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
fn from_repo_errors_when_root_worktree_absent() {
    // A directory that exists but has no `root/` subdir (i.e. not a
    // v0.3 conv-repo) should fail with a git error rather than silently
    // returning an empty tree.
    let dir = tempdir().unwrap();
    let err = GitTree::from_repo(dir.path()).unwrap_err();
    assert!(
        matches!(err, GitTreeError::Git { .. } | GitTreeError::Spawn(_)),
        "got {err:?}"
    );
}

#[test]
fn from_repo_empty_tree_on_fresh_repo() {
    let fx = Fixture::new();
    let err = GitTree::from_repo(&fx.path).unwrap_err();
    assert!(matches!(err, GitTreeError::Git { .. }), "got {err:?}");
}

#[test]
fn from_repo_commit_without_conversation_has_no_preview() {
    let fx = Fixture::new();
    fx.commit_other("README.md", "hi");
    let tree = GitTree::from_repo(&fx.path).unwrap();
    assert_eq!(tree.commits.len(), 1);
    assert!(tree.commits[0].conv_id.is_none());
    assert!(tree.commits[0].preview.is_none());
    assert!(tree.commits[0].steps.is_empty());
    assert!(tree.in_flight.is_empty());
}

#[test]
fn from_repo_v03_merged_conversation_surfaces_merge_with_steps() {
    // v0.3.1 conversation branch shape: dispatch commit + compactor
    // merge commit on the conv branch → 2 step commits. Step records
    // (request.json) sit on disk at <conv-repo>/steps/<conv-id>/001/,
    // outside every worktree, sourcing the preview.
    let fx = Fixture::new();
    fx.commit_other("README.md", "initial");
    fx.commit_v03_merged_conversation("20260422T120000Z-a001", "hello v03");
    let tree = GitTree::from_repo(&fx.path).unwrap();
    assert_eq!(tree.commits.len(), 2);
    let merge = &tree.commits[1];
    assert_eq!(merge.conv_id.as_deref(), Some("20260422T120000Z-a001"));
    assert_eq!(merge.preview.as_deref(), Some("hello v03"));
    assert_eq!(merge.steps.len(), 2);
    assert!(tree.in_flight.is_empty());
    assert_eq!(merge.short_oid.len(), 8);
}

#[test]
fn from_repo_v03_in_flight_conversation_surfaces_branch_with_steps() {
    let fx = Fixture::new();
    fx.commit_other("README.md", "initial");
    fx.build_v03_in_flight("20260422T120500Z-b002", "ping v03");
    let tree = GitTree::from_repo(&fx.path).unwrap();
    assert_eq!(tree.commits.len(), 1);
    assert!(tree.commits[0].conv_id.is_none());
    assert_eq!(tree.in_flight.len(), 1);
    let branch = &tree.in_flight[0];
    assert_eq!(branch.branch_name, "20260422T120500Z-b002");
    assert_eq!(branch.conv_id, "20260422T120500Z-b002");
    assert_eq!(branch.preview.as_deref(), Some("ping v03"));
    assert_eq!(branch.steps.len(), 2);
    assert_eq!(branch.tip_short_oid.len(), 8);
}

#[test]
fn from_repo_multiple_merged_conversations_appear_in_order() {
    let fx = Fixture::new();
    fx.commit_other("README.md", "initial");
    fx.commit_v03_merged_conversation("20260422T120000Z-old0", "first prompt");
    fx.commit_v03_merged_conversation("20260422T120500Z-new0", "second prompt");
    fx.build_v03_in_flight("20260422T121000Z-wip0", "wip prompt");
    let tree = GitTree::from_repo(&fx.path).unwrap();
    assert_eq!(tree.commits.len(), 3);
    assert!(tree.commits[0].conv_id.is_none());
    assert_eq!(
        tree.commits[1].conv_id.as_deref(),
        Some("20260422T120000Z-old0")
    );
    assert_eq!(tree.commits[1].steps.len(), 2);
    assert_eq!(
        tree.commits[2].conv_id.as_deref(),
        Some("20260422T120500Z-new0")
    );
    assert_eq!(tree.commits[2].steps.len(), 2);
    assert_eq!(tree.in_flight.len(), 1);
    assert_eq!(tree.in_flight[0].conv_id, "20260422T121000Z-wip0");
}

#[test]
fn from_repo_merge_with_non_conversation_subject_has_no_conv_id() {
    // Edge case: a `--no-ff` merge whose subject doesn't match
    // `Merge branch 'X'` (e.g. a hand-rewritten message) is treated as
    // a plain trunk commit, not a conversation. Branch-name detection
    // is the only signal post-bl-c22c (ARCH §2.3).
    let fx = Fixture::new();
    fx.commit_other("README.md", "initial");
    run_git(&fx.primary, &["checkout", "-q", "-b", "side", "main"]);
    fs::write(fx.primary.join("a.txt"), "x").unwrap();
    run_git(&fx.primary, &["add", "a.txt"]);
    run_git(&fx.primary, &["commit", "-q", "-m", "side work"]);
    run_git(&fx.primary, &["checkout", "-q", "main"]);
    run_git(
        &fx.primary,
        &["merge", "--no-ff", "-q", "-m", "manual merge", "side"],
    );
    let tree = GitTree::from_repo(&fx.path).unwrap();
    assert_eq!(tree.commits.len(), 2);
    let merge = &tree.commits[1];
    assert!(merge.conv_id.is_none());
    assert!(merge.preview.is_none());
    assert!(merge.steps.is_empty());
}

#[test]
fn from_repo_v03_in_flight_without_request_json_drops_preview() {
    let fx = Fixture::new();
    fx.commit_other("README.md", "initial");
    run_git(
        &fx.primary,
        &["checkout", "-q", "-b", "20260422T130000Z-xxxx", "main"],
    );
    fs::write(fx.primary.join("note.txt"), "n").unwrap();
    run_git(&fx.primary, &["add", "note.txt"]);
    run_git(&fx.primary, &["commit", "-q", "-m", "note"]);
    run_git(&fx.primary, &["checkout", "-q", "main"]);
    let tree = GitTree::from_repo(&fx.path).unwrap();
    assert_eq!(tree.in_flight.len(), 1);
    assert!(tree.in_flight[0].preview.is_none());
    assert!(tree.in_flight[0].streaming_text.is_none());
    assert_eq!(tree.in_flight[0].steps.len(), 1);
}

#[test]
fn from_repo_in_flight_surfaces_partial_response_text() {
    // bl-0619: live-streaming text view-model. The harness writes
    // `<conv-repo>/steps/<conv-id>/<NNN>/response.json` as JSONL of
    // §4.4 stream events while the model is producing output; the
    // frontend reads it on every tick and folds `text_delta` events
    // into the in-flight branch's `streaming_text`.
    let fx = Fixture::new();
    fx.commit_other("README.md", "initial");
    fx.build_v03_in_flight("20260427T120100Z-strm", "summarize Rust ownership");
    fx.write_response_events(
        "20260427T120100Z-strm",
        1,
        &[
            r#"{"type":"message_start","message":{"id":"m1"}}"#,
            r#"{"type":"content_block_start","index":0,"content_block":{"type":"text","text":""}}"#,
            r#"{"type":"text_delta","index":0,"text":"Rust"}"#,
            r#"{"type":"text_delta","index":0,"text":" tracks"}"#,
            r#"{"type":"text_delta","index":0,"text":" ownership"}"#,
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
    fx.commit_other("README.md", "initial");
    fx.build_v03_in_flight("20260427T120200Z-loop", "step into the loop");
    fx.write_response_events(
        "20260427T120200Z-loop",
        1,
        &[r#"{"type":"text_delta","index":0,"text":"first step body"}"#],
    );
    fx.write_response_events(
        "20260427T120200Z-loop",
        2,
        &[r#"{"type":"text_delta","index":0,"text":"second step partial"}"#],
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
    fx.commit_other("README.md", "initial");
    fx.build_v03_in_flight("20260427T120300Z-prep", "still preparing");
    fx.write_response_events(
        "20260427T120300Z-prep",
        1,
        &[r#"{"type":"message_start","message":{"id":"m1"}}"#],
    );
    let tree = GitTree::from_repo(&fx.path).unwrap();
    assert!(tree.in_flight[0].streaming_text.is_none());
}

#[test]
fn from_repo_v03_merged_conversation_has_no_streaming_text_field_set() {
    // Streaming text is an in-flight-only concern; merged commits
    // surface their text through the per-step commit view, not via the
    // streaming pane. We assert the in-flight list is empty so there's
    // no place for streaming text to land.
    let fx = Fixture::new();
    fx.commit_other("README.md", "initial");
    fx.commit_v03_merged_conversation("20260427T120400Z-done", "merged work");
    fx.write_response_events(
        "20260427T120400Z-done",
        1,
        &[r#"{"type":"text_delta","index":0,"text":"after-merge text"}"#],
    );
    let tree = GitTree::from_repo(&fx.path).unwrap();
    assert!(tree.in_flight.is_empty());
}
