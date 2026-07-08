//! Budget-module tests. Split by concern so each file stays under the
//! 300-line cap:
//!
//! - [`derive`]: spend / wall / depth derivations over on-disk step
//!   trees (branch + descent), including the tolerance branches.
//! - [`enforce`]: [`super::check`] boundaries, [`super::remaining`] /
//!   [`super::clamp`] clamped inheritance, [`super::mark_exhausted`],
//!   and the `Display` diagnostics.

mod derive;
mod enforce;

use std::path::Path;
use tempfile::TempDir;

/// A serialized `Usage` event line (ARCH §6). A `None` counter is
/// written as JSON `null`, which the deriver must treat as 0.
pub(super) fn usage_line(
    input: Option<u32>,
    output: Option<u32>,
    cache_read: Option<u32>,
    cache_write: Option<u32>,
) -> String {
    let j = |v: Option<u32>| v.map(|n| n.to_string()).unwrap_or_else(|| "null".into());
    format!(
        "{{\"type\":\"usage\",\"input_tokens\":{},\"output_tokens\":{},\
         \"cache_read_tokens\":{},\"cache_write_tokens\":{}}}",
        j(input),
        j(output),
        j(cache_read),
        j(cache_write)
    )
}

/// One self-delimiting attempt segment: the given body line + a
/// terminal `end` (ARCH §4.4). Concatenate several to model retries.
pub(super) fn seg(body: &str) -> String {
    format!("{body}\n{{\"type\":\"end\"}}\n")
}

/// Write `<repo>/steps/<conv>/<NNN>/response.json` (outside every
/// worktree, ARCH §2.2 / §2.3).
pub(super) fn write_response(repo: &Path, conv: &str, seq: u32, body: &str) {
    let dir = repo.join("steps").join(conv).join(format!("{seq:03}"));
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("response.json"), body).unwrap();
}

/// Write `<repo>/steps/<conv>/<NNN>/meta.json` with the given span.
pub(super) fn write_meta(repo: &Path, conv: &str, seq: u32, started_at: &str, ended_at: &str) {
    let dir = repo.join("steps").join(conv).join(format!("{seq:03}"));
    std::fs::create_dir_all(&dir).unwrap();
    let body = format!(
        "{{\"commit\":\"abc\",\"started_at\":\"{started_at}\",\"ended_at\":\"{ended_at}\"}}"
    );
    std::fs::write(dir.join("meta.json"), body).unwrap();
}

pub(super) fn repo() -> TempDir {
    TempDir::new().unwrap()
}
