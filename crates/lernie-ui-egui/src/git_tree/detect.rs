//! Conversation detection and user-message preview extraction for v0.3.
//!
//! v0.3.1 detects a conversation by the merged branch's name on the
//! trunk's `--no-ff` merge subjects. With v0.3 dropping branch prefixes
//! and v0.3.1 relocating step records out of every worktree (ARCH §2.3),
//! the merge commit's tree no longer carries a `steps/<conv-id>/...`
//! path that could be diffed for the conv-id — the diagnostic record is
//! at `<conv-repo>/steps/<conv-id>/<NNN>/`, never committed (§2.3
//! "Step records are not committed to git"). The branch name is the
//! authoritative source instead, and `git merge --no-ff <branch>` (the
//! shape used by `src/prompt/merge`) writes the default subject
//! `Merge branch '<branch>'`, which is what we parse here.
//!
//! The user-message preview is read from disk at
//! `<conv-repo>/steps/<conv-id>/001/request.json` — it is no longer in
//! a git tree to `git show` against. Cap at [`PREVIEW_MAX`] chars after
//! whitespace normalization so the render layer can size predictably.

pub(super) const PREVIEW_MAX: usize = 80;

/// Extract the merged branch name from a `--no-ff` merge subject. Git's
/// default subject for `git merge --no-ff <branch>` is
/// `Merge branch '<branch>'`, with an optional ` into <target>` tail
/// when the merge target isn't `main`. Returns `None` for any other
/// subject shape (non-conversation merges, plain commits, etc.).
pub(super) fn parse_merge_subject(subject: &str) -> Option<&str> {
    let rest = subject.strip_prefix("Merge branch '")?;
    let end = rest.find('\'')?;
    Some(&rest[..end])
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
