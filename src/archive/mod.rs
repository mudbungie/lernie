//! Bundle and replay of an agent subtree (ARCH §9.2 *Replay and
//! archival*).
//!
//! A **run** is an agent subtree within a long-lived workspace, so the
//! archival unit follows the agent, not the workspace. [`bundle`] writes
//! the whole run as **one `git bundle` plus two slices** (§9.2): the
//! bundle carries the agent's branch, its hyphen-descendants (§2.3), and
//! the complete ancestry those refs reach — back through the dispatch
//! commits to the founding commit — while `steps/<id>*` and `inbox/<id>*`
//! (the diagnostic directories outside git, §2.2) ride alongside as plain
//! copies. [`replay`] reconstructs a **scratch workspace** from that
//! archive: it fetches every branch out of the bundle into a fresh repo,
//! materializes the subtree root's worktree, and restores the two slices.
//! Inspection is then the ordinary frontend over the scratch workspace
//! (§3.5) — **replay is not a mode** (§2.3); it is plumbing plus a verb.
//!
//! With the workspace substrate (§2.2), the governing config commit is
//! an ancestor of every agent branch, so it travels inside the bundle's
//! ancestry exactly as §9.2 promises — no config sidecar. (The bundle's
//! *refs* are the `agents/*` subtree only; the config commit rides as a
//! reachable object, and the agent's governing commit is re-derivable
//! from the dispatch commit's parent.)

use crate::template::GitRunner;
use crate::workspace;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

#[cfg(test)]
mod tests;

/// The bundle filename inside an archive directory (§9.2 "One `git
/// bundle`").
pub const BUNDLE_FILE: &str = "agents.bundle";
/// The diagnostic slice directory names carried beside the bundle (§2.2).
const SLICES: [&str; 2] = ["steps", "inbox"];

/// Every way [`bundle`] or [`replay`] can fail.
#[derive(Debug, thiserror::Error)]
pub enum ArchiveError {
    #[error("I/O error: {0}")]
    Io(#[from] io::Error),
    #[error("git {op}: {source}")]
    Git {
        op: &'static str,
        #[source]
        source: io::Error,
    },
    #[error("no branch matches agent id {0:?} in the workspace")]
    UnknownAgent(String),
    #[error("bundle {0} not found")]
    BundleMissing(PathBuf),
    #[error("bundle names no branches")]
    EmptyBundle,
    #[error("bundle branches {0:?} share no common subtree root")]
    MalformedBundle(Vec<String>),
    #[error("replay destination {0} already exists")]
    DestExists(PathBuf),
}

/// Archive the agent subtree rooted at `agent_id` into `out_dir` (§9.2):
/// one `git bundle` of `agents/<agent_id>` and its hyphen-descendants
/// (with all ancestry they reach — the governing config commit included,
/// §2.2), plus the `steps/<id>*` and `inbox/<id>*` slices.
///
/// The bundle's refs are enumerated with `git branch --list` against the
/// bare `repo.git` (the pattern `agents/<id>` plus `agents/<id>-*` is
/// the §2.3 descent namespace); an agent id that matches no branch is
/// [`ArchiveError::UnknownAgent`].
pub fn bundle(
    ws: &Path,
    agent_id: &str,
    out_dir: &Path,
    git: &dyn GitRunner,
) -> Result<(), ArchiveError> {
    let repo = workspace::repo_git(ws);
    let refs = subtree_refs(&repo, agent_id, git)?;
    if refs.is_empty() {
        return Err(ArchiveError::UnknownAgent(agent_id.to_owned()));
    }
    fs::create_dir_all(out_dir)?;
    let bundle_path = out_dir.join(BUNDLE_FILE);
    let bundle_str = bundle_path.to_string_lossy().into_owned();
    let mut args: Vec<&str> = vec!["bundle", "create", &bundle_str];
    args.extend(refs.iter().map(String::as_str));
    git.run(&repo, &args).map_err(|source| ArchiveError::Git {
        op: "bundle create",
        source,
    })?;
    for slice in SLICES {
        copy_matching(&ws.join(slice), &out_dir.join(slice), agent_id)?;
    }
    Ok(())
}

/// Reconstruct a scratch workspace from an archive directory under
/// `scratch_base` (§9.2). Fetches every branch out of
/// `<archive>/agents.bundle` into a fresh bare `repo.git` (§2.2),
/// materializes the subtree root's worktree under `agents/`, and
/// restores the `steps/` and `inbox/` slices. Returns the scratch
/// workspace path — the frontend inspects it directly.
///
/// The scratch workspace is `<scratch_base>/<primary-id>`, where the
/// **primary id** is the shortest agent id in the bundle (every other
/// branch is one of its hyphen-descendants, §2.3). It must not already
/// exist ([`ArchiveError::DestExists`]).
pub fn replay(
    archive: &Path,
    scratch_base: &Path,
    git: &dyn GitRunner,
) -> Result<PathBuf, ArchiveError> {
    let bundle_path = archive.join(BUNDLE_FILE);
    if !bundle_path.exists() {
        return Err(ArchiveError::BundleMissing(bundle_path));
    }
    let heads = bundle_heads(archive, &bundle_path, git)?;
    let primary = primary_head(&heads)?;
    let scratch = scratch_base.join(primary);
    if scratch.exists() {
        return Err(ArchiveError::DestExists(scratch));
    }
    let repo = workspace::repo_git(&scratch);
    fs::create_dir_all(&repo)?;
    // Absolute bundle path: `-C repo.git` moves git's cwd, so a relative
    // spelling would resolve against the wrong directory.
    let bundle_abs = fs::canonicalize(&bundle_path)?;
    let bundle_arg = bundle_abs.to_string_lossy().into_owned();
    run(git, &repo, &["init", "-q", "--bare"], "init")?;
    run(
        git,
        &repo,
        &["fetch", &bundle_arg, "refs/heads/*:refs/heads/*"],
        "fetch",
    )?;
    let primary_ref = workspace::agent_ref(primary);
    let primary_wt = workspace::agent_worktree(&scratch, primary);
    let primary_wt_str = primary_wt.to_string_lossy().into_owned();
    run(
        git,
        &repo,
        &["worktree", "add", &primary_wt_str, &primary_ref],
        "worktree add",
    )?;
    for slice in SLICES {
        let src = archive.join(slice);
        if src.is_dir() {
            copy_dir_all(&src, &scratch.join(slice))?;
        }
    }
    Ok(scratch)
}

/// `lernie replay` wiring (§3.4/§9.2): resolve the scratch base under the
/// data root (`replays/`, isolated by `LERNIE_HOME`) and replay with
/// production git. Kept in the lib so the bin stays thin, the same
/// discipline as `prompt::inbox::cli_run`.
pub fn replay_cli(archive: &Path) -> Result<PathBuf, ArchiveError> {
    let roots = crate::harness_root::resolve().map_err(io::Error::other)?;
    replay(
        archive,
        &roots.data.join("replays"),
        &crate::template::RealGit::new(),
    )
}

/// Enumerate the subtree's branches: `agents/<agent_id>` and every
/// `agents/<agent_id>-*` hyphen-descendant (§2.3), via
/// `git branch --list` against the bare repo.git.
fn subtree_refs(
    repo: &Path,
    agent_id: &str,
    git: &dyn GitRunner,
) -> Result<Vec<String>, ArchiveError> {
    let subtree_root = workspace::agent_ref(agent_id);
    let descendants = format!("{subtree_root}-*");
    let out = git
        .run_capture(
            repo,
            &[
                "branch",
                "--list",
                "--format=%(refname:short)",
                subtree_root.as_str(),
                &descendants,
            ],
        )
        .map_err(|source| ArchiveError::Git {
            op: "branch --list",
            source,
        })?;
    Ok(out
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty())
        .map(str::to_owned)
        .collect())
}

