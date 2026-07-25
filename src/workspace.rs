//! The workspace physical model (ARCH §2.2–§2.3).
//!
//! A **workspace** is one git repository at `<workspace>/repo.git`
//! (bare), holding config branches (`config/<name>`) and agent refs
//! (`agents/<agent-id>`). There is no `main`: no branch is a trunk, and
//! which advancement rule a ref lives under is derived from its path
//! prefix, never recorded anywhere else (§2.3). Agent worktrees are
//! materialized as siblings under `<workspace>/agents/`; `steps/` and
//! `inbox/` sit at the workspace root, outside every worktree (§2.2).
//!
//! Control files — `workflow.yaml`, `manifest.yaml`, `providers.yaml`,
//! `souls/`, `version` — are read from the agent's **governing config
//! commit**: the nearest ancestor of the agent's branch reachable from
//! any `config/*` ref, derived from ancestry (`git merge-base`) and
//! never stored (§2.2, PRINCIPLES "Single source of truth").
//!
//! This module owns the path/ref arithmetic and the ancestry
//! derivation; it holds no state and performs no writes beyond what the
//! injected [`GitRunner`] is asked to run.

use crate::template::GitRunner;
use std::io;
use std::path::{Path, PathBuf};

/// The workspace repository, bare, at `<workspace>/repo.git` (§2.2).
pub const REPO_DIR: &str = "repo.git";
/// Directory under the workspace root where agent worktrees live as
/// siblings — `<workspace>/agents/<agent-id>/` (§2.2).
pub const AGENTS_DIR: &str = "agents";
/// Ref-namespace prefix for agent branches: `agents/<agent-id>` (§2.3).
pub const AGENT_REF_PREFIX: &str = "agents/";
/// Ref-namespace prefix for config branches: `config/<name>` (§2.3).
pub const CONFIG_REF_PREFIX: &str = "config/";
/// The default config branch a fresh root agent forks off (§2.3 *Fresh
/// start* — the head of a config branch; `lernie new` authors this one).
pub const DEFAULT_CONFIG_REF: &str = "config/default";

/// A config branch ref, `config/<name>` (§2.3). The prefix is the kind
/// (config vs agent), applied only at the git boundary — the bare name
/// is what a user names on the `lernie config` command line.
pub fn config_ref(name: &str) -> String {
    format!("{CONFIG_REF_PREFIX}{name}")
}
/// The harness-facing control paths the dispatch commit removes from an
/// agent's tree when it forks off a config commit (§2.2 "Control is
/// read from the config commit; worktrees hold only context").
/// `descriptions/**` stays: it *is* context (§3.3).
pub const CONTROL_PATHS: &[&str] = &[
    "manifest.yaml",
    "workflow.yaml",
    "providers.yaml",
    "version",
    "souls",
];

/// `<workspace>/repo.git` — where every ref-level git command runs.
pub fn repo_git(workspace: &Path) -> PathBuf {
    workspace.join(REPO_DIR)
}

/// `<workspace>/agents/<agent-id>` — the agent's worktree (§2.2).
pub fn agent_worktree(workspace: &Path, agent_id: &str) -> PathBuf {
    workspace.join(AGENTS_DIR).join(agent_id)
}

/// The agent's branch ref, `agents/<agent-id>` (§2.3). The id — the
/// full hyphenated descent — is the primary identifier everywhere
/// (inbox and steps namespaces, worktree dir, `LERNIE_CONV_BRANCH`);
/// the prefix is applied only at the git boundary.
pub fn agent_ref(agent_id: &str) -> String {
    format!("{AGENT_REF_PREFIX}{agent_id}")
}

