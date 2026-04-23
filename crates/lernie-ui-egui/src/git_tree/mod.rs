//! Git-tree view-model and egui widget.
//!
//! `GitTree::from_repo(repo)` walks the linear commit history on `HEAD`
//! and produces a view-model describing each commit — oid, timestamp,
//! and (for v0.1-shape repos, per ARCH §12) the `exchanges/<ts>-<id>.json`
//! file the commit introduced plus a truncated preview of its user
//! message. The view-model is a pure function of the repo's current ref
//! state and holds no egui dependency, so a future `lernie-ui-web` crate
//! can render the same tree from the web.
//!
//! Git access is via the `git` CLI (a hard dep of lernie itself, per
//! ARCH §2.2) — no libgit2 native build step is required.
//!
//! v0.2 adds branches; the view-model extends by walking per-ref heads
//! and threading parents, without any change to the rendering layer.

use std::path::{Path, PathBuf};
use std::process::Command;

const PREVIEW_MAX: usize = 80;

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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GitTree {
    pub commits: Vec<CommitNode>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommitNode {
    pub oid: String,
    pub short_oid: String,
    pub timestamp_unix: i64,
    pub exchange_id: Option<String>,
    pub preview: Option<String>,
}

impl GitTree {
    pub fn from_repo(repo: &Path) -> Result<Self, GitTreeError> {
        let log = git_log(repo)?;
        let mut commits = Vec::with_capacity(log.len());
        for (oid, ts) in log {
            commits.push(build_node(repo, oid, ts)?);
        }
        Ok(Self { commits })
    }
}

fn git(repo: &Path, args: &[&str]) -> Result<Vec<u8>, GitTreeError> {
    let output = Command::new("git").arg("-C").arg(repo).args(args).output()?;
    if !output.status.success() {
        return Err(GitTreeError::Git {
            command: args.join(" "),
            repo: repo.to_path_buf(),
            stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
        });
    }
    Ok(output.stdout)
}

fn git_log(repo: &Path) -> Result<Vec<(String, i64)>, GitTreeError> {
    let out = git(repo, &["log", "--format=%H %ct", "--reverse", "HEAD"])?;
    parse_log(&out)
}

fn parse_log(stdout: &[u8]) -> Result<Vec<(String, i64)>, GitTreeError> {
    let text = String::from_utf8_lossy(stdout);
    let mut result = Vec::new();
    for line in text.lines() {
        let (oid, ts) = line
            .split_once(' ')
            .ok_or_else(|| GitTreeError::LogFormat(line.to_string()))?;
        let ts: i64 = ts
            .parse()
            .map_err(|_| GitTreeError::LogFormat(line.to_string()))?;
        result.push((oid.to_string(), ts));
    }
    Ok(result)
}

fn build_node(repo: &Path, oid: String, ts: i64) -> Result<CommitNode, GitTreeError> {
    let short_oid = oid.get(..8).unwrap_or(&oid).to_string();
    let (exchange_id, preview) = exchange_from_commit(repo, &oid)?;
    Ok(CommitNode {
        oid,
        short_oid,
        timestamp_unix: ts,
        exchange_id,
        preview,
    })
}

fn exchange_from_commit(
    repo: &Path,
    oid: &str,
) -> Result<(Option<String>, Option<String>), GitTreeError> {
    let files = files_changed(repo, oid)?;
    let Some(path) = files
        .into_iter()
        .find(|p| is_v01_exchange_path(p))
    else {
        return Ok((None, None));
    };
    let id = exchange_id_from_path(&path);
    let content = git(repo, &["show", &format!("{oid}:{path}")])?;
    let preview = extract_preview(&content);
    Ok((Some(id), preview))
}

fn files_changed(repo: &Path, oid: &str) -> Result<Vec<String>, GitTreeError> {
    let out = git(
        repo,
        &[
            "diff-tree",
            "--no-commit-id",
            "--name-only",
            "-r",
            "--root",
            oid,
        ],
    )?;
    Ok(String::from_utf8_lossy(&out)
        .lines()
        .map(|s| s.to_string())
        .collect())
}

fn is_v01_exchange_path(path: &str) -> bool {
    path.starts_with("exchanges/") && path.ends_with(".json") && !path[10..].contains('/')
}

fn exchange_id_from_path(path: &str) -> String {
    path.strip_prefix("exchanges/")
        .and_then(|s| s.strip_suffix(".json"))
        .map(|s| s.to_string())
        .unwrap_or_else(|| path.to_string())
}

fn extract_preview(json_bytes: &[u8]) -> Option<String> {
    let value: serde_json::Value = serde_json::from_slice(json_bytes).ok()?;
    let msg = value.get("user_message")?.as_str()?;
    Some(truncate_preview(msg))
}

fn truncate_preview(s: &str) -> String {
    let collapsed: String = s
        .chars()
        .map(|c| if c.is_whitespace() { ' ' } else { c })
        .collect();
    let trimmed = collapsed.trim();
    if trimmed.chars().count() <= PREVIEW_MAX {
        return trimmed.to_string();
    }
    let head: String = trimmed.chars().take(PREVIEW_MAX - 1).collect();
    format!("{head}…")
}

/// egui widget that renders a `GitTree` as a vertical list of commits.
/// Thin wrapper — all structure lives in the view-model.
pub fn render(ui: &mut egui::Ui, tree: &GitTree) {
    if tree.commits.is_empty() {
        ui.label("(no commits yet)");
        return;
    }
    for commit in &tree.commits {
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
}

#[cfg(test)]
mod tests;