/// The agent ids a bundle carries (the `refs/heads/agents/` prefix
/// stripped, §2.3), via `git bundle list-heads`.
fn bundle_heads(
    dir: &Path,
    bundle_path: &Path,
    git: &dyn GitRunner,
) -> Result<Vec<String>, ArchiveError> {
    let bundle_str = bundle_path.to_string_lossy().into_owned();
    let out = git
        .run_capture(dir, &["bundle", "list-heads", &bundle_str])
        .map_err(|source| ArchiveError::Git {
            op: "bundle list-heads",
            source,
        })?;
    Ok(out
        .lines()
        .filter_map(|l| {
            let refname = l.split_whitespace().nth(1)?;
            let short = refname.strip_prefix("refs/heads/").unwrap_or(refname);
            Some(
                short
                    .strip_prefix(workspace::AGENT_REF_PREFIX)
                    .unwrap_or(short)
                    .to_owned(),
            )
        })
        .collect())
}

/// The subtree root among `heads` (agent ids): the shortest, of which
/// every other is a hyphen-descendant (§2.3). Empty is
/// [`ArchiveError::EmptyBundle`]; heads sharing no such root is
/// [`ArchiveError::MalformedBundle`].
fn primary_head(heads: &[String]) -> Result<&str, ArchiveError> {
    let primary = heads
        .iter()
        .min_by_key(|h| h.len())
        .ok_or(ArchiveError::EmptyBundle)?;
    let prefix = format!("{primary}-");
    for h in heads {
        if h != primary && !h.starts_with(&prefix) {
            return Err(ArchiveError::MalformedBundle(heads.to_vec()));
        }
    }
    Ok(primary)
}

/// Run a fire-and-forget git op, tagging failures with `op`.
fn run(
    git: &dyn GitRunner,
    dest: &Path,
    args: &[&str],
    op: &'static str,
) -> Result<(), ArchiveError> {
    git.run(dest, args)
        .map_err(|source| ArchiveError::Git { op, source })
}

/// Copy each entry of `src_root` named `<agent_id>` or `<agent_id>-*`
/// into `dst_root`. A missing `src_root` (no slice for this run) is a
/// clean no-op; `dst_root` is created only when something matches.
fn copy_matching(src_root: &Path, dst_root: &Path, agent_id: &str) -> io::Result<()> {
    if !src_root.is_dir() {
        return Ok(());
    }
    let prefix = format!("{agent_id}-");
    for entry in fs::read_dir(src_root)? {
        let entry = entry?;
        let name = entry.file_name();
        let name_s = name.to_string_lossy();
        if *name_s == *agent_id || name_s.starts_with(&prefix) {
            copy_entry(&entry.path(), &dst_root.join(&name))?;
        }
    }
    Ok(())
}

/// Recursively copy a directory tree.
fn copy_dir_all(src: &Path, dst: &Path) -> io::Result<()> {
    fs::create_dir_all(dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        copy_entry(&entry.path(), &dst.join(entry.file_name()))?;
    }
    Ok(())
}

/// Copy one filesystem entry — recursing for directories, `fs::copy` for
/// files.
fn copy_entry(src: &Path, dst: &Path) -> io::Result<()> {
    if src.is_dir() {
        copy_dir_all(src, dst)
    } else {
        fs::copy(src, dst).map(|_| ())
    }
}
