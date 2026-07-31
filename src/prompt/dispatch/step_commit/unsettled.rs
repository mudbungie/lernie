//! Prune the inherited **unsettled tool step** from the forked tree, at
//! the dispatch commit (ARCH §2.3 step 2, §2.5, §3.3, §5.4).
//!
//! A tool-call dispatch runs *inside* the parent's tool step (§2.5): the
//! parent's model-output entry — carrying the `tool_use` block whose
//! execution is the dispatch — is already committed, and the answering
//! `messages/NNN-tool.json` cannot commit until the tool returns, which
//! is after the fork. So the child forks off a tip whose transcript ends
//! in what §3.3 already names *tool in progress*: a `tool_use` block with
//! no matching committed `tool_result` entry.
//!
//! That tail is legal on the parent's branch — it settles moments later —
//! and illegal on the child's, where nothing will ever answer it. Every
//! provider validates each `tool_use` against a `tool_result` in the
//! immediately following wire message (§2.5), so the child's first model
//! call was refused outright: `{"kind":{"provider":{"status":400}},
//! "message":"No tool output found for function call call_…"}` (bl-4231),
//! which killed the whole agent-initiated dispatch path.
//!
//! **The child's record is made honest at fork time, not at read time.**
//! The dispatch commit deletes the unsettled tail — the trailing
//! model-output entry whose `tool_use` blocks are not all answered, and
//! every entry after it (its partial results, which orphan the moment
//! their `tool_use` leaves). Deletion is the sanctioned transcript change
//! (§2.3 *change is append or delete, never edit-in-place*; §5.4) and the
//! parent's own copy is untouched, so nothing is lost: the entry stays on
//! the parent's branch and in the child's git history.
//!
//! The alternative — teaching assembly to skip a trailing unpaired
//! `tool_use` — is rejected on §2.7's own reasoning about the compactor's
//! inherited blocks: the wire framing is transcript-backed, so a filter
//! there would make the model call disagree with the branch's record, and
//! assembly would stop being a pure function of the tree (§5.1). Moving
//! the fork point past the tool step is rejected too: the dispatch's
//! `tool_result` *is* the child's address (§2.5), so the settled boundary
//! does not exist until after the fork must have happened.
//!
//! It lives in [`super::trim_to_context`] beside the control-file and
//! descriptor removals because it is the same act: the dispatch commit
//! trimming the forked tree to exactly what this agent may hold. Total
//! and idempotent like its siblings — a root forked off a config commit
//! carries no transcript, a compactor forks off a checkpoint commit read
//! at a *closed* tool step (§2.7), and a verifier forks off a terminal
//! ref; all three settle to no git command at all.

use crate::prompt::Error;
use crate::template::GitRunner;
use brazen::Content;
use std::path::Path;

/// Branch-scoped transcript directory (ARCH §2.3 — `messages/NNN-…`).
const MESSAGES_DIR: &str = "messages";
/// The one reserved `.json` origin token (§2.3): a tool call's result.
/// Every other `.json` token is the model id that authored the entry.
const TOOL_ORIGIN: &str = "tool";

/// Stage the removal of the inherited transcript's unsettled tool step.
///
/// Issues **no** git command when the inherited tail is settled — every
/// fork point but a mid-tool-step one, and every re-run over an
/// already-pruned tree.
pub(crate) fn prune_unsettled(worktree: &Path, git: &dyn GitRunner) -> Result<(), Error> {
    let entries = sequence(worktree)?;
    let Some(cut) = unsettled_from(worktree, &entries)? else {
        return Ok(());
    };
    let mut args: Vec<&str> = vec!["rm", "-q", "--"];
    args.extend(entries[cut..].iter().map(String::as_str));
    git.run(worktree, &args).map_err(|source| Error::Git {
        op: "rm unsettled tool step",
        source,
    })
}

/// The branch's transcript entries as worktree-relative paths, ordered by
/// the filename's `NNN` counter — order lives in the name and nowhere
/// else (§2.3). An absent `messages/` yields none: the general path with
/// empty inputs, which is every fresh root.
fn sequence(worktree: &Path) -> Result<Vec<String>, Error> {
    let dir = worktree.join(MESSAGES_DIR);
    let read = match std::fs::read_dir(&dir) {
        Ok(rd) => rd,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(e) => return Err(Error::Io(e)),
    };
    let mut numbered: Vec<(u32, String)> = Vec::new();
    for entry in read {
        let name = entry.map_err(Error::Io)?.file_name();
        let name = name.to_string_lossy().into_owned();
        if let Some(seq) = name.split('-').next().and_then(|p| p.parse::<u32>().ok()) {
            numbered.push((seq, format!("{MESSAGES_DIR}/{name}")));
        }
    }
    numbered.sort_by_key(|(seq, _)| *seq);
    Ok(numbered.into_iter().map(|(_, rel)| rel).collect())
}

/// The index in `entries` at which the unsettled tool step begins, or
/// `None` when the tail is settled.
///
/// The step is the branch's *last* model-output entry plus everything
/// after it: it is unsettled when some `tool_use` id it emitted has no
/// `tool_result` naming it among the following tool entries. Only the
/// tail can be unsettled — the executor commits every result before the
/// next model call (§2.5 pairing) — so one look at the end is total.
fn unsettled_from(worktree: &Path, entries: &[String]) -> Result<Option<usize>, Error> {
    let Some(cut) = entries.iter().rposition(|rel| kind(rel) == Kind::Model) else {
        return Ok(None);
    };
    let mut pending: Vec<String> = blocks(worktree, &entries[cut])?
        .iter()
        .filter_map(|b| match b {
            Content::ToolUse { id, .. } => Some(id.clone()),
            _ => None,
        })
        .collect();
    let answering = entries[cut + 1..]
        .iter()
        .filter(|rel| kind(rel) == Kind::Tool);
    for rel in answering {
        for block in blocks(worktree, rel)? {
            if let Content::ToolResult { tool_use_id, .. } = block {
                pending.retain(|id| *id != tool_use_id);
            }
        }
    }
    Ok((!pending.is_empty()).then_some(cut))
}

/// The canonical blocks of one `.json` transcript entry. Harness-written
/// (the staging seal / `commit_tool`, §2.3), so a parse failure is a
/// programmer error, not a reachable state.
fn blocks(worktree: &Path, rel: &str) -> Result<Vec<Content>, Error> {
    let bytes = std::fs::read(worktree.join(rel)).map_err(Error::Io)?;
    Ok(serde_json::from_slice(&bytes).expect("transcript entry is a canonical Content array"))
}

/// What a transcript entry is, derived from its path alone (§2.3
/// *Origins and wire framing*): the extension and the reserved-token
/// test, never frontmatter.
#[derive(PartialEq, Eq)]
enum Kind {
    /// `NNN-<sender>.md` — a delivered message (§2.11).
    Message,
    /// `NNN-tool.json` — one tool call's result.
    Tool,
    /// `NNN-<model-id>.json` — one step's model output.
    Model,
}

fn kind(rel: &str) -> Kind {
    let path = Path::new(rel);
    if path.extension().and_then(|e| e.to_str()) != Some("json") {
        return Kind::Message;
    }
    let stem = path
        .file_stem()
        .map(|s| s.to_string_lossy())
        .unwrap_or_default();
    match stem.split_once('-').map(|x| x.1) {
        Some(TOOL_ORIGIN) => Kind::Tool,
        _ => Kind::Model,
    }
}

#[cfg(test)]
mod tests;
