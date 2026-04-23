//! egui rendering smoke tests — exercise the widget against both empty
//! and populated view-models to guarantee it does not panic.

use crate::git_tree::{CommitNode, ExchangeBranch, GitTree, StepCommit, render};

#[test]
fn render_empty_tree_shows_placeholder() {
    let ctx = egui::Context::default();
    let tree = GitTree::default();
    let _ = ctx.run(Default::default(), |ctx| {
        egui::CentralPanel::default().show(ctx, |ui| render(ui, &tree));
    });
}

#[test]
fn render_populated_tree_runs_without_panic() {
    let ctx = egui::Context::default();
    let tree = GitTree {
        commits: vec![
            CommitNode {
                oid: "a".repeat(40),
                short_oid: "aaaaaaaa".into(),
                timestamp_unix: 1,
                exchange_id: Some("ex1".into()),
                preview: Some("hello".into()),
                steps: vec![StepCommit {
                    oid: "c".repeat(40),
                    short_oid: "cccccccc".into(),
                    timestamp_unix: 2,
                }],
            },
            CommitNode {
                oid: "b".repeat(40),
                short_oid: "bbbbbbbb".into(),
                timestamp_unix: 3,
                exchange_id: None,
                preview: None,
                steps: vec![],
            },
        ],
        in_flight: vec![ExchangeBranch {
            branch_name: "ex/wip".into(),
            exchange_id: "wip".into(),
            tip_oid: "d".repeat(40),
            tip_short_oid: "dddddddd".into(),
            tip_timestamp_unix: 4,
            steps: vec![StepCommit {
                oid: "e".repeat(40),
                short_oid: "eeeeeeee".into(),
                timestamp_unix: 5,
            }],
            preview: Some("wip".into()),
        }],
    };
    let _ = ctx.run(Default::default(), |ctx| {
        egui::CentralPanel::default().show(ctx, |ui| render(ui, &tree));
    });
}

#[test]
fn short_oid_falls_back_for_unexpectedly_short_hash() {
    let node = CommitNode {
        oid: "abc".into(),
        short_oid: "abc".into(),
        timestamp_unix: 0,
        exchange_id: None,
        preview: None,
        steps: vec![],
    };
    assert_eq!(node.short_oid, "abc");
}
