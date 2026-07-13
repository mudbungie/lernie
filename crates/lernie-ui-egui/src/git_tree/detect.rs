//! User-message preview extraction.
//!
//! The preview is read from disk at
//! `<workspace>/steps/<agent-id>/001/request.json` — step records are
//! never in a git tree (§2.3 "Step records are not committed to git").
//! Cap at [`PREVIEW_MAX`] chars after whitespace normalization so the
//! render layer can size predictably.

pub(super) const PREVIEW_MAX: usize = 80;

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
