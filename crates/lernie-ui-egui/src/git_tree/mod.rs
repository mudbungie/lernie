//! Git-tree view-model (ARCH §7.1 live view, §3.5 agent-state contract).
//!
//! [`GitTree::from_repo`] inspects the workspace's on-disk state and
//! produces a view-model suitable for rendering. The view-model is a pure
//! function of the workspace's refs and tree content; it holds no egui
//! dependency, so a future `lernie-ui-web` crate can render the same
//! structure from the web.
//!
//! Git access is via the `git` CLI (a hard dep of lernie itself, per ARCH
//! §2.2) — no libgit2 native build step is required.
//!
//! # Workspace layout (ARCH §2.2–§2.3)
//!
//! A workspace holds one bare repository at `<workspace>/repo.git`: config
//! branches (`config/<name>`) and agent refs (`agents/<agent-id>`) — no
//! `main`. Callers pass the workspace path; this module resolves the git
//! dir to `<workspace>/repo.git` before issuing any git command, and reads
//! step records, inboxes, and marks directly from the workspace root.
//!
//! The trunk section is the config lineage (`HEAD`, which the workspace
//! repository points at `config/default`); the agent section enumerates
//! every `agents/*` ref and renders it as a **tree** by hyphenated descent
//! (§2.3) — agents never merge anywhere (§2.6), so every agent persists on
//! its own ref. Each agent carries its §3.5 state ([`AgentState`]), the two
//! ref-derived marks (declined-transfer, budget-exhausted), a
//! pending-message count from its inbox, and its branch commits with their
//! subjects (delivery / work-product-transfer commits surface by subject).

mod cmd;
mod descent;
mod detect;
mod enumerate;
mod fd_probe;
mod lock_probe;
mod marks;
mod render;
mod state;
mod streaming;
mod terminal;
mod tools;

pub use descent::{DescentRow, descent_order};
pub use render::render;
#[cfg(test)]
pub(crate) use render::state_badge;
pub use state::AgentState;

use std::path::{Path, PathBuf};

/// The bare workspace repository dir (ARCH §2.2). Mirrors
/// `src/workspace::REPO_DIR` in the harness; the duplicate constant keeps
/// the UI crate free of a dep on the harness binary.
const REPO_DIR: &str = "repo.git";

/// Top-level directory under the workspace root holding per-agent step
/// records (ARCH §2.2 / §2.3). Mirrors `src/prompt/step::STEPS_DIR`.
const STEPS_DIR: &str = "steps";

/// Top-level directory under the workspace root holding per-agent inboxes
/// (ARCH §2.11). Mirrors the harness's `inbox/<agent-id>/` layout; the
/// pending-message count and the executor-lock probe both key off it.
const INBOX_DIR: &str = "inbox";

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
    /// `git for-each-ref refs/heads/agents/`. A flat authoritative set;
    /// the render tree is derived from the ids by [`descent_order`]
    /// (§2.3 hyphenated descent) — never stored (PRINCIPLES "Single
    /// source of truth").
    pub agents: Vec<Agent>,
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
    /// Commit subject. Surfaces what a branch commit is — a dispatch, a
    /// delivery commit, or a work-product-transfer commit (§2.11, §2.6,
    /// §7.1 "delivery/result-message commits surfaced").
    pub subject: String,
}

/// One agent branch (`agents/<agent-id>`, §2.3). Named `Agent` — every
/// row is an agent, not an "unmerged conversation branch"; nothing merges
/// (§2.6), so the merged/unmerged framing is gone.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Agent {
    /// Branch name (`agents/<agent-id>`). Held separately so the renderer
    /// can label rows without re-deriving.
    pub branch_name: String,
    /// The agent id (`agents/` prefix stripped) — the identity everywhere
    /// (steps/, inbox/, worktree dir, descent). The descent tree keys off
    /// this (§2.3).
    pub agent_id: String,
    pub tip_oid: String,
    pub tip_short_oid: String,
    pub tip_timestamp_unix: i64,
    /// Commits on this branch past every config lineage, oldest to newest
    /// (each with its subject).
    pub steps: Vec<StepCommit>,
    pub preview: Option<String>,
    /// Live-updating model text for the latest step on this branch.
    /// Re-derived from `<workspace>/steps/<agent-id>/<NNN>/response.json`
    /// on every `from_repo` call (§3.5: stateless re-read on each tick).
    /// `None` when no `text_delta` events have landed yet.
    pub streaming_text: Option<String>,
    /// Tool calls under this branch's latest step's `tools/` directory
    /// (ARCH §3.3), derived purely from `input.json` / `output.json`
    /// presence. Re-derived on every `from_repo` call (§3.5).
    pub tool_calls: Vec<ToolCall>,
    /// §3.5 agent-state classification, derived from the executor lock and
    /// the latest step's `response.json` terminal segment. Re-derived on
    /// every `from_repo` call (§3.5).
    pub state: AgentState,
    /// Count of pending (undelivered) messages in the agent's inbox
    /// (`<workspace>/inbox/<agent-id>/`, §2.11). A non-empty inbox drives
    /// the §7.1 pending-message indicator; derived from the listing,
    /// never stored.
    pub pending_messages: usize,
    /// `refs/lernie/conflicted/<agent-id>` exists — a work-product
    /// transfer was declined (§2.6). Rendered as an orthogonal mark
    /// alongside the state (§3.5, §7.1).
    pub declined_transfer: bool,
    /// `refs/lernie/budget-exhausted/<agent-id>` exists — the agent hit a
    /// budget ceiling (§6). Rendered as an orthogonal mark alongside the
    /// state (§3.5, §7.1).
    pub budget_exhausted: bool,
}

/// A single tool call surfaced to the renderer. The disk records carry
/// more metadata (timing, exit code, raw stdout) but the view-model only
/// needs identity + state to drive the indicator.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ToolCall {
    /// `tool_use.id` from the wire (e.g. `toolu_01abc…`); also the
    /// `<tool-id>/` directory name under
    /// `<workspace>/steps/<agent-id>/<NNN>/tools/`.
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
    pub fn from_repo(workspace: &Path) -> Result<Self, GitTreeError> {
        let git_dir = workspace.join(REPO_DIR);
        let log = cmd::git_log_first_parent(&git_dir)?;
        let commits = log.into_iter().map(enumerate::build_node).collect();
        // Two liveness observations (ARCH §2.11 "two observations"): the
        // executor lock (inbox-dir fd) answers *is anyone driving*
        // (`live`); the open `response.json` fd answers *is a model call in
        // flight right now* (the `in_flight` sub-state).
        let lock = lock_probe::ProcFsLockProbe::default();
        let writer = fd_probe::ProcFsProbe::default();
        let agents = enumerate::enumerate_agents(workspace, &git_dir, &lock, &writer)?;
        Ok(Self { commits, agents })
    }
}

#[cfg(test)]
mod tests;
