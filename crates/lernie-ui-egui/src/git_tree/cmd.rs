//! Git CLI wrapper and log/diff parsing for the git-tree view-model.
//!
//! The UI reads git state exclusively through the CLI (no libgit2), so
//! all subprocess invocations route through [`git`] here. That gives us
//! one place to scrub inherited env vars (GIT_DIR and friends) that
//! would otherwise redirect a child `git` back to the outer repo when
//! the UI is launched from a git-hook context.

use super::{GitTreeError, StepCommit};
use std::path::Path;
use std::process::Command;

/// Env vars that, when inherited, override `-C <repo>` and break
/// fixture/test isolation — cleared on every spawn.
const INHERITED_GIT_ENV: &[&str] = &[
    "GIT_DIR",
    "GIT_WORK_TREE",
    "GIT_INDEX_FILE",
    "GIT_OBJECT_DIRECTORY",
    "GIT_PREFIX",
    "GIT_COMMON_DIR",
    "GIT_ALTERNATE_OBJECT_DIRECTORIES",
];

pub(super) fn git(repo: &Path, args: &[&str]) -> Result<Vec<u8>, GitTreeError> {
    let mut cmd = Command::new("git");
    for var in INHERITED_GIT_ENV {
        cmd.env_remove(var);
    }
    let output = cmd.arg("-C").arg(repo).args(args).output()?;
    if !output.status.success() {
        return Err(GitTreeError::Git {
            command: args.join(" "),
            repo: repo.to_path_buf(),
            stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
        });
    }
    Ok(output.stdout)
}

/// Raw `git log --format='%H %ct %P'` row, parsed.
///
/// Keeping the parent count here avoids a second `git log` per commit
/// when we need to choose between root-diff and first-parent-diff in
/// [`files_changed`].
#[derive(Debug)]
pub(super) struct LogEntry {
    pub(super) oid: String,
    pub(super) timestamp: i64,
    pub(super) parent_count: usize,
}

pub(super) fn git_log_first_parent(repo: &Path) -> Result<Vec<LogEntry>, GitTreeError> {
    // `--first-parent` keeps v0.2 exchange branches off the trunk log;
    // step commits are rendered nested under their merge node instead.
    // For v0.1-shape linear history it is a no-op.
    let out = git(
        repo,
        &[
            "log",
            "--first-parent",
            "--format=%H %ct %P",
            "--reverse",
            "HEAD",
        ],
    )?;
    parse_log(&out)
}

pub(super) fn parse_log(stdout: &[u8]) -> Result<Vec<LogEntry>, GitTreeError> {
    let text = String::from_utf8_lossy(stdout);
    let mut result = Vec::new();
    for line in text.lines() {
        let mut parts = line.splitn(3, ' ');
        let oid = parts
            .next()
            .ok_or_else(|| GitTreeError::LogFormat(line.to_string()))?
            .to_string();
        let ts_str = parts
            .next()
            .ok_or_else(|| GitTreeError::LogFormat(line.to_string()))?;
        let ts: i64 = ts_str
            .parse()
            .map_err(|_| GitTreeError::LogFormat(line.to_string()))?;
        // Parents column is empty for a root commit; any other value is
        // whitespace-separated parent shas.
        let parent_count = parts
            .next()
            .map(|p| p.split_whitespace().count())
            .unwrap_or(0);
        result.push(LogEntry {
            oid,
            timestamp: ts,
            parent_count,
        });
    }
    Ok(result)
}

/// Files this commit introduces versus its first parent (or versus the
/// empty tree, for a root commit). For a merge commit this is the set
/// of paths that the merge brought in on top of `main`'s prior state,
/// which is what we want for detecting the exchange files added by a
/// `--no-ff` merge.
pub(super) fn files_changed(
    repo: &Path,
    oid: &str,
    parent_count: usize,
) -> Result<Vec<String>, GitTreeError> {
    let out = if parent_count == 0 {
        git(
            repo,
            &[
                "diff-tree",
                "--no-commit-id",
                "--name-only",
                "-r",
                "--root",
                oid,
            ],
        )?
    } else {
        let first_parent = format!("{oid}^1");
        git(
            repo,
            &[
                "diff-tree",
                "--no-commit-id",
                "--name-only",
                "-r",
                &first_parent,
                oid,
            ],
        )?
    };
    Ok(String::from_utf8_lossy(&out)
        .lines()
        .map(|s| s.to_string())
        .collect())
}

/// Commits on the exchange branch reachable from the merge's second
/// parent but not from its first parent, along first-parent only —
/// i.e. the snapshot commit, response commit, and compactor merge
/// commit on the exchange branch itself, excluding the compactor
/// invocation's internal commits.
pub(super) fn walk_merge_step_commits(
    repo: &Path,
    merge_oid: &str,
) -> Result<Vec<StepCommit>, GitTreeError> {
    let second_parent = format!("{merge_oid}^2");
    let exclude_first = format!("^{merge_oid}^1");
    let out = git(
        repo,
        &[
            "log",
            "--reverse",
            "--first-parent",
            "--format=%H %ct",
            &second_parent,
            &exclude_first,
        ],
    )?;
    parse_step_commits(&out)
}

pub(super) fn walk_branch_steps(
    repo: &Path,
    branch: &str,
) -> Result<Vec<StepCommit>, GitTreeError> {
    let out = git(
        repo,
        &[
            "log",
            "--reverse",
            "--first-parent",
            "--format=%H %ct",
            branch,
            "^main",
        ],
    )?;
    parse_step_commits(&out)
}

pub(super) fn parse_step_commits(stdout: &[u8]) -> Result<Vec<StepCommit>, GitTreeError> {
    let text = String::from_utf8_lossy(stdout);
    let mut result = Vec::new();
    for line in text.lines() {
        let (oid, ts) = line
            .split_once(' ')
            .ok_or_else(|| GitTreeError::LogFormat(line.to_string()))?;
        let ts: i64 = ts
            .parse()
            .map_err(|_| GitTreeError::LogFormat(line.to_string()))?;
        let short_oid = oid.get(..8).unwrap_or(oid).to_string();
        result.push(StepCommit {
            oid: oid.to_string(),
            short_oid,
            timestamp_unix: ts,
        });
    }
    Ok(result)
}

pub(super) fn for_each_ref_unmerged_ex(repo: &Path) -> Result<Vec<u8>, GitTreeError> {
    git(
        repo,
        &[
            "for-each-ref",
            "--no-merged=main",
            "--format=%(refname:short) %(objectname) %(committerdate:unix)",
            "refs/heads/ex/",
        ],
    )
}

pub(super) fn show_blob(repo: &Path, oid: &str, path: &str) -> Result<Vec<u8>, GitTreeError> {
    git(repo, &["show", &format!("{oid}:{path}")])
}
