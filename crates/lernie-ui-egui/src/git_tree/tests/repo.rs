//! `GitTree::from_repo` end-to-end tests: real git repos, real commits,
//! assertions on the resulting view-model.

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
fn from_repo_empty_tree_on_fresh_repo() {
    let fx = Fixture::new();
    let err = GitTree::from_repo(&fx.path).unwrap_err();
    assert!(matches!(err, GitTreeError::Git { .. }), "got {err:?}");
}

#[test]
fn from_repo_linear_history_with_v01_exchanges() {
    let fx = Fixture::new();
    fx.commit_v01_exchange("20260422T120000Z-aaaa", "first prompt");
    fx.commit_v01_exchange("20260422T120500Z-bbbb", "second prompt");
    let tree = GitTree::from_repo(&fx.path).unwrap();
    assert_eq!(tree.commits.len(), 2);
    assert_eq!(
        tree.commits[0].exchange_id.as_deref(),
        Some("20260422T120000Z-aaaa")
    );
    assert_eq!(tree.commits[0].preview.as_deref(), Some("first prompt"));
    assert!(tree.commits[0].steps.is_empty());
    assert_eq!(
        tree.commits[1].exchange_id.as_deref(),
        Some("20260422T120500Z-bbbb")
    );
    assert_eq!(tree.commits[1].preview.as_deref(), Some("second prompt"));
    assert_eq!(tree.commits[0].short_oid.len(), 8);
    assert!(tree.in_flight.is_empty());
}

#[test]
fn from_repo_commit_without_exchange_has_no_preview() {
    let fx = Fixture::new();
    fx.commit_other("README.md", "hi");
    let tree = GitTree::from_repo(&fx.path).unwrap();
    assert_eq!(tree.commits.len(), 1);
    assert!(tree.commits[0].exchange_id.is_none());
    assert!(tree.commits[0].preview.is_none());
    assert!(tree.commits[0].steps.is_empty());
    assert!(tree.in_flight.is_empty());
}

#[test]
fn from_repo_v01_malformed_exchange_keeps_id_but_drops_preview() {
    let fx = Fixture::new();
    fx.commit_malformed_v01_exchange("20260422T120000Z-cccc");
    let tree = GitTree::from_repo(&fx.path).unwrap();
    assert_eq!(
        tree.commits[0].exchange_id.as_deref(),
        Some("20260422T120000Z-cccc")
    );
    assert!(tree.commits[0].preview.is_none());
}

