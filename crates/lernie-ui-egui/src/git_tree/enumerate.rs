//! Commit-node and agent construction. Bridges raw git output (see
//! [`super::cmd`]) into the view-model's [`CommitNode`] and [`Agent`]
//! shapes.
//!
//! The trunk is the config lineage (§2.2); agents are the `agents/*` refs
//! (§2.3), never merged anywhere (§2.6). Per-agent disk reads (preview,
//! streaming text, tool calls, pending-message count) come from the
//! workspace root (`steps/<agent-id>/…`, `inbox/<agent-id>/`, §2.2/§2.11);
//! the §3.5 state and the two ref-derived marks are classified here.

use super::cmd::{LogEntry, for_each_ref_agents, walk_branch_steps};
use super::detect::extract_request_preview;
use super::fd_probe::WriterProbe;
use super::lock_probe::LockProbe;
use super::marks::Marks;
use super::state::classify;
use super::streaming::streaming_text_from_disk;
use super::tools::tool_calls_from_disk;
use super::{Agent, CommitNode, GitTreeError, INBOX_DIR, STEPS_DIR};
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

pub(super) fn enumerate_agents(
    workspace: &Path,
    git_dir: &Path,
    lock: &dyn LockProbe,
    writer: &dyn WriterProbe,
) -> Result<Vec<Agent>, GitTreeError> {
    let out = for_each_ref_agents(git_dir)?;
    let text = String::from_utf8_lossy(&out);
    let marks = Marks::from_repo(git_dir)?;
    let mut agents = Vec::new();
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
        // everywhere else (steps/, inbox/, worktree dir, descent).
        let agent_id = branch_name
            .strip_prefix("agents/")
            .unwrap_or(&branch_name)
            .to_string();
        let tip_short_oid = tip_oid.get(..8).unwrap_or(&tip_oid).to_string();
        let steps = walk_branch_steps(git_dir, &branch_name)?;
        agents.push(Agent {
            preview: preview_from_disk(workspace, &agent_id),
            streaming_text: streaming_text_from_disk(workspace, &agent_id),
            tool_calls: tool_calls_from_disk(workspace, &agent_id),
            state: classify(workspace, &agent_id, lock, writer),
            pending_messages: pending_from_disk(workspace, &agent_id),
            declined_transfer: marks.declined_transfer(&agent_id),
            budget_exhausted: marks.budget_exhausted(&agent_id),
            branch_name,
            agent_id,
            tip_oid,
            tip_short_oid,
            tip_timestamp_unix: tip_ts,
            steps,
        });
    }
    Ok(agents)
}

fn preview_from_disk(workspace: &Path, agent_id: &str) -> Option<String> {
    let path = workspace
        .join(STEPS_DIR)
        .join(agent_id)
        .join("001")
        .join("request.json");
    let bytes = std::fs::read(&path).ok()?;
    extract_request_preview(&bytes)
}

/// Count of pending (undelivered) messages in the agent's inbox
/// (`<workspace>/inbox/<agent-id>/`, §2.11). Deposits are
/// `<sender>-<NNN>.md`; the atomic-rename temp files are dotfiles
/// (`.<name>.tmp`), so counting `*.md` entries excludes them. A missing
/// inbox directory is zero.
fn pending_from_disk(workspace: &Path, agent_id: &str) -> usize {
    let inbox = workspace.join(INBOX_DIR).join(agent_id);
    let Ok(entries) = std::fs::read_dir(&inbox) else {
        return 0;
    };
    entries
        .flatten()
        .filter(|e| e.file_name().to_str().is_some_and(|n| n.ends_with(".md")))
        .count()
}
