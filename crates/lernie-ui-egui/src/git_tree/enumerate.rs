//! Commit-node and in-flight-branch construction. Bridges raw git
//! output (see [`super::cmd`]) into the view-model's [`CommitNode`]
//! and [`ExchangeBranch`] shapes by applying the detection rules in
//! [`super::detect`].

use super::cmd::{
    LogEntry, files_changed, for_each_ref_unmerged_ex, show_blob, walk_branch_steps,
    walk_merge_step_commits,
};
use super::detect::{
    exchange_id_from_branch, exchange_id_from_v01_path, extract_v01_preview, extract_v02_preview,
    is_v01_exchange_path, v02_exchange_id_from_path,
};
use super::{CommitNode, ExchangeBranch, GitTreeError};
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

    if let Some(id) = files.iter().find_map(|p| v02_exchange_id_from_path(p)) {
        // v0.2 merge commit — preview from the first step's request.json.
        let step_path = format!("exchanges/{id}/steps/001/request.json");
        let preview = show_blob(repo, &oid, &step_path)
            .ok()
            .and_then(|bytes| extract_v02_preview(&bytes));
        let steps = if is_merge {
            walk_merge_step_commits(repo, &oid)?
        } else {
            Vec::new()
        };
        return Ok(CommitNode {
            oid,
            short_oid,
            timestamp_unix: timestamp,
            exchange_id: Some(id.to_string()),
            preview,
            steps,
        });
    }

    if let Some(path) = files.iter().find(|p| is_v01_exchange_path(p)) {
        let id = exchange_id_from_v01_path(path);
        let content = show_blob(repo, &oid, path)?;
        let preview = extract_v01_preview(&content);
        return Ok(CommitNode {
            oid,
            short_oid,
            timestamp_unix: timestamp,
            exchange_id: Some(id),
            preview,
            steps: Vec::new(),
        });
    }

    Ok(CommitNode {
        oid,
        short_oid,
        timestamp_unix: timestamp,
        exchange_id: None,
        preview: None,
        steps: Vec::new(),
    })
}

pub(super) fn enumerate_in_flight(repo: &Path) -> Result<Vec<ExchangeBranch>, GitTreeError> {
    let out = for_each_ref_unmerged_ex(repo)?;
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
        let exchange_id = exchange_id_from_branch(&branch_name);
        let tip_short_oid = tip_oid.get(..8).unwrap_or(&tip_oid).to_string();
        let steps = walk_branch_steps(repo, &branch_name)?;
        let preview = preview_from_branch_tip(repo, &tip_oid, &exchange_id);
        branches.push(ExchangeBranch {
            branch_name,
            exchange_id,
            tip_oid,
            tip_short_oid,
            tip_timestamp_unix: tip_ts,
            steps,
            preview,
        });
    }
    Ok(branches)
}

fn preview_from_branch_tip(repo: &Path, tip_oid: &str, exchange_id: &str) -> Option<String> {
    let path = format!("exchanges/{exchange_id}/steps/001/request.json");
    let bytes = show_blob(repo, tip_oid, &path).ok()?;
    extract_v02_preview(&bytes)
}
