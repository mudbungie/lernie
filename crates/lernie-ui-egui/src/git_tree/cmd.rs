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

/// Raw `git log --format='%H %ct %P%x00%s'` row, parsed. The trunk is
/// the config lineage (§2.2), so `subject` labels a config commit.
#[derive(Debug)]
pub(super) struct LogEntry {
    pub(super) oid: String,
    pub(super) timestamp: i64,
    pub(super) parent_count: usize,
    pub(super) subject: String,
}

pub(super) fn git_log_first_parent(repo: &Path) -> Result<Vec<LogEntry>, GitTreeError> {
    // `--first-parent` keeps conversation branches off the trunk log;
    // step commits are rendered nested under their merge node instead.
    // `\x00` separates the parent list from the subject so a subject
    // containing spaces parses unambiguously.
    let out = git(
        repo,
        &[
            "log",
            "--first-parent",
            "--format=%H %ct %P%x00%s",
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
        let (head, subject) = line
            .split_once('\x00')
            .ok_or_else(|| GitTreeError::LogFormat(line.to_string()))?;
        let mut parts = head.splitn(3, ' ');
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
            subject: subject.to_string(),
        });
    }
    Ok(result)
}

pub(super) fn walk_branch_steps(
    repo: &Path,
    branch: &str,
) -> Result<Vec<StepCommit>, GitTreeError> {
    // Commits on the agent branch past every config lineage (§2.2 —
    // there is no `main`; the fork point is a config commit). `\x00`
    // separates the timestamp from the subject so a subject containing
    // spaces parses unambiguously — the subject surfaces delivery and
    // work-product-transfer commits (§2.11, §2.6, §7.1).
    let out = git(
        repo,
        &[
            "log",
            "--reverse",
            "--first-parent",
            "--format=%H %ct%x00%s",
            branch,
            "--not",
            "--branches=config/*",
        ],
    )?;
    parse_step_commits(&out)
}

pub(super) fn parse_step_commits(stdout: &[u8]) -> Result<Vec<StepCommit>, GitTreeError> {
    let text = String::from_utf8_lossy(stdout);
    let mut result = Vec::new();
    for line in text.lines() {
        let (head, subject) = line
            .split_once('\x00')
            .ok_or_else(|| GitTreeError::LogFormat(line.to_string()))?;
        let (oid, ts) = head
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
            subject: subject.to_string(),
        });
    }
    Ok(result)
}

/// Agent branches: every ref under `refs/heads/agents/` (ARCH §2.3 —
/// the prefix is the kind; agents never merge anywhere, §2.6).
pub(super) fn for_each_ref_agents(repo: &Path) -> Result<Vec<u8>, GitTreeError> {
    git(
        repo,
        &[
            "for-each-ref",
            "--format=%(refname:short) %(objectname) %(committerdate:unix)",
            "refs/heads/agents/",
        ],
    )
}

/// Every ref under a `refs/lernie/<kind>/` namespace, full refnames
/// (ARCH §2.6 declined-transfer, §6 budget-exhausted). The caller strips
/// `prefix` to recover the agent ids ([`super::marks`]).
pub(super) fn for_each_ref_under(repo: &Path, prefix: &str) -> Result<Vec<u8>, GitTreeError> {
    git(repo, &["for-each-ref", "--format=%(refname)", prefix])
}
