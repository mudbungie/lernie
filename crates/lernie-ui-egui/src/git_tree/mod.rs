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
//! # v0.3 layout
//!
//! The conv-repo (ARCH §2.2) holds its `.git` inside the primary
//! worktree at `<conv-repo>/root/`; control-plane files live at the
//! conv-repo root, outside any worktree. Callers pass the conv-repo
//! path; this module resolves the git working dir to `<conv-repo>/root/`
//! before issuing any git command.
//!
//! Each user-message dispatch spawns a bare `<conv-id>` branch off
//! `main` and merges back with `--no-ff` on completion (ARCH §2.3).
//! The merge commit on `main`'s first-parent trunk introduces step
//! files at `steps/<conv-id>/<NNN>/{request.json,response.json}`. The
//! UI keys off that path to recognize a conversation merge and pull
//! the user message from `request.json`'s `messages[0].content`.
//! Subagent branches (named by full hyphenated descent) appear under
//! the same enumeration when unmerged.

mod cmd;
mod detect;
mod enumerate;

use std::path::{Path, PathBuf};

/// Subdir under the conv-repo where the primary worktree (and the
/// only `.git`) lives (ARCH §2.2). Mirrors `src/template::ROOT_WORKTREE`
/// in the harness; the duplicate constant keeps the UI crate free of a
/// dep on the harness binary.
const ROOT_WORKTREE: &str = "root";

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
    /// for `--no-ff` merge commits whose introduced files include
    /// `steps/<conv-id>/<NNN>/...`; `None` for trunk commits that
    /// are neither (initial scaffold commit, config tweaks, etc.).
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
}

impl GitTree {
    pub fn from_repo(conv_repo: &Path) -> Result<Self, GitTreeError> {
        let git_dir = conv_repo.join(ROOT_WORKTREE);
        let log = cmd::git_log_first_parent(&git_dir)?;
        let mut commits = Vec::with_capacity(log.len());
        for entry in log {
            commits.push(enumerate::build_node(&git_dir, entry)?);
        }
        let in_flight = enumerate::enumerate_in_flight(&git_dir)?;
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
}

#[cfg(test)]
mod tests;
