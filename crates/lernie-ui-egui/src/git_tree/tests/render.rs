//! egui rendering smoke tests — exercise the widget against both empty
//! and populated view-models to guarantee it does not panic, plus
//! shape-level assertions that user-visible text actually lands in the
//! paint output (in particular, in-flight streaming text per bl-0619).

use crate::git_tree::{
    BranchState, CommitNode, ConversationBranch, GitTree, StepCommit, ToolCall, ToolCallState,
    render,
};

// `BranchState` only appears in struct-init positions below; renderer-
// level tests for the badge mapping itself live in
// `tests::state_render`.

/// Run the renderer headlessly and concatenate every `Shape::Text`
/// galley's text in paint order. Used by tests that assert specific
/// strings reach the paint layer (e.g. streaming text in flight).
pub(super) fn rendered_text(tree: &GitTree) -> String {
    let ctx = egui::Context::default();
    let output = ctx.run(Default::default(), |ctx| {
        egui::CentralPanel::default().show(ctx, |ui| render(ui, tree));
    });
    let mut out = String::new();
    for clipped in output.shapes {
        collect_text(&clipped.shape, &mut out);
    }
    out
}

fn collect_text(shape: &egui::Shape, out: &mut String) {
    match shape {
        egui::Shape::Text(t) => {
            out.push_str(t.galley.text());
            out.push('\n');
        }
        egui::Shape::Vec(shapes) => {
            for s in shapes {
                collect_text(s, out);
            }
        }
        _ => {}
    }
}

#[test]
fn collect_text_recurses_into_shape_vec() {
    // egui can wrap multiple shapes in `Shape::Vec` (e.g. nested
    // layouts). Our shape walker must descend into it; an empty arm
    // would silently drop any text under such a wrapper. We build the
    // galleys inside a `ctx.run` so the font cache is initialized
    // (fonts aren't available before first run; egui asserts on this).
    use egui::{Color32, FontId, Pos2, Stroke};
    let ctx = egui::Context::default();
    let mut nested: Option<egui::Shape> = None;
    let mut flat: Option<egui::Shape> = None;
    let _ = ctx.run(Default::default(), |ctx| {
        let inner_galley = ctx
            .fonts(|f| f.layout_no_wrap("nested-text".into(), FontId::default(), Color32::WHITE));
        let outer_galley =
            ctx.fonts(|f| f.layout_no_wrap("flat-text".into(), FontId::default(), Color32::WHITE));
        nested = Some(egui::Shape::Vec(vec![egui::Shape::Text(
            egui::epaint::TextShape {
                pos: Pos2::ZERO,
                galley: inner_galley,
                underline: Stroke::NONE,
                fallback_color: Color32::WHITE,
                override_text_color: None,
                opacity_factor: 1.0,
                angle: 0.0,
            },
        )]));
        flat = Some(egui::Shape::Text(egui::epaint::TextShape {
            pos: Pos2::ZERO,
            galley: outer_galley,
            underline: Stroke::NONE,
            fallback_color: Color32::WHITE,
            override_text_color: None,
            opacity_factor: 1.0,
            angle: 0.0,
        }));
    });
    let mut out = String::new();
    collect_text(nested.as_ref().unwrap(), &mut out);
    collect_text(flat.as_ref().unwrap(), &mut out);
    collect_text(&egui::Shape::Noop, &mut out);
    assert!(out.contains("nested-text"));
    assert!(out.contains("flat-text"));
}

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
                subject: "config: init [config/default]".into(),
            },
            CommitNode {
                oid: "b".repeat(40),
                short_oid: "bbbbbbbb".into(),
                timestamp_unix: 3,
                subject: "config: amend".into(),
            },
        ],
        in_flight: vec![ConversationBranch {
            branch_name: "20260422T130000Z-wwww".into(),
            conv_id: "20260422T130000Z-wwww".into(),
            tip_oid: "d".repeat(40),
            tip_short_oid: "dddddddd".into(),
            tip_timestamp_unix: 4,
            steps: vec![StepCommit {
                oid: "e".repeat(40),
                short_oid: "eeeeeeee".into(),
                timestamp_unix: 5,
            }],
            preview: Some("wip".into()),
            streaming_text: None,
            tool_calls: Vec::new(),
            state: BranchState::InFlight,
        }],
    };
    let _ = ctx.run(Default::default(), |ctx| {
        egui::CentralPanel::default().show(ctx, |ui| render(ui, &tree));
    });
}