/// Does the agent exist? — `git rev-parse --verify refs/heads/agents/<id>`
/// against the bare repo (§2.3: the ref namespace *is* the registry, so
/// existence is a query, never a stored fact). The one home of the
/// question every verb addressing an existing agent asks: `lernie stop`
/// before signaling (via [`crate::prompt::stop::inspector`]) and
/// `lernie message` before depositing (§2.11 — "a message is content
/// addressed to an *existing* agent"). A non-zero exit is the answer
/// `false`, which also covers an id git refuses as a ref name.
pub fn agent_exists(workspace: &Path, agent_id: &str, git: &dyn GitRunner) -> bool {
    let refspec = format!("refs/heads/{}", agent_ref(agent_id));
    git.run(
        &repo_git(workspace),
        &["rev-parse", "--verify", "--quiet", &refspec],
    )
    .is_ok()
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

/// Resolve a config branch's head commit sha (§2.3 *Fresh start* fork
/// point). Loud when the branch does not exist — a workspace with no
/// `config/default` cannot dispatch a root agent.
pub fn config_head(workspace: &Path, config_ref: &str, git: &dyn GitRunner) -> io::Result<String> {
    let refspec = format!("refs/heads/{config_ref}");
    git.run_capture(&repo_git(workspace), &["rev-parse", "--verify", &refspec])
}

/// Enumerate the workspace's agent ids: every `agents/*` ref, prefix
/// stripped (§2.3 — the prefix is the kind, derived from the path).
/// This is the §8 enumeration seam: scan/stop/budget candidate sets
/// read agent branches from here, never "every branch except main".
pub fn agent_ids(workspace: &Path, git: &dyn GitRunner) -> io::Result<Vec<String>> {
    let out = git.run_capture(
        &repo_git(workspace),
        &[
            "for-each-ref",
            "--format=%(refname:short)",
            "refs/heads/agents/",
        ],
    )?;
    Ok(out
        .lines()
        .map(str::trim)
        .filter_map(|r| r.strip_prefix(AGENT_REF_PREFIX))
        .filter(|id| !id.is_empty())
        .map(str::to_string)
        .collect())
}

/// Derive the **governing config commit** of `agent_id`'s branch: the
/// nearest ancestor reachable from any `config/*` ref (§2.2). For each
/// config head, `git merge-base <agent-ref> <head>` yields the shared
/// ancestor on that lineage; the governing commit is the *descendant*
/// among the candidates (nearest to the agent's tip). Derived from
/// ancestry, never stored. Loud when no config lineage reaches the
/// branch, and loud when two candidates are incomparable — both mean a
/// defective workspace, declined rather than guessed (PRINCIPLES
/// "Decline illegal operations").
pub fn governing_config(
    workspace: &Path,
    agent_id: &str,
    git: &dyn GitRunner,
) -> io::Result<String> {
    let repo = repo_git(workspace);
    let heads = git.run_capture(
        &repo,
        &["for-each-ref", "--format=%(refname)", "refs/heads/config/"],
    )?;
    let target = agent_ref(agent_id);
    let mut best: Option<String> = None;
    for head in heads.lines().map(str::trim).filter(|l| !l.is_empty()) {
        // A config lineage sharing no ancestor with the agent (a fresh
        // orphan config) contributes no candidate.
        let Ok(base) = git.run_capture(&repo, &["merge-base", &target, head]) else {
            continue;
        };
        best = Some(match best {
            None => base,
            Some(prev) if prev == base => prev,
            Some(prev) => nearest(&repo, prev, base, git)?,
        });
    }
    best.ok_or_else(|| {
        io::Error::other(format!(
            "no config/* ancestor for {target} — every agent forks off a config commit (§2.2)"
        ))
    })
}

/// Of two candidate ancestors of one branch tip, keep the descendant —
/// the nearer one. Incomparable candidates are declined loudly.
fn nearest(repo: &Path, a: String, b: String, git: &dyn GitRunner) -> io::Result<String> {
    if git
        .run(repo, &["merge-base", "--is-ancestor", &a, &b])
        .is_ok()
    {
        return Ok(b);
    }
    if git
        .run(repo, &["merge-base", "--is-ancestor", &b, &a])
        .is_ok()
    {
        return Ok(a);
    }
    Err(io::Error::other(format!(
        "governing config is ambiguous: candidates {a} and {b} are incomparable ancestors \
         — declined (§2.2, PRINCIPLES)"
    )))
}

/// Read one control file's contents from a config commit's tree
/// (`git show <commit>:<path>`, §2.2 "Control is read from the config
/// commit"). The worktree is never consulted.
pub fn show_control(
    workspace: &Path,
    commit: &str,
    path: &str,
    git: &dyn GitRunner,
) -> io::Result<String> {
    let spec = format!("{commit}:{path}");
    git.run_capture(&repo_git(workspace), &["show", &spec])
}

/// Does `path` exist in the config commit's tree? (`git cat-file -e`.)
pub fn control_exists(workspace: &Path, commit: &str, path: &str, git: &dyn GitRunner) -> bool {
    let spec = format!("{commit}:{path}");
    git.run(&repo_git(workspace), &["cat-file", "-e", &spec])
        .is_ok()
}

#[cfg(test)]
pub(crate) mod fixture;
#[cfg(test)]
mod tests;
