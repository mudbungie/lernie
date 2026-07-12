//! Committing transcript entries to the branch (ARCH §2.3, §3.3).
//!
//! The **transcript** is the branch-scoped sequence under `messages/`:
//! each step's assistant output and each tool call's result is one
//! immutable entry, committed by the executor as it lands (§2.3). An
//! entry's filename is `NNN-<origin>.json` — `NNN` the branch's single
//! zero-padded transcript counter and `<origin>` a reserved token
//! (`assistant` / `tool`, §2.3). Order lives in the filename and nowhere
//! else, so the counter is *derived* — [`next_seq`] reads the `messages/`
//! listing and returns max-present-plus-one — never stored (PRINCIPLES
//! single source of truth).
//!
//! Each entry file is a JSON array of brazen's canonical [`Content`]
//! blocks (an assistant entry's streamed blocks; a tool entry's single
//! `tool_result` block), so it composes verbatim as one wire message
//! (§2.3) — what makes replay bit-identical rather than a lossy
//! re-rendering.

use crate::prompt::Error;
use crate::template::GitRunner;
use brazen::Content;
use std::path::Path;

/// Branch-scoped transcript directory (ARCH §2.3 — `messages/NNN-…`).
pub(super) const MESSAGES_DIR: &str = "messages";
/// Zero-pad width of the transcript counter, matching the step-record
/// convention (`steps/<id>/NNN`, `summary/NNN`).
const SEQ_WIDTH: usize = 3;
/// Reserved origin token for a step's assistant output (§2.3).
const ASSISTANT_ORIGIN: &str = "assistant";
/// Reserved origin token for a tool call's result (§2.3).
const TOOL_ORIGIN: &str = "tool";

/// The branch's next transcript counter: max of the `NNN` prefixes
/// present under `<worktree>/messages/`, plus one (§2.3). An absent or
/// empty directory yields `1` — the general path with empty inputs, not
/// a bootstrap special case.
pub(super) fn next_seq(worktree: &Path) -> Result<u32, Error> {
    let dir = worktree.join(MESSAGES_DIR);
    let entries = match std::fs::read_dir(&dir) {
        Ok(rd) => rd,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(1),
        Err(e) => return Err(Error::Io(e)),
    };
    let mut max = 0u32;
    for entry in entries {
        let name = entry.map_err(Error::Io)?.file_name();
        if let Some(seq) = name
            .to_string_lossy()
            .split('-')
            .next()
            .and_then(|p| p.parse::<u32>().ok())
        {
            max = max.max(seq);
        }
    }
    Ok(max + 1)
}

/// Commit a delivered message as `messages/NNN-<sender>.md` at the
/// branch's next counter (§2.3 *Origins*, §2.11). The initial user
/// message is delivered this way — the pre-inbox stand-in for the
/// step-boundary drain (§2.11) — and composes as user-role content
/// (§5.3). `sender` is the origin token (`user`, or an agent id).
pub(super) fn commit_message(
    worktree: &Path,
    conv_id: &str,
    sender: &str,
    body: &str,
    git: &dyn GitRunner,
) -> Result<(), Error> {
    let seq = next_seq(worktree)?;
    let rel = format!("{MESSAGES_DIR}/{seq:0w$}-{sender}.md", w = SEQ_WIDTH);
    let dest = worktree.join(&rel);
    std::fs::create_dir_all(dest.parent().expect("messages/ has a parent"))?;
    std::fs::write(&dest, body)?;
    commit_entry(worktree, conv_id, seq, &rel, sender, git)
}

/// Commit a step's assistant output: seal-and-rename — the sealed
/// staging file (§2.3 *The transcript writer*) *leaves* by rename into
/// `messages/NNN-assistant.json` at the branch's next counter, then a
/// commit lands it. `NNN` is evaluated here, inside the executor's
/// serialized commit section (§2.3). Returns the committed canonical
/// blocks — read back from the transcript entry (its one content home,
/// §2.3), never from any `steps/` record — so the step loop can run this
/// step's `tool_use` calls without a second content fold.
pub(super) fn commit_assistant(
    worktree: &Path,
    conv_id: &str,
    staging_path: &Path,
    git: &dyn GitRunner,
) -> Result<Vec<Content>, Error> {
    let seq = next_seq(worktree)?;
    let rel = entry_rel(seq, ASSISTANT_ORIGIN);
    let dest = worktree.join(&rel);
    std::fs::create_dir_all(dest.parent().expect("messages/ has a parent"))?;
    std::fs::rename(staging_path, &dest)?;
    commit_entry(worktree, conv_id, seq, &rel, ASSISTANT_ORIGIN, git)?;
    let bytes = std::fs::read(&dest)?;
    // Harness-sealed staging, so always a valid canonical array (§2.3).
    Ok(serde_json::from_slice(&bytes).expect("assistant entry is a canonical Content array"))
}

/// Commit one resolved tool call's canonical `tool_result` block as
/// `messages/NNN-tool.json` (§3.3 "Wire `tool_result` framing is
/// transcript-backed"). The counter read happens inside the sibling
/// tool serialization the caller already imposes (§3.3).
pub(super) fn commit_tool(
    worktree: &Path,
    conv_id: &str,
    tool_result: &Content,
    git: &dyn GitRunner,
) -> Result<(), Error> {
    let seq = next_seq(worktree)?;
    let rel = entry_rel(seq, TOOL_ORIGIN);
    let dest = worktree.join(&rel);
    std::fs::create_dir_all(dest.parent().expect("messages/ has a parent"))?;
    let bytes = serde_json::to_vec(std::slice::from_ref(tool_result)).expect("Content serializes");
    std::fs::write(&dest, bytes)?;
    commit_entry(worktree, conv_id, seq, &rel, TOOL_ORIGIN, git)
}

/// `messages/NNN-<origin>.json` for `seq`, zero-padded to [`SEQ_WIDTH`].
fn entry_rel(seq: u32, origin: &str) -> String {
    format!("{MESSAGES_DIR}/{seq:0w$}-{origin}.json", w = SEQ_WIDTH)
}

/// `git add <rel>` then commit the entry on the conversation branch.
fn commit_entry(
    worktree: &Path,
    conv_id: &str,
    seq: u32,
    rel: &str,
    origin: &str,
    git: &dyn GitRunner,
) -> Result<(), Error> {
    git.run(worktree, &["add", rel])
        .map_err(|source| Error::Git {
            op: "transcript add",
            source,
        })?;
    let msg = format!("transcript {seq:0w$}: {origin} [{conv_id}]", w = SEQ_WIDTH);
    git.run(worktree, &["commit", "-m", msg.as_str()])
        .map_err(|source| Error::Git {
            op: "transcript commit",
            source,
        })
}

#[cfg(test)]
mod tests;
