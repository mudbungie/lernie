//! Commit-node and agent-branch construction. Bridges raw git output
//! (see [`super::cmd`]) into the view-model's [`CommitNode`] and
//! [`ConversationBranch`] shapes.
//!
//! The trunk is the config lineage (§2.2); agents are the `agents/*`
//! refs (§2.3), never merged anywhere (§2.6). The user-message preview
//! is read from disk at `<workspace>/steps/<agent-id>/001/request.json`
//! (§2.3, "Step records are not committed to git").

use super::cmd::{LogEntry, for_each_ref_agents, walk_branch_steps};
use super::detect::extract_request_preview;
use super::fd_probe::WriterProbe;
use super::state::classify_unmerged;
use super::streaming::streaming_text_from_disk;
use super::tools::tool_calls_from_disk;
use super::{CommitNode, ConversationBranch, GitTreeError, STEPS_DIR};
use std::path::Path;

pub(super) fn build_node(entry: LogEntry) -> CommitNode {
    let LogEntry {
        oid,
        timestamp,
        parent_count: _,
        subject,
    } = entry;
    let short_oid = oid.get(..8).unwrap_or(&oid).to_string();
    CommitNode {
        oid,
        short_oid,
        timestamp_unix: timestamp,
        subject,
    }
}

pub(super) fn enumerate_in_flight(
    conv_repo: &Path,
    git_dir: &Path,
    probe: &dyn WriterProbe,
) -> Result<Vec<ConversationBranch>, GitTreeError> {
    let out = for_each_ref_agents(git_dir)?;
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
        // Agent refs are `agents/<id>` (§2.3); the id is the identity
        // everywhere else (steps/, inbox/, worktree dir).
        let conv_id = branch_name
            .strip_prefix("agents/")
            .unwrap_or(&branch_name)
            .to_string();
        let tip_short_oid = tip_oid.get(..8).unwrap_or(&tip_oid).to_string();
        let steps = walk_branch_steps(git_dir, &branch_name)?;
        let preview = preview_from_disk(conv_repo, &conv_id);
        let streaming_text = streaming_text_from_disk(conv_repo, &conv_id);
        let tool_calls = tool_calls_from_disk(conv_repo, &conv_id);
        let state = classify_unmerged(conv_repo, &conv_id, probe);
        branches.push(ConversationBranch {
            branch_name,
            conv_id,
            tip_oid,
            tip_short_oid,
            tip_timestamp_unix: tip_ts,
            steps,
            preview,
            streaming_text,
            tool_calls,
            state,
        });
    }
    Ok(branches)
}

fn preview_from_disk(conv_repo: &Path, conv_id: &str) -> Option<String> {
    let path = conv_repo
        .join(STEPS_DIR)
        .join(conv_id)
        .join("001")
        .join("request.json");
    let bytes = std::fs::read(&path).ok()?;
    extract_request_preview(&bytes)
}