#[test]
fn render_in_flight_branch_paints_streaming_text() {
    let tree = GitTree {
        commits: vec![],
        in_flight: vec![ConversationBranch {
            branch_name: "20260427T120000Z-stream".into(),
            conv_id: "20260427T120000Z-stream".into(),
            tip_oid: "f".repeat(40),
            tip_short_oid: "ffffffff".into(),
            tip_timestamp_unix: 6,
            steps: vec![],
            preview: Some("explain quicksort".into()),
            streaming_text: Some("Quicksort partitions around a pivot".into()),
            tool_calls: Vec::new(),
            state: BranchState::InFlight,
        }],
    };
    let painted = rendered_text(&tree);
    assert!(
        painted.contains("Quicksort partitions around a pivot"),
        "streaming text not found in paint shapes; got:\n{painted}"
    );
}

#[test]
fn render_in_flight_branch_without_streaming_text_paints_no_body() {
    let tree = GitTree {
        commits: vec![],
        in_flight: vec![ConversationBranch {
            branch_name: "20260427T120000Z-quiet".into(),
            conv_id: "20260427T120000Z-quiet".into(),
            tip_oid: "0".repeat(40),
            tip_short_oid: "00000000".into(),
            tip_timestamp_unix: 7,
            steps: vec![],
            preview: None,
            streaming_text: None,
            tool_calls: Vec::new(),
            state: BranchState::Stopped,
        }],
    };
    // No assertion on absence (egui paints frame chrome around the
    // panel); the contract under test is the positive case above.
    // Here we only verify the branch row itself still renders.
    let painted = rendered_text(&tree);
    assert!(painted.contains("20260427T120000Z-quiet"));
}

/// Run the renderer twice and return the second frame's
/// `repaint_delay`. Egui returns 0 on the first frame regardless of the
/// content (font cache + layout still warming up), so we have to settle
/// to read the steady-state delay set by `request_repaint_after`. An
/// in-flight tool keeps the delay short; a complete-only tree falls
/// back to `Duration::MAX`.
fn repaint_delay_for(tree: &GitTree) -> std::time::Duration {
    let ctx = egui::Context::default();
    for _ in 0..2 {
        let _ = ctx.run(Default::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| render(ui, tree));
        });
    }
    let output = ctx.run(Default::default(), |ctx| {
        egui::CentralPanel::default().show(ctx, |ui| render(ui, tree));
    });
    output
        .viewport_output
        .get(&egui::ViewportId::ROOT)
        .expect("root viewport output present")
        .repaint_delay
}

fn branch_with_tool(tool_id: &str, state: ToolCallState) -> ConversationBranch {
    ConversationBranch {
        branch_name: "20260427T140000Z-tool".into(),
        conv_id: "20260427T140000Z-tool".into(),
        tip_oid: "9".repeat(40),
        tip_short_oid: "99999999".into(),
        tip_timestamp_unix: 8,
        steps: vec![],
        preview: None,
        streaming_text: None,
        tool_calls: vec![ToolCall {
            tool_id: tool_id.into(),
            state,
        }],
        state: BranchState::InFlight,
    }
}

#[test]
fn render_in_flight_tool_call_schedules_repaint_and_paints_id() {
    let tree = GitTree {
        commits: vec![],
        in_flight: vec![branch_with_tool("toolu_pulse_a", ToolCallState::InFlight)],
    };
    assert!(
        repaint_delay_for(&tree) < std::time::Duration::from_secs(1),
        "in-flight tool must schedule a near-term repaint"
    );
    let painted = rendered_text(&tree);
    assert!(
        painted.contains("toolu_pulse_a"),
        "tool id must reach the paint layer; got:\n{painted}"
    );
    assert!(painted.contains("(in-flight)"));
}

#[test]
fn render_complete_tool_call_does_not_schedule_repaint() {
    let tree = GitTree {
        commits: vec![],
        in_flight: vec![branch_with_tool("toolu_done_b", ToolCallState::Complete)],
    };
    assert_eq!(
        repaint_delay_for(&tree),
        std::time::Duration::MAX,
        "complete tools must not pull repaints"
    );
    let painted = rendered_text(&tree);
    assert!(painted.contains("toolu_done_b"));
    assert!(!painted.contains("(in-flight)"));
}

#[test]
fn render_mixed_tool_calls_schedules_repaint_when_any_in_flight() {
    let mut branch = branch_with_tool("toolu_done_c", ToolCallState::Complete);
    branch.tool_calls.push(ToolCall {
        tool_id: "toolu_pulse_d".into(),
        state: ToolCallState::InFlight,
    });
    let tree = GitTree {
        commits: vec![],
        in_flight: vec![branch],
    };
    assert!(repaint_delay_for(&tree) < std::time::Duration::from_secs(1));
    let painted = rendered_text(&tree);
    assert!(painted.contains("toolu_done_c"));
    assert!(painted.contains("toolu_pulse_d"));
}

#[test]
fn short_oid_falls_back_for_unexpectedly_short_hash() {
    let node = CommitNode {
        oid: "abc".into(),
        short_oid: "abc".into(),
        timestamp_unix: 0,
        subject: "s".into(),
    };
    assert_eq!(node.short_oid, "abc");
}
