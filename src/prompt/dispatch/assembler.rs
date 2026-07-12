//! Assemble the model-facing wire history from the committed transcript
//! (ARCH §2.3, §5).
//!
//! Context assembly has exactly one input: the read-state commit's tree
//! (§5.1), materialized as the branch's worktree. This module reads
//! `messages/` from that tree, sorts by the filename's `NNN` prefix —
//! order lives in the name, no git-log walk and no index (§2.3) — and
//! composes each entry into a wire [`Message`] by its origin token:
//!
//! - `NNN-<sender>.md` — a delivered message (§2.11): user-role text.
//! - `NNN-assistant.json` — one step's assistant output: the canonical
//!   [`Content`] blocks verbatim, as an assistant-role message.
//! - `NNN-tool.json` — one tool call's `tool_result` block(s): user-role
//!   content in the following wire message.
//!
//! Consecutive same-side entries group into one alternating wire
//! message, so every `tool_use` block is matched by a `tool_result` in
//! the immediately following user message *by construction* (§2.3, §2.5
//! pairing). Running, retry, and replay all call this one function
//! against one input — a commit's tree — so "replay" is not a mode
//! (§2.3 *Crash and recovery*).

use crate::prompt::Error;
use brazen::{Content, Message, Role};
use std::path::{Path, PathBuf};

/// Branch-scoped transcript directory (ARCH §2.3 — `messages/NNN-…`).
const MESSAGES_DIR: &str = "messages";
/// Reserved origin token for a step's assistant output (§2.3); every
/// other `.json` token (`tool`) and every `.md` sender composes
/// user-side.
const ASSISTANT_ORIGIN: &str = "assistant";

/// Which wire side an entry composes onto (§2.3). Grouping is by side,
/// not by [`Role`], so the enum can derive `PartialEq` without leaning
/// on brazen's type.
#[derive(PartialEq, Eq, Clone, Copy)]
enum Side {
    User,
    Assistant,
}

impl Side {
    fn role(self) -> Role {
        match self {
            Side::User => Role::User,
            Side::Assistant => Role::Assistant,
        }
    }
}

/// Assemble the wire message history from the transcript under
/// `<worktree>/messages/` (ARCH §2.3, §5). An absent or empty directory
/// yields no messages — the general path with empty inputs, not a
/// bootstrap special case.
pub(super) fn assemble(worktree: &Path) -> Result<Vec<Message>, Error> {
    let dir = worktree.join(MESSAGES_DIR);
    let mut entries: Vec<(u32, PathBuf)> = match std::fs::read_dir(&dir) {
        Ok(rd) => {
            let mut v = Vec::new();
            for entry in rd {
                let path = entry.map_err(Error::Io)?.path();
                if let Some(seq) = seq_of(&path) {
                    v.push((seq, path));
                }
            }
            v
        }
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Vec::new(),
        Err(e) => return Err(Error::Io(e)),
    };
    entries.sort_by_key(|(seq, _)| *seq);

    let mut messages: Vec<Message> = Vec::new();
    for (_, path) in entries {
        let (side, content) = compose_entry(&path)?;
        push_grouped(&mut messages, side, content);
    }
    Ok(messages)
}

/// The `NNN` counter of a `messages/NNN-<origin>.<ext>` path (the prefix
/// before the first `-`). A non-conforming name contributes no entry.
fn seq_of(path: &Path) -> Option<u32> {
    path.file_name()?
        .to_string_lossy()
        .split('-')
        .next()
        .and_then(|p| p.parse::<u32>().ok())
}

/// Compose one transcript entry into its wire `(side, content)`
/// (ARCH §2.3 *Origins and wire framing*). Role framing is derived from
/// the path — the extension and origin token — never from frontmatter.
fn compose_entry(path: &Path) -> Result<(Side, Vec<Content>), Error> {
    if path.extension().and_then(|e| e.to_str()) == Some("md") {
        let body = std::fs::read_to_string(path).map_err(Error::Io)?;
        return Ok((Side::User, vec![Content::Text(body)]));
    }
    let bytes = std::fs::read(path).map_err(Error::Io)?;
    // Harness-written (staging seal / `commit_tool`), so always a valid
    // canonical `Content` array — the writer's invariant (§2.3).
    let blocks: Vec<Content> =
        serde_json::from_slice(&bytes).expect("transcript entry is a canonical Content array");
    Ok((entry_side(path), blocks))
}

/// A `.json` entry is assistant-side iff its origin token is the
/// reserved `assistant` (§2.3); `tool` (and any other) composes
/// user-side as `tool_result` content.
fn entry_side(path: &Path) -> Side {
    let stem = path
        .file_stem()
        .map(|s| s.to_string_lossy())
        .unwrap_or_default();
    if stem.ends_with(&format!("-{ASSISTANT_ORIGIN}")) {
        Side::Assistant
    } else {
        Side::User
    }
}

/// Append `content` to the trailing message when it is the same side,
/// else start a new message (ARCH §2.3 — consecutive same-side entries
/// group into one alternating wire message).
fn push_grouped(messages: &mut Vec<Message>, side: Side, mut content: Vec<Content>) {
    match messages.last_mut() {
        Some(last) if last.role == side.role() => last.content.append(&mut content),
        _ => messages.push(Message {
            role: side.role(),
            content,
        }),
    }
}

#[cfg(test)]
mod tests;
