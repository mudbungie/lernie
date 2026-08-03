//! The success report (ARCH §3.3 *The patch tool*): what `apply_patch`
//! prints on stdout, and therefore what the executor's ordinary capture
//! lands in the diagnostic `output.json` and — via the §3.3 result
//! envelope — in the `tool_result` the model reads.

use serde::Serialize;

/// One entry per file operation, in patch order. For update hunks the
/// report carries the winning ladder rung and 1-based landing line;
/// when a fuzzy rung won, `matched` preserves the lines actually
/// replaced, so the true pre-state rides the ordinary tool record even
/// where it differs from the authored context (the patch itself,
/// preserved verbatim in `input.json`, is the authored pre/post diff).
#[derive(Debug, Serialize)]
pub struct Report {
    pub status: &'static str,
    pub files: Vec<FileReport>,
}

/// One file operation's outcome.
#[derive(Debug, Serialize)]
pub struct FileReport {
    pub path: String,
    pub op: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub moved_to: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub hunks: Vec<HunkReport>,
}

/// One applied hunk: where it landed and how it was matched.
#[derive(Debug, Serialize)]
pub struct HunkReport {
    pub rung: &'static str,
    pub line: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub matched: Option<Vec<String>>,
}

/// Assemble one file's report entry.
pub fn entry(
    path: &str,
    op: &'static str,
    moved_to: Option<String>,
    hunks: Vec<HunkReport>,
) -> FileReport {
    FileReport {
        path: path.to_string(),
        op,
        moved_to,
        hunks,
    }
}
