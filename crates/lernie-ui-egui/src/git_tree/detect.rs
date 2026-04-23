//! Exchange-path detection and user-message preview extraction.
//!
//! Two shapes coexist in repos that have been migrated from v0.1 to
//! v0.2 (ARCH §12 v0.1 exception). Both live under `exchanges/` but
//! have different internal structure:
//!
//! - v0.1: `exchanges/<id>.json` — flat per-exchange JSON with a
//!   `user_message` string field.
//! - v0.2: `exchanges/<id>/steps/<NNN>/{request,response}.json` — the
//!   user message is `messages[0].content` in the request file.
//!
//! Preview strings cap at [`PREVIEW_MAX`] chars after whitespace
//! normalization so the render layer can size them predictably.

pub(super) const PREVIEW_MAX: usize = 80;

/// `exchanges/<id>.json` at the top level — the v0.1 shape.
pub(super) fn is_v01_exchange_path(path: &str) -> bool {
    path.starts_with("exchanges/") && path.ends_with(".json") && !path[10..].contains('/')
}

/// `exchanges/<id>/steps/<NNN>/...` — the v0.2 shape. Returns the
/// exchange id if the path matches, else `None`.
pub(super) fn v02_exchange_id_from_path(path: &str) -> Option<&str> {
    let rest = path.strip_prefix("exchanges/")?;
    let (id, tail) = rest.split_once("/steps/")?;
    // Ensure at least one more segment after steps/ — the step dir —
    // so bare `exchanges/<id>/steps/` (no file) doesn't match.
    tail.split_once('/').map(|_| id)
}

pub(super) fn exchange_id_from_v01_path(path: &str) -> String {
    path.strip_prefix("exchanges/")
        .and_then(|s| s.strip_suffix(".json"))
        .map(|s| s.to_string())
        .unwrap_or_else(|| path.to_string())
}

pub(super) fn exchange_id_from_branch(branch: &str) -> String {
    branch
        .strip_prefix("ex/")
        .map(|s| s.to_string())
        .unwrap_or_else(|| branch.to_string())
}

pub(super) fn extract_v01_preview(json_bytes: &[u8]) -> Option<String> {
    let value: serde_json::Value = serde_json::from_slice(json_bytes).ok()?;
    let msg = value.get("user_message")?.as_str()?;
    Some(truncate_preview(msg))
}

pub(super) fn extract_v02_preview(json_bytes: &[u8]) -> Option<String> {
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
