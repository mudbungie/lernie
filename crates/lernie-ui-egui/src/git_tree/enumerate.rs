//! Commit-node and in-flight-branch construction. Bridges raw git
//! output (see [`super::cmd`]) into the view-model's [`CommitNode`]
//! and [`ConversationBranch`] shapes by applying the v0.3 detection
//! rules in [`super::detect`].

use super::cmd::{
    LogEntry, files_changed, for_each_ref_unmerged, show_blob, walk_branch_steps,
    walk_merge_step_commits,
};
use super::detect::{extract_request_preview, v03_conv_id_from_path};
use super::{CommitNode, ConversationBranch, GitTreeError};
use std::path::Path;

pub(super) fn build_node(repo: &Path, entry: LogEntry) -> Result<CommitNode, GitTreeError> {
    let LogEntry {
        oid,
        timestamp,
        parent_count,
    } = entry;
    let short_oid = oid.get(..8).unwrap_or(&oid).to_string();
    let files = files_changed(repo, &oid, parent_count)?;
    let is_merge = parent_count >= 2;

    if let Some(id) = files.iter().find_map(|p| v03_conv_id_from_path(p)) {
        let step_path = format!("steps/{id}/001/request.json");
        let preview = show_blob(repo, &oid, &step_path)
            .ok()
            .and_then(|bytes| extract_request_preview(&bytes));
        let steps = if is_merge {
            walk_merge_step_commits(repo, &oid)?
        } else {
            Vec::new()
        };
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

pub(super) fn enumerate_in_flight(repo: &Path) -> Result<Vec<ConversationBranch>, GitTreeError> {
    let out = for_each_ref_unmerged(repo)?;
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
        let steps = walk_branch_steps(repo, &branch_name)?;
        let preview = preview_from_branch_tip(repo, &tip_oid, &conv_id);
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

fn preview_from_branch_tip(repo: &Path, tip_oid: &str, conv_id: &str) -> Option<String> {
    let path = format!("steps/{conv_id}/001/request.json");
    let bytes = show_blob(repo, tip_oid, &path).ok()?;
    extract_request_preview(&bytes)
}
