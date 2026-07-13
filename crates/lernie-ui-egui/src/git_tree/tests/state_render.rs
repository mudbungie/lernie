//! Renderer tests for §3.5 agent-state badges.
//!
//! Asserts the per-state glyph reaches the paint layer and that each of the
//! four [`AgentState`] variants (`live`, `in_flight`, `quiescent`,
//! `stopped`) maps to a distinct (glyph, colour) pair, plus that the two
//! orthogonal ref-derived marks render alongside.

use super::render::rendered_text;
use crate::git_tree::{Agent, AgentState, GitTree, state_badge};

fn agent_in_state(state: AgentState) -> Agent {
    Agent {
        branch_name: "agents/20260427T150000Z-state".into(),
        agent_id: "20260427T150000Z-state".into(),
        tip_oid: "1".repeat(40),
        tip_short_oid: "11111111".into(),
        tip_timestamp_unix: 9,
        steps: vec![],
        preview: None,
        streaming_text: None,
        tool_calls: Vec::new(),
        state,
        pending_messages: 0,
        declined_transfer: false,
        budget_exhausted: false,
    }
}

fn tree_with(agent: Agent) -> GitTree {
    GitTree {
        commits: vec![],
        agents: vec![agent],
    }
}

#[test]
fn render_live_agent_paints_live_badge() {
    let painted = rendered_text(&tree_with(agent_in_state(AgentState::Live)));
    let (glyph, _) = state_badge(AgentState::Live);
    assert!(painted.contains(glyph), "got:\n{painted}");
}

#[test]
fn render_in_flight_agent_paints_in_flight_badge() {
    let painted = rendered_text(&tree_with(agent_in_state(AgentState::InFlight)));
    let (glyph, _) = state_badge(AgentState::InFlight);
    assert!(painted.contains(glyph), "got:\n{painted}");
}

#[test]
fn render_quiescent_agent_paints_quiescent_badge() {
    let painted = rendered_text(&tree_with(agent_in_state(AgentState::Quiescent)));
    let (glyph, _) = state_badge(AgentState::Quiescent);
    assert!(painted.contains(glyph), "got:\n{painted}");
}

#[test]
fn render_stopped_agent_paints_stopped_badge() {
    let painted = rendered_text(&tree_with(agent_in_state(AgentState::Stopped)));
    let (glyph, _) = state_badge(AgentState::Stopped);
    assert!(painted.contains(glyph), "got:\n{painted}");
}

#[test]
fn state_badges_are_distinct_per_state() {
    let states = [
        AgentState::Live,
        AgentState::InFlight,
        AgentState::Quiescent,
        AgentState::Stopped,
    ];
    let mut seen_glyphs = std::collections::HashSet::new();
    let mut seen_colors = std::collections::HashSet::new();
    for s in states {
        let (glyph, color) = state_badge(s);
        assert!(
            seen_glyphs.insert(glyph),
            "duplicate glyph {glyph:?} for {s:?}"
        );
        assert!(
            seen_colors.insert(color.to_array()),
            "duplicate colour for {s:?}"
        );
    }
}

#[test]
fn declined_transfer_and_budget_marks_render_alongside() {
    let mut agent = agent_in_state(AgentState::Stopped);
    agent.declined_transfer = true;
    agent.budget_exhausted = true;
    let painted = rendered_text(&tree_with(agent));
    assert!(painted.contains("declined-transfer"), "got:\n{painted}");
    assert!(painted.contains("budget-exhausted"), "got:\n{painted}");
}

#[test]
fn pending_message_indicator_renders_count() {
    let mut agent = agent_in_state(AgentState::Quiescent);
    agent.pending_messages = 3;
    let painted = rendered_text(&tree_with(agent));
    assert!(painted.contains("✉3"), "got:\n{painted}");
}
