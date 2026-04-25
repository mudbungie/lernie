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
    let fx = Fixture::new();
    fx.commit_other("README.md", "initial");
    fx.commit_v03_merged_conversation("20260422T120000Z-a001", "hello v03");
    let tree = GitTree::from_repo(&fx.path).unwrap();
    assert_eq!(tree.commits.len(), 2);
    let merge = &tree.commits[1];
    assert_eq!(merge.conv_id.as_deref(), Some("20260422T120000Z-a001"));
    assert_eq!(merge.preview.as_deref(), Some("hello v03"));
    assert_eq!(merge.steps.len(), 3);
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
    assert_eq!(branch.steps.len(), 3);
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
    assert_eq!(tree.commits[1].steps.len(), 3);
    assert_eq!(
        tree.commits[2].conv_id.as_deref(),
        Some("20260422T120500Z-new0")
    );
    assert_eq!(tree.commits[2].steps.len(), 3);
    assert_eq!(tree.in_flight.len(), 1);
    assert_eq!(tree.in_flight[0].conv_id, "20260422T121000Z-wip0");
}

#[test]
fn from_repo_v03_shape_on_non_merge_commit_has_no_steps() {
    // Edge case: a v0.3-shape path introduced by a plain single-parent
    // commit (not a `--no-ff` merge) — e.g. imported state or a hand-
    // edit. The conv id is still recognized, preview is pulled, but
    // the steps list stays empty because there is no branch to walk.
    let fx = Fixture::new();
    fx.commit_other("README.md", "initial");
    let step_dir = "steps/direct-commit-id/001";
    fs::create_dir_all(fx.primary.join(step_dir)).unwrap();
    let req = serde_json::json!({
        "messages": [{"role": "user", "content": "direct"}],
    });
    fs::write(
        fx.primary.join(format!("{step_dir}/request.json")),
        serde_json::to_vec_pretty(&req).unwrap(),
    )
    .unwrap();
    run_git(&fx.primary, &["add", &format!("{step_dir}/request.json")]);
    run_git(&fx.primary, &["commit", "-q", "-m", "direct v03"]);
    let tree = GitTree::from_repo(&fx.path).unwrap();
    assert_eq!(tree.commits.len(), 2);
    let node = &tree.commits[1];
    assert_eq!(node.conv_id.as_deref(), Some("direct-commit-id"));
    assert_eq!(node.preview.as_deref(), Some("direct"));
    assert!(node.steps.is_empty());
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
    assert_eq!(tree.in_flight[0].steps.len(), 1);
}
