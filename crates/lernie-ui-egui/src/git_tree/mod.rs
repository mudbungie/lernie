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
//! # v0.3.1 layout
//!
//! The conv-repo (ARCH §2.2) holds its `.git` inside the primary
//! worktree at `<conv-repo>/root/`; control-plane files (`manifest.yaml`,
//! `souls/`, the `steps/` tree) live at the conv-repo root, outside any
//! worktree. Callers pass the conv-repo path; this module resolves the
//! git working dir to `<conv-repo>/root/` before issuing any git command,
//! and reads step records (for previews) directly from the conv-repo
//! root.
//!
//! Each user-message dispatch spawns a bare `<conv-id>` branch off
//! `main` and merges back with `--no-ff` on completion (ARCH §2.3).
//! v0.3.1 (bl-c22c P4) keys conversation detection off the merged
//! branch's name, recovered from the trunk merge commit's default
//! `Merge branch '<name>'` subject — branch names already encode the
//! conv-id (or hyphenated descent for subagents). Step records are no
//! longer in any commit (§2.3 "Step records are not committed to git"),
//! so the user-message preview reads from
//! `<conv-repo>/steps/<conv-id>/001/request.json` on disk.

mod cmd;
mod detect;
mod enumerate;
mod streaming;
mod tools;

use std::path::{Path, PathBuf};

/// Subdir under the conv-repo where the primary worktree (and the
/// only `.git`) lives (ARCH §2.2). Mirrors `src/template::ROOT_WORKTREE`
/// in the harness; the duplicate constant keeps the UI crate free of a
/// dep on the harness binary.
const ROOT_WORKTREE: &str = "root";

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
    /// `main`'s first-parent trunk, oldest to newest. Includes the
    /// `--no-ff` merge commits of completed root conversations.
    pub commits: Vec<CommitNode>,
    /// Conversation branches not yet merged to `main`, enumerated via
    /// `git for-each-ref --no-merged=main refs/heads/`. Empty in the
    /// steady state where every dispatch has merged back.
    pub in_flight: Vec<ConversationBranch>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommitNode {
    pub oid: String,
    pub short_oid: String,
    pub timestamp_unix: i64,
    /// The conversation id this commit represents, if any. Populated
    /// for `--no-ff` merge commits whose default `Merge branch '<name>'`
    /// subject parses (ARCH §2.3); `None` for non-merge commits and for
    /// merges whose subject does not match the conversation shape
    /// (initial scaffold commit, config tweaks, hand-run merges with
    /// rewritten subjects, etc.).
    pub conv_id: Option<String>,
    pub preview: Option<String>,
    /// Step commits on the conversation branch this merge commit
    /// closes. Empty unless this is a merged-conversation commit.
    /// Ordered oldest to newest (dispatch, response, compactor merge).
    pub steps: Vec<StepCommit>,
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
        let git_dir = conv_repo.join(ROOT_WORKTREE);
        let log = cmd::git_log_first_parent(&git_dir)?;
        let mut commits = Vec::with_capacity(log.len());
        for entry in log {
            commits.push(enumerate::build_node(conv_repo, &git_dir, entry)?);
        }
        let in_flight = enumerate::enumerate_in_flight(conv_repo, &git_dir)?;
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
        for step in &commit.steps {
            render_step(ui, step);
        }
    }
    if !tree.in_flight.is_empty() {
        ui.separator();
        ui.label("in-flight conversations");
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
        if let Some(id) = &commit.conv_id {
            ui.label(id);
        }
        if let Some(preview) = &commit.preview {
            ui.label(preview);
        }
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
