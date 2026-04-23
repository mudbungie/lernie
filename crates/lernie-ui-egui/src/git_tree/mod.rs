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
//! # Shapes handled
//!
//! - **v0.1-shape** (ARCH §12 v0.1 exception): flat linear history on
//!   `main` with one commit per exchange, each carrying a top-level
//!   `exchanges/<id>.json` file. The preview comes from the file's
//!   `user_message` key.
//! - **v0.2-shape** (ARCH §2.3, §2.6): each exchange is an `ex/<ts>-<id>`
//!   branch off `main` that merges back with `--no-ff`. The merge
//!   commit lives on `main`'s trunk; its step commits (snapshot,
//!   response, compactor merge) live on the exchange branch. Unmerged
//!   exchange branches are enumerable via `git branch --list ex/*
//!   --no-merged main` (PRINCIPLES.md single-source-of-truth).
//!
//! The two shapes coexist — a repo migrated from v0.1 to v0.2 has both
//! kinds of commits on `main`; the view-model handles each without the
//! other's existence affecting it.

mod cmd;
mod detect;
mod enumerate;

use std::path::{Path, PathBuf};

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
    /// `main`'s first-parent trunk, oldest to newest. For v0.2 repos
    /// this includes the `--no-ff` merge commits of completed
    /// exchanges; for v0.1 repos it is a flat linear list of exchange
    /// commits.
    pub commits: Vec<CommitNode>,
    /// Exchange branches not yet merged to `main` (`git branch --list
    /// ex/* --no-merged main`). Empty in repos where every exchange
    /// has merged back, which is the steady state after each
    /// `lernie prompt` completes.
    pub in_flight: Vec<ExchangeBranch>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommitNode {
    pub oid: String,
    pub short_oid: String,
    pub timestamp_unix: i64,
    /// The exchange id this commit represents, if any. Populated for
    /// v0.1-shape exchange commits and for v0.2-shape `--no-ff` merge
    /// commits; `None` for trunk commits that are neither (initial
    /// commit, config tweaks, etc.).
    pub exchange_id: Option<String>,
    pub preview: Option<String>,
    /// Step commits on the exchange branch this merge commit closes.
    /// Empty unless this is a v0.2 merged-exchange commit. Ordered
    /// oldest to newest (snapshot, response, compactor merge).
    pub steps: Vec<StepCommit>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StepCommit {
    pub oid: String,
    pub short_oid: String,
    pub timestamp_unix: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExchangeBranch {
    pub branch_name: String,
    pub exchange_id: String,
    pub tip_oid: String,
    pub tip_short_oid: String,
    pub tip_timestamp_unix: i64,
    /// Commits on this branch not reachable from `main`, oldest to
    /// newest. Same shape as merged exchanges' steps.
    pub steps: Vec<StepCommit>,
    pub preview: Option<String>,
}

impl GitTree {
    pub fn from_repo(repo: &Path) -> Result<Self, GitTreeError> {
        let log = cmd::git_log_first_parent(repo)?;
        let mut commits = Vec::with_capacity(log.len());
        for entry in log {
            commits.push(enumerate::build_node(repo, entry)?);
        }
        let in_flight = enumerate::enumerate_in_flight(repo)?;
        Ok(Self { commits, in_flight })
    }
}

/// egui widget that renders a `GitTree` as a vertical list. Main's
/// trunk comes first (each merge node with its step commits indented
/// beneath); any in-flight exchange branches follow in their own
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
        ui.label("in-flight exchanges");
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
        if let Some(id) = &commit.exchange_id {
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

fn render_in_flight(ui: &mut egui::Ui, branch: &ExchangeBranch) {
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