#[test]
fn from_repo_v01_exchange_json_without_user_message_drops_preview() {
    let fx = Fixture::new();
    let rel = "exchanges/20260422T120000Z-dddd.json";
    fs::write(fx.path.join(rel), r#"{"other":"thing"}"#).unwrap();
    run_git(&fx.path, &["add", rel]);
    run_git(&fx.path, &["commit", "-q", "-m", "ex"]);
    let tree = GitTree::from_repo(&fx.path).unwrap();
    assert!(tree.commits[0].preview.is_none());
    assert!(tree.commits[0].exchange_id.is_some());
}

#[test]
fn from_repo_v02_merged_exchange_surfaces_merge_with_steps() {
    let fx = Fixture::new();
    fx.commit_other("README.md", "initial");
    fx.commit_v02_merged_exchange("20260422T120000Z-a001", "hello v02");
    let tree = GitTree::from_repo(&fx.path).unwrap();
    assert_eq!(tree.commits.len(), 2);
    let merge = &tree.commits[1];
    assert_eq!(merge.exchange_id.as_deref(), Some("20260422T120000Z-a001"));
    assert_eq!(merge.preview.as_deref(), Some("hello v02"));
    assert_eq!(merge.steps.len(), 3);
    assert!(tree.in_flight.is_empty());
}

#[test]
fn from_repo_v02_in_flight_exchange_surfaces_branch_with_steps() {
    let fx = Fixture::new();
    fx.commit_other("README.md", "initial");
    fx.build_v02_in_flight("20260422T120500Z-b002", "ping v02");
    let tree = GitTree::from_repo(&fx.path).unwrap();
    assert_eq!(tree.commits.len(), 1);
    assert!(tree.commits[0].exchange_id.is_none());
    assert_eq!(tree.in_flight.len(), 1);
    let branch = &tree.in_flight[0];
    assert_eq!(branch.branch_name, "ex/20260422T120500Z-b002");
    assert_eq!(branch.exchange_id, "20260422T120500Z-b002");
    assert_eq!(branch.preview.as_deref(), Some("ping v02"));
    assert_eq!(branch.steps.len(), 3);
    assert_eq!(branch.tip_short_oid.len(), 8);
}

#[test]
fn from_repo_v02_mixed_with_v01_history() {
    let fx = Fixture::new();
    fx.commit_v01_exchange("20260422T110000Z-old0", "legacy prompt");
    fx.commit_v02_merged_exchange("20260422T120000Z-new0", "new prompt");
    fx.build_v02_in_flight("20260422T120500Z-wip0", "wip prompt");
    let tree = GitTree::from_repo(&fx.path).unwrap();
    assert_eq!(tree.commits.len(), 2);
    assert_eq!(
        tree.commits[0].exchange_id.as_deref(),
        Some("20260422T110000Z-old0")
    );
    assert!(tree.commits[0].steps.is_empty());
    assert_eq!(
        tree.commits[1].exchange_id.as_deref(),
        Some("20260422T120000Z-new0")
    );
    assert_eq!(tree.commits[1].steps.len(), 3);
    assert_eq!(tree.in_flight.len(), 1);
    assert_eq!(tree.in_flight[0].exchange_id, "20260422T120500Z-wip0");
}

#[test]
fn from_repo_v02_shape_on_non_merge_commit_has_no_steps() {
    // Edge case: a v0.2-shape path introduced by a plain single-parent
    // commit (not a `--no-ff` merge) — e.g. imported state or a hand-
    // edit. The exchange id is still recognized, preview is pulled,
    // but the steps list stays empty because there is no branch to
    // walk.
    let fx = Fixture::new();
    fx.commit_other("README.md", "initial");
    let step_dir = "exchanges/direct-commit-id/steps/001";
    fs::create_dir_all(fx.path.join(step_dir)).unwrap();
    let req = serde_json::json!({
        "messages": [{"role": "user", "content": "direct"}],
    });
    fs::write(
        fx.path.join(format!("{step_dir}/request.json")),
        serde_json::to_vec_pretty(&req).unwrap(),
    )
    .unwrap();
    run_git(&fx.path, &["add", &format!("{step_dir}/request.json")]);
    run_git(&fx.path, &["commit", "-q", "-m", "direct v02"]);
    let tree = GitTree::from_repo(&fx.path).unwrap();
    assert_eq!(tree.commits.len(), 2);
    let node = &tree.commits[1];
    assert_eq!(node.exchange_id.as_deref(), Some("direct-commit-id"));
    assert_eq!(node.preview.as_deref(), Some("direct"));
    assert!(node.steps.is_empty());
}

#[test]
fn from_repo_v02_in_flight_without_request_json_drops_preview() {
    let fx = Fixture::new();
    fx.commit_other("README.md", "initial");
    run_git(
        &fx.path,
        &["checkout", "-q", "-b", "ex/20260422T130000Z-xxxx", "main"],
    );
    fs::write(fx.path.join("note.txt"), "n").unwrap();
    run_git(&fx.path, &["add", "note.txt"]);
    run_git(&fx.path, &["commit", "-q", "-m", "note"]);
    run_git(&fx.path, &["checkout", "-q", "main"]);
    let tree = GitTree::from_repo(&fx.path).unwrap();
    assert_eq!(tree.in_flight.len(), 1);
    assert!(tree.in_flight[0].preview.is_none());
    assert_eq!(tree.in_flight[0].steps.len(), 1);
}
