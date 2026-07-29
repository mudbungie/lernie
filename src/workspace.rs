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
//! injected [`GitRunner`] is asked to run. The two guards every verb
//! runs before any of it — *is this a workspace*, *does this agent
//! exist* — are [`guard`], re-exported here.

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
/// Ref-namespace prefix for the §2.6 **decline** mark,
/// `refs/lernie/conflicted/<agent-id>`. One namespace, one home here in
/// the ref-naming module, written by every operation that must refuse
/// rather than guess: the declined work-product transfer
/// (`prompt::dispatch::transfer`) and the conflicted compaction merge
/// (`prompt::compactor::merge`). The UI renders it as `declined-transfer`
/// alongside the orthogonal budget-exhausted mark (§3.5, §7.1).
pub const CONFLICTED_REF_PREFIX: &str = "refs/lernie/conflicted/";

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

/// Resolve a config branch's head commit sha (§2.3 *Fresh start* fork
/// point). Loud when the branch does not exist — a workspace with no
/// `config/default` cannot dispatch a root agent.
pub fn config_head(workspace: &Path, config_ref: &str, git: &dyn GitRunner) -> io::Result<String> {
    let refspec = format!("refs/heads/{config_ref}");
    git.run_capture(&repo_git(workspace), &["rev-parse", "--verify", &refspec])
}

/// Enumerate the short names under one ref-namespace prefix, prefix
/// stripped (§2.3 — the prefix is the kind, derived from the path, never
/// recorded). The workspace keeps exactly two registries, and both are
/// this one query: the ref namespace *is* the registry.
fn ref_names(workspace: &Path, prefix: &str, git: &dyn GitRunner) -> io::Result<Vec<String>> {
    let pattern = format!("refs/heads/{prefix}");
    let out = git.run_capture(
        &repo_git(workspace),
        &["for-each-ref", "--format=%(refname:short)", &pattern],
    )?;
    Ok(out
        .lines()
        .map(str::trim)
        .filter_map(|r| r.strip_prefix(prefix))
        .filter(|name| !name.is_empty())
        .map(str::to_string)
        .collect())
}

/// Enumerate the workspace's agent ids: every `agents/*` ref, prefix
/// stripped (§2.3). This is the §8 enumeration seam: scan/stop/budget
/// candidate sets read agent branches from here, never "every branch
/// except main".
pub fn agent_ids(workspace: &Path, git: &dyn GitRunner) -> io::Result<Vec<String>> {
    ref_names(workspace, AGENT_REF_PREFIX, git)
}

/// Enumerate the workspace's config lineage names: every `config/*` ref,
/// prefix stripped (§2.3). The bare names are what a user names on the
/// `lernie config` command line, so this is both the existence query for
/// a `--from <source>` and the pool a decline names.
pub fn config_names(workspace: &Path, git: &dyn GitRunner) -> io::Result<Vec<String>> {
    ref_names(workspace, CONFIG_REF_PREFIX, git)
}

/// The **governing lineage** of `agent_id`'s branch: every `config/*`
/// ref whose history reaches the branch, paired with the ancestor it
/// contributes (`git merge-base <agent-ref> <head>`). A config lineage
/// sharing no ancestor with the agent — a fresh orphan config — reaches
/// the branch through nothing and is absent from the result.
///
/// This is the candidate set [`governing_config`] folds to one commit,
/// and the ref set `archive::bundle` carries (§9.2): a config branch
/// that advanced past the fork is *not* an ancestor of the agent, yet
/// it is the ref the merge-base is taken against, so the lineage
/// travels **at its heads** and a replayed workspace re-derives over
/// the same candidate set rather than an approximation of it.
pub fn config_lineage(
    workspace: &Path,
    agent_id: &str,
    git: &dyn GitRunner,
) -> io::Result<Vec<(String, String)>> {
    let repo = repo_git(workspace);
    let heads = git.run_capture(
        &repo,
        &["for-each-ref", "--format=%(refname)", "refs/heads/config/"],
    )?;
    let target = agent_ref(agent_id);
    Ok(heads
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty())
        .filter_map(|head| {
            let base = git
                .run_capture(&repo, &["merge-base", &target, head])
                .ok()?;
            Some((head.to_owned(), base))
        })
        .collect())
}

/// Derive the **governing config commit** of `agent_id`'s branch: the
/// nearest ancestor reachable from any `config/*` ref (§2.2). Each ref
/// of the governing lineage ([`config_lineage`]) contributes the shared
/// ancestor on its lineage; the governing commit is the *descendant*
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
    let mut best: Option<String> = None;
    for (_, base) in config_lineage(workspace, agent_id, git)? {
        best = Some(match best {
            None => base,
            Some(prev) if prev == base => prev,
            Some(prev) => nearest(&repo, prev, base, git)?,
        });
    }
    best.ok_or_else(|| {
        io::Error::other(format!(
            "no config/* ancestor for {} — every agent forks off a config commit (§2.2)",
            agent_ref(agent_id)
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

mod guard;
pub use guard::{LayoutError, UnknownAgent, agent_exists, require, require_agent};

#[cfg(test)]
pub(crate) mod fixture;
#[cfg(test)]
mod tests;
