//! Shared test fixtures for the workspace physical model (ARCH §2.2):
//! a real scaffolded workspace (bare repo.git + first config commit on
//! `config/default`) and an agent branch forked off it, exactly the
//! shapes production verbs run against. Test-only (`cfg(test)` on the
//! module declaration).

use super::{DEFAULT_CONFIG_REF, agent_ref, agent_worktree, repo_git};
use crate::template::{GitRunner, RealGit, scaffold};
use std::path::{Path, PathBuf};
use tempfile::TempDir;

/// A real workspace under a fresh tempdir (the `lernie new` core, with
/// an empty descriptions pool). Returns `(holder, workspace_path)`.
pub(crate) fn workspace() -> (TempDir, PathBuf) {
    let holder = TempDir::new().unwrap();
    let data_root = holder.path().join("data");
    std::fs::create_dir_all(&data_root).unwrap();
    let ws = holder.path().join("ws");
    scaffold(&ws, &data_root, &RealGit::new()).unwrap();
    (holder, ws)
}

/// Fork `agents/<id>` off `start` (a config branch or another agent's
/// ref) with its worktree at `<ws>/agents/<id>`, and land a dispatch
/// commit (`goal.md`) so the tip advances past the fork point. Returns
/// the worktree path.
pub(crate) fn spawn_agent(ws: &Path, id: &str, start: &str) -> PathBuf {
    let g = RealGit::new();
    let wt = agent_worktree(ws, id);
    let wt_str = wt.to_string_lossy().to_string();
    let branch = agent_ref(id);
    g.run(
        &repo_git(ws),
        &[
            "worktree",
            "add",
            "-b",
            branch.as_str(),
            wt_str.as_str(),
            start,
        ],
    )
    .unwrap();
    // Unique goal content (the id) so a child forked off a parent's
    // tip still stages a change and the dispatch commit lands.
    std::fs::write(wt.join("goal.md"), id).unwrap();
    g.run(&wt, &["add", "goal.md"]).unwrap();
    g.run(&wt, &["commit", "-m", "dispatch"]).unwrap();
    wt
}

/// [`spawn_agent`] off the default config branch — the fresh-root shape.
pub(crate) fn spawn_root(ws: &Path, id: &str) -> PathBuf {
    spawn_agent(ws, id, DEFAULT_CONFIG_REF)
}

/// Advance `config/default` with the given control files — the
/// harness-assisted user act of §2.2, as a test fixture: materialize a
/// transient checkout, overwrite the files, commit, tear down. Agents
/// forked after this govern under the new head.
pub(crate) fn amend_config(ws: &Path, files: &[(&str, &str)]) {
    let g = RealGit::new();
    let author = ws.join(".amend-config");
    let author_str = author.to_string_lossy().to_string();
    g.run(
        &repo_git(ws),
        &["worktree", "add", author_str.as_str(), DEFAULT_CONFIG_REF],
    )
    .unwrap();
    for (rel, content) in files {
        let path = author.join(rel);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, content).unwrap();
    }
    g.run(&author, &["add", "-A"]).unwrap();
    g.run(&author, &["commit", "-m", "config: amend"]).unwrap();
    g.run(&repo_git(ws), &["worktree", "remove", author_str.as_str()])
        .unwrap();
}
