//! Step-path detection and user-message preview extraction for v0.3.
//!
//! v0.3 commits land step records under `steps/<conv-id>/<NNN>/` (ARCH
//! §2.3). The user message lives in `request.json` as
//! `messages[0].content`. Branches are bare conv-ids — root
//! conversations on `main`, subagents on their full hyphenated descent
//! (ARCH §2.3) — so no branch-name unprefixing is needed.
//!
//! Preview strings cap at [`PREVIEW_MAX`] chars after whitespace
//! normalization so the render layer can size them predictably.

pub(super) const PREVIEW_MAX: usize = 80;

/// `steps/<conv-id>/<NNN>/<file>...` — the v0.3 shape. Returns the
/// conv-id if the path matches, else `None`. Bare `steps/<id>/<NNN>/`
/// (no file) does not match.
pub(super) fn v03_conv_id_from_path(path: &str) -> Option<&str> {
    let rest = path.strip_prefix("steps/")?;
    let (id, tail) = rest.split_once('/')?;
    // Tail is `<NNN>/<file>` at minimum; bare `<NNN>` (no slash) means
    // the path stopped at the step dir, which is not a file commit.
    tail.split_once('/').map(|_| id)
}

pub(super) fn extract_request_preview(json_bytes: &[u8]) -> Option<String> {
    let value: serde_json::Value = serde_json::from_slice(json_bytes).ok()?;
    let msg = value
        .get("messages")?
        .as_array()?
        .first()?
        .get("content")?
        .as_str()?;
    Some(truncate_preview(msg))
}

pub(super) fn truncate_preview(s: &str) -> String {
    let collapsed: String = s
        .chars()
        .map(|c| if c.is_whitespace() { ' ' } else { c })
        .collect();
    let trimmed = collapsed.trim();
    if trimmed.chars().count() <= PREVIEW_MAX {
        return trimmed.to_string();
    }
    let head: String = trimmed.chars().take(PREVIEW_MAX - 1).collect();
    format!("{head}…")
}
