//! Git-tree view-model and egui widget.
//!
//! [`GitTree::from_repo`] inspects the repo's current git state and
//! produces a view-model suitable for rendering. The view-model is a
//! pure function of the repo's refs and tree content; it holds no egui
//! dependency, so a future `lernie-ui-web` crate can render the same
//! structure from the web.
//!
//! Git access is via the `git` CLI (a hard dep of lernie itself, per
//! ARCH §2.2) — no libgit2 native build step is required.
//!
//! # Workspace layout (ARCH §2.2–§2.3)
//!
//! A workspace holds one bare repository at `<workspace>/repo.git`:
//! config branches (`config/<name>`) and agent refs
//! (`agents/<agent-id>`) — no `main`. Callers pass the workspace path;
//! this module resolves the git dir to `<workspace>/repo.git` before
//! issuing any git command, and reads step records (for previews)
//! directly from the workspace root's `steps/` tree.
//!
//! The trunk section of the view is the config lineage (`HEAD`, which
//! the workspace repository points at `config/default`); the agent
//! section enumerates every `agents/*` ref — agents never merge
//! anywhere (§2.6), so every agent is a live row. The user-message
//! preview reads from `<workspace>/steps/<agent-id>/001/request.json`
//! on disk.

mod cmd;
mod detect;
mod enumerate;
mod fd_probe;
mod state;
mod streaming;
mod tools;

pub use state::BranchState;

use std::path::{Path, PathBuf};

/// The bare workspace repository dir (ARCH §2.2). Mirrors
/// `src/workspace::REPO_DIR` in the harness; the duplicate constant
/// keeps the UI crate free of a dep on the harness binary.
const REPO_DIR: &str = "repo.git";

/// Top-level directory under the conv-repo holding per-conversation
/// step records (ARCH §2.2 / §2.3). Mirrors
/// `src/prompt/step::STEPS_DIR`; the duplicate constant keeps the UI
/// crate free of a dep on the harness binary.
const STEPS_DIR: &str = "steps";

