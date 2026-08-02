//! The workspace guards (ARCH §2.2, §2.3) — the two questions every verb
//! taking a path and an agent id from outside asks before it does
//! anything: *is this a workspace* ([`require`]) and *does this agent
//! exist* ([`require_agent`]). One home each, so the five id-taking verbs
//! — `message`, `advance`, `stop`, `dispatch`, `bundle` — share the rule
//! and the voice rather than each keeping a copy of both.
//!
//! The existence half is not universal, and the exception says why the
//! rule holds: `lernie delete` (§9.2) guards the layout and the id's
//! *shape* but admits an id no ref answers to, because absence is the
//! postcondition it establishes — the other five decline an absent agent
//! precisely because their act would silently do nothing.

use super::{GitRunner, Path, PathBuf, agent_ref, repo_git};

/// Does the agent exist? — `git rev-parse --verify refs/heads/agents/<id>`
/// against the bare repo (§2.3: the ref namespace *is* the registry, so
/// existence is a query, never a stored fact). The one home of the
/// question; the verbs ask it through [`require_agent`], `lernie stop`
/// as a plain predicate (via [`crate::prompt::stop::inspector`]). A
/// non-zero exit is the answer `false`, which also covers an id git
/// refuses as a ref name.
pub fn agent_exists(workspace: &Path, agent_id: &str, git: &dyn GitRunner) -> bool {
    let refspec = format!("refs/heads/{}", agent_ref(agent_id));
    git.run(
        &repo_git(workspace),
        &["rev-parse", "--verify", "--quiet", &refspec],
    )
    .is_ok()
}

/// A well-formed agent id with no `agents/*` ref. One decline, one voice,
/// for every verb that addresses an existing agent; `reason` is the
/// verb's own clause naming *why* it needed one, so what differs between
/// verbs is the cause, never the phrasing or the remedy.
#[derive(Debug, thiserror::Error)]
#[error(
    "no agent {id:?} in this workspace — {reason}; check the id against the workspace's \
     `agents/*` refs, or start an agent with `lernie prompt` / `lernie dispatch`"
)]
pub struct UnknownAgent {
    id: String,
    reason: &'static str,
}

/// The existence guard every verb taking an agent id from outside runs
/// before doing anything else (§2.3) — `message` before depositing
/// (§2.11), `advance` before the lease, `dispatch` before deriving the
/// parent's governing config (§2.5). Paired with [`require`], which
/// guards the workspace itself, this is the shared sequence README
/// promises at all five id-taking verbs.
pub fn require_agent(
    workspace: &Path,
    agent_id: &str,
    reason: &'static str,
    git: &dyn GitRunner,
) -> Result<(), UnknownAgent> {
    if agent_exists(workspace, agent_id, git) {
        return Ok(());
    }
    Err(UnknownAgent {
        id: agent_id.to_owned(),
        reason,
    })
}

/// Layout guard failures (§2.2; pre-v1 clean break, §10).
#[derive(Debug, thiserror::Error)]
pub enum LayoutError {
    /// The retired per-conversation layout: a `root/` primary worktree
    /// with untracked control files at the repo root. Pre-v1 there is
    /// no migration (§10 clean-break note): the refusal is loud and
    /// names both what was found and what the current layout is.
    #[error(
        "{0} uses the retired per-conversation layout (a `root/` primary worktree with \
         control files at the repo root); the current layout is one repo per workspace — \
         `<workspace>/repo.git` (bare) with `config/*` branches and `agents/*` refs \
         (ARCH §2.2). Pre-v1 clean break (§10): no migration — create a fresh workspace \
         with `lernie new` and re-author its config"
    )]
    OldLayout(PathBuf),
    /// No `repo.git` and no old-layout signature: not a workspace.
    #[error("{0} is not a workspace (no repo.git) — create one with `lernie new` (ARCH §2.2)")]
    NotAWorkspace(PathBuf),
}

/// Require `workspace` to be a current-layout workspace: `repo.git`
/// present. The retired per-conversation layout is refused with a
/// clear, actionable error (pre-v1 clean break); anything else is not
/// a workspace at all.
pub fn require(workspace: &Path) -> Result<(), LayoutError> {
    if repo_git(workspace).is_dir() {
        return Ok(());
    }
    if workspace.join("root").join(".git").exists() {
        return Err(LayoutError::OldLayout(workspace.to_path_buf()));
    }
    Err(LayoutError::NotAWorkspace(workspace.to_path_buf()))
}
