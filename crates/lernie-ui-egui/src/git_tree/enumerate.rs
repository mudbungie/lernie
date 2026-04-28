//! Commit-node and in-flight-branch construction. Bridges raw git
//! output (see [`super::cmd`]) into the view-model's [`CommitNode`]
//! and [`ConversationBranch`] shapes.
//!
//! v0.3.1 detection is by branch name (ARCH §2.3): trunk merge commits
//! carry the conv-id in their default `Merge branch '<name>'` subject,
//! and in-flight branch refs *are* the conv-id verbatim. The user-
//! message preview is read from disk at
//! `<conv-repo>/steps/<conv-id>/001/request.json`; step records are
//! filesystem-only post-P2 (§2.3, "Step records are not committed to
//! git"), so a `git show` against the merge commit would not find them.

use super::cmd::{LogEntry, for_each_ref_unmerged, walk_branch_steps, walk_merge_step_commits};
use super::detect::{extract_request_preview, parse_merge_subject};
use super::{CommitNode, ConversationBranch, GitTreeError};
use std::path::Path;

pub(super) fn build_node(
    conv_repo: &Path,
    git_dir: &Path,
    entry: LogEntry,
) -> Result<CommitNode, GitTreeError> {
    let LogEntry {
        oid,
        timestamp,
        parent_count,
        subject,
    } = entry;
    let short_oid = oid.get(..8).unwrap_or(&oid).to_string();
    let is_merge = parent_count >= 2;

    if let Some(id) = is_merge.then(|| parse_merge_subject(&subject)).flatten() {
        let preview = preview_from_disk(conv_repo, id);
        let steps = walk_merge_step_commits(git_dir, &oid)?;
        return Ok(CommitNode {
            oid,
            short_oid,
            timestamp_unix: timestamp,
            conv_id: Some(id.to_string()),
            preview,
            steps,
        });
    }

    Ok(CommitNode {
        oid,
        short_oid,
        timestamp_unix: timestamp,
        conv_id: None,
        preview: None,
        steps: Vec::new(),
    })
}

pub(super) fn enumerate_in_flight(
    conv_repo: &Path,
    git_dir: &Path,
) -> Result<Vec<ConversationBranch>, GitTreeError> {
    let out = for_each_ref_unmerged(git_dir)?;
    let text = String::from_utf8_lossy(&out);
    let mut branches = Vec::new();
    for line in text.lines() {
        if line.is_empty() {
            continue;
        }
        let mut parts = line.splitn(3, ' ');
        let branch_name = parts
            .next()
            .ok_or_else(|| GitTreeError::LogFormat(line.to_string()))?
            .to_string();
        let tip_oid = parts
            .next()
            .ok_or_else(|| GitTreeError::LogFormat(line.to_string()))?
            .to_string();
        let ts_str = parts
            .next()
            .ok_or_else(|| GitTreeError::LogFormat(line.to_string()))?;
        let tip_ts: i64 = ts_str
            .parse()
            .map_err(|_| GitTreeError::LogFormat(line.to_string()))?;
        // v0.3 branch names are conv-ids verbatim (ARCH §2.3).
        let conv_id = branch_name.clone();
        let tip_short_oid = tip_oid.get(..8).unwrap_or(&tip_oid).to_string();
        let steps = walk_branch_steps(git_dir, &branch_name)?;
        let preview = preview_from_disk(conv_repo, &conv_id);
        branches.push(ConversationBranch {
            branch_name,
            conv_id,
            tip_oid,
            tip_short_oid,
            tip_timestamp_unix: tip_ts,
            steps,
            preview,
        });
    }
    Ok(branches)
}

fn preview_from_disk(conv_repo: &Path, conv_id: &str) -> Option<String> {
    let path = conv_repo
        .join("steps")
        .join(conv_id)
        .join("001")
        .join("request.json");
    let bytes = std::fs::read(&path).ok()?;
    extract_request_preview(&bytes)
}