#[derive(Debug, thiserror::Error)]
pub enum GitTreeError {
    #[error("git invocation failed: {0}")]
    Spawn(#[from] std::io::Error),
    #[error("git {command} in {repo:?} failed: {stderr}")]
    Git {
        command: String,
        repo: PathBuf,
        stderr: String,
    },
    #[error("malformed git log line: {0:?}")]
    LogFormat(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct GitTree {
    /// The config lineage (`HEAD` → `config/default`, §2.2),
    /// first-parent, oldest to newest.
    pub commits: Vec<CommitNode>,
    /// Every agent branch (`agents/*`, §2.3), enumerated via
    /// `git for-each-ref refs/heads/agents/`. Agents never merge
    /// anywhere (§2.6); each persists on its own ref.
    pub in_flight: Vec<ConversationBranch>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommitNode {
    pub oid: String,
    pub short_oid: String,
    pub timestamp_unix: i64,
    /// Commit subject — config commits are the only trunk commits
    /// (§2.2–§2.3: agents never merge anywhere), so the subject is the
    /// row's label.
    pub subject: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StepCommit {
    pub oid: String,
    pub short_oid: String,
    pub timestamp_unix: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConversationBranch {
    /// Branch name; equal to `conv_id` in v0.3 (no prefix). Held
    /// separately so the renderer can label rows without re-deriving.
    pub branch_name: String,
    pub conv_id: String,
    pub tip_oid: String,
    pub tip_short_oid: String,
    pub tip_timestamp_unix: i64,
    /// Commits on this branch not reachable from `main`, oldest to
    /// newest. Same shape as merged-conversation steps.
    pub steps: Vec<StepCommit>,
    pub preview: Option<String>,
    /// Live-updating model text for the latest in-flight step on this
    /// branch. Re-derived from
    /// `<conv-repo>/steps/<conv-id>/<NNN>/response.json` on every
    /// `from_repo` call (ARCH §3.5: stateless re-read on each tick).
    /// `None` when no `text_delta` events have landed yet (or the
    /// step's `response.json` is absent).
    pub streaming_text: Option<String>,
    /// Tool calls under this branch's latest step's `tools/` directory
    /// (ARCH §3.3). State is derived purely from the presence of
    /// `input.json` / `output.json`: input only ⇒ in-flight, both ⇒
    /// complete. The renderer pulses in-flight nodes and animates them
    /// via `request_repaint_after`. Re-derived on every `from_repo` call
    /// (§3.5).
    pub tool_calls: Vec<ToolCall>,
    /// Branch-state classification (ARCH §2.9 / §7.1) derived from the
    /// latest step's `response.json` terminal event. Always `InFlight`
    /// or `Stopped` for a row in this section — `Merged` and
    /// `Conflicted` are not produced for unmerged branches. Re-derived
    /// on every `from_repo` call (§3.5).
    pub state: BranchState,
}

/// A single tool call surfaced to the renderer. v0.5 scope is the
/// pulsing in-flight indicator (bl-23d9); the disk records carry more
/// metadata (timing, exit code, raw stdout) but the view-model only
/// needs identity + state to drive the indicator.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ToolCall {
    /// `tool_use.id` from the wire (e.g. `toolu_01abc…`); also the
    /// `<tool-id>/` directory name under `<conv-repo>/steps/<conv-id>/<NNN>/tools/`.
    pub tool_id: String,
    pub state: ToolCallState,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToolCallState {
    /// `input.json` has landed but `output.json` has not — the tool
    /// executor is still running. Renderer pulses this node.
    InFlight,
    /// Both `input.json` and `output.json` are present on disk. Renders
    /// statically; no repaint scheduling.
    Complete,
}

impl GitTree {
    pub fn from_repo(conv_repo: &Path) -> Result<Self, GitTreeError> {
        let git_dir = conv_repo.join(REPO_DIR);
        let log = cmd::git_log_first_parent(&git_dir)?;
        let commits = log.into_iter().map(enumerate::build_node).collect();
        // The §3.5 fd-close gate probes `/proc` for a writer still
        // holding a terminal step's `response.json` open (mid-retry).
        let probe = fd_probe::ProcFsProbe::default();
        let in_flight = enumerate::enumerate_in_flight(conv_repo, &git_dir, &probe)?;
        Ok(Self { commits, in_flight })
    }
}

/// egui widget that renders a `GitTree` as a vertical list. Main's
/// trunk comes first (each merge node with its step commits indented
/// beneath); any in-flight conversation branches follow in their own
/// section. Thin wrapper — all structure lives in the view-model.
pub fn render(ui: &mut egui::Ui, tree: &GitTree) {
    if tree.commits.is_empty() && tree.in_flight.is_empty() {
        ui.label("(no commits yet)");
        return;
    }
    for commit in &tree.commits {
        render_commit(ui, commit);
    }
    if !tree.in_flight.is_empty() {
        ui.separator();
        ui.label("agents");
        for branch in &tree.in_flight {
            render_in_flight(ui, branch);
            for step in &branch.steps {
                render_step(ui, step);
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

fn render_step(ui: &mut egui::Ui, step: &StepCommit) {
    ui.horizontal(|ui| {
        ui.label("  ↳");
        ui.monospace(&step.short_oid);
        ui.label(step.timestamp_unix.to_string());
    });
}

fn render_in_flight(ui: &mut egui::Ui, branch: &ConversationBranch) {
    ui.horizontal(|ui| {
        let (glyph, color) = state_badge(branch.state);
        ui.colored_label(color, glyph);
        ui.monospace(&branch.tip_short_oid);
        ui.label(branch.tip_timestamp_unix.to_string());
        ui.label(&branch.branch_name);
        if let Some(preview) = &branch.preview {
            ui.label(preview);
        }
    });
    if let Some(text) = &branch.streaming_text {
        ui.indent(("streaming", &branch.branch_name), |ui| {
            ui.label(text);
        });
    }
    for call in &branch.tool_calls {
        render_tool_call(ui, call);
    }
}

/// Repaint cadence for pulsing tool indicators. ~30 fps is smooth
/// enough for the eye and cheap enough for an idle UI (egui's
/// `request_repaint_after` is the standard knob for this; an in-flight
/// node sets it on every render so the loop sustains itself, while a
/// complete node leaves the default `Duration::MAX` in place and the
/// app goes back to waiting on input).
const PULSE_REPAINT_DELAY: std::time::Duration = std::time::Duration::from_millis(33);

/// Pulse frequency in radians per second — ~0.6 Hz, slow enough to read
/// as "alive" rather than "blinking error".
const PULSE_RATE_RAD_PER_SEC: f64 = 4.0;

/// Glyph + colour for each branch state (ARCH §7.1 termination markers).
/// Pure function over the enum so renderer tests can assert the mapping
/// without a windowing context. `Conflicted` is unreachable in v0.5
/// (no subagent merges yet) but the mapping is provided so a future
/// renderer pass picks it up without code changes.
pub(crate) fn state_badge(state: BranchState) -> (&'static str, egui::Color32) {
    match state {
        BranchState::Merged => ("●", egui::Color32::from_rgb(120, 200, 120)),
        BranchState::InFlight => ("◐", egui::Color32::from_rgb(120, 180, 220)),
        BranchState::Stopped => ("■", egui::Color32::from_rgb(180, 180, 180)),
        BranchState::Conflicted => ("✕", egui::Color32::from_rgb(220, 120, 120)),
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

#[cfg(test)]
mod tests;
