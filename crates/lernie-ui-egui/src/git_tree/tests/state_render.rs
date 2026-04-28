//! Renderer tests for branch-state badges (bl-de6b).
//!
//! Asserts the per-state glyph reaches the paint layer and that each of
//! the four [`BranchState`] variants maps to a distinct (glyph, colour)
//! pair, so a future renderer pass that surfaces `Merged` (on trunk
//! merge nodes) or `Conflicted` (once v0.4 subagent merges ship) can
//! rely on the mapping without code changes.

use super::render::rendered_text;
use crate::git_tree::{BranchState, ConversationBranch, GitTree, state_badge};

fn branch_in_state(state: BranchState) -> ConversationBranch {
    ConversationBranch {
        branch_name: "20260427T150000Z-state".into(),
        conv_id: "20260427T150000Z-state".into(),
        tip_oid: "1".repeat(40),
        tip_short_oid: "11111111".into(),
        tip_timestamp_unix: 9,
        steps: vec![],
        preview: None,
        streaming_text: None,
        tool_calls: Vec::new(),
        state,
    }
}

#[test]
fn render_in_flight_branch_paints_in_flight_badge() {
    let tree = GitTree {
        commits: vec![],
        in_flight: vec![branch_in_state(BranchState::InFlight)],
    };
    let painted = rendered_text(&tree);
    let (glyph, _) = state_badge(BranchState::InFlight);
    assert!(
        painted.contains(glyph),
        "in-flight badge glyph {glyph:?} not found in paint shapes; got:\n{painted}"
    );
}

#[test]
fn render_stopped_branch_paints_stopped_badge() {
    let tree = GitTree {
        commits: vec![],
        in_flight: vec![branch_in_state(BranchState::Stopped)],
    };
    let painted = rendered_text(&tree);
    let (glyph, _) = state_badge(BranchState::Stopped);
    assert!(
        painted.contains(glyph),
        "stopped badge glyph {glyph:?} not found in paint shapes; got:\n{painted}"
    );
}

#[test]
fn state_badges_are_distinct_per_state() {
    let states = [
        BranchState::Merged,
        BranchState::InFlight,
        BranchState::Stopped,
        BranchState::Conflicted,
    ];
    let mut seen_glyphs = std::collections::HashSet::new();
    let mut seen_colors = std::collections::HashSet::new();
    for s in states {
        let (glyph, color) = state_badge(s);
        assert!(
            seen_glyphs.insert(glyph),
            "duplicate glyph {glyph:?} for state {s:?}"
        );
        assert!(
            seen_colors.insert(color.to_array()),
            "duplicate colour for state {s:?}"
        );
    }
}
