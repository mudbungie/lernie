//! egui widget: renders a [`GitTree`] as the §7.1 live view.
//!
//! The config lineage (trunk) comes first, then the agent tree — agents
//! nested by hyphenated descent ([`descent_order`], §2.3), each row
//! carrying its §3.5 state badge, the two ref-derived marks
//! (declined-transfer, budget-exhausted), a pending-message indicator, and
//! its branch commits (delivery / work-product-transfer commits legible by
//! subject). Thin wrapper — all structure lives in the view-model, so a
//! `lernie-ui-web` can drive the same [`descent_order`] over the same data.

use super::descent::descent_order;
use super::{Agent, AgentState, CommitNode, GitTree, StepCommit, ToolCall, ToolCallState};

/// Render the whole tree. Empty trunk *and* no agents → placeholder.
pub fn render(ui: &mut egui::Ui, tree: &GitTree) {
    if tree.commits.is_empty() && tree.agents.is_empty() {
        ui.label("(no commits yet)");
        return;
    }
    for commit in &tree.commits {
        render_commit(ui, commit);
    }
    if !tree.agents.is_empty() {
        ui.separator();
        ui.label("agents");
        for row in descent_order(&tree.agents) {
            render_agent(ui, row.depth, row.branch);
            for step in &row.branch.steps {
                render_step(ui, row.depth, step);
            }
        }
    }
}

fn render_commit(ui: &mut egui::Ui, commit: &CommitNode) {
    ui.horizontal(|ui| {
        ui.monospace(&commit.short_oid);
        ui.label(commit.timestamp_unix.to_string());
        ui.label(&commit.subject);
    });
}

/// Two spaces of indent per descent level; the tree shape reads off the
/// leading whitespace, no per-level widget nesting required.
fn indent(depth: usize) -> String {
    "  ".repeat(depth)
}

fn render_step(ui: &mut egui::Ui, depth: usize, step: &StepCommit) {
    ui.horizontal(|ui| {
        ui.monospace(format!("{}  ↳", indent(depth)));
        ui.monospace(&step.short_oid);
        ui.label(step.timestamp_unix.to_string());
        ui.label(&step.subject);
    });
}

fn render_agent(ui: &mut egui::Ui, depth: usize, agent: &Agent) {
    ui.horizontal(|ui| {
        let pad = indent(depth);
        if !pad.is_empty() {
            ui.monospace(pad);
        }
        let (glyph, color) = state_badge(agent.state);
        ui.colored_label(color, glyph);
        for (text, mark_color) in marks(agent) {
            ui.colored_label(mark_color, text);
        }
        if agent.pending_messages > 0 {
            ui.colored_label(PENDING_COLOR, format!("✉{}", agent.pending_messages));
        }
        ui.monospace(&agent.tip_short_oid);
        ui.label(agent.tip_timestamp_unix.to_string());
        ui.label(&agent.branch_name);
        if let Some(preview) = &agent.preview {
            ui.label(preview);
        }
    });
    if let Some(text) = &agent.streaming_text {
        ui.indent(("streaming", &agent.branch_name), |ui| {
            ui.label(text);
        });
    }
    for call in &agent.tool_calls {
        render_tool_call(ui, call);
    }
}

/// Colour for the pending-message indicator (§7.1) — a warm accent so a
/// non-empty inbox reads at a glance.
const PENDING_COLOR: egui::Color32 = egui::Color32::from_rgb(220, 200, 120);

/// The two orthogonal ref-derived marks (§2.6, §6), each rendered
/// alongside the state with the taxonomy's own label so the vocabulary is
/// unambiguous on screen.
fn marks(agent: &Agent) -> Vec<(&'static str, egui::Color32)> {
    let mut out = Vec::new();
    if agent.declined_transfer {
        out.push(("declined-transfer", egui::Color32::from_rgb(220, 120, 120)));
    }
    if agent.budget_exhausted {
        out.push(("budget-exhausted", egui::Color32::from_rgb(220, 160, 90)));
    }
    out
}

/// Repaint cadence for pulsing tool indicators. ~30 fps is smooth enough
/// for the eye and cheap enough for an idle UI.
const PULSE_REPAINT_DELAY: std::time::Duration = std::time::Duration::from_millis(33);

/// Pulse frequency in radians per second — ~0.6 Hz, slow enough to read as
/// "alive" rather than "blinking error".
const PULSE_RATE_RAD_PER_SEC: f64 = 4.0;

/// Glyph + colour for each §3.5 agent state. Pure function over the enum so
/// renderer tests can assert the mapping without a windowing context. The
/// four states are `live`, `in_flight`, `quiescent`, `stopped` (§3.5) —
/// each a distinct (glyph, colour) pair.
pub(crate) fn state_badge(state: AgentState) -> (&'static str, egui::Color32) {
    match state {
        // A driver holds the lock, not in a model call: solid + green.
        AgentState::Live => ("●", egui::Color32::from_rgb(120, 200, 120)),
        // A model call is streaming right now: half-filled + blue.
        AgentState::InFlight => ("◐", egui::Color32::from_rgb(120, 180, 220)),
        // Finished-for-now, awaiting a message: hollow + neutral amber.
        AgentState::Quiescent => ("○", egui::Color32::from_rgb(190, 180, 120)),
        // No live executor, no clean terminal: square + grey.
        AgentState::Stopped => ("■", egui::Color32::from_rgb(170, 170, 170)),
    }
}

fn render_tool_call(ui: &mut egui::Ui, call: &ToolCall) {
    ui.horizontal(|ui| {
        ui.label("    ⚙");
        match call.state {
            ToolCallState::InFlight => {
                let time = ui.ctx().input(|i| i.time);
                let alpha = (0.5 + 0.5 * (time * PULSE_RATE_RAD_PER_SEC).sin()).clamp(0.0, 1.0);
                let color = egui::Color32::from_white_alpha((alpha * 255.0) as u8);
                ui.colored_label(color, &call.tool_id);
                ui.label("(in-flight)");
                ui.ctx().request_repaint_after(PULSE_REPAINT_DELAY);
            }
            ToolCallState::Complete => {
                ui.label(&call.tool_id);
            }
        }
    });
}
