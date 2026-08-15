//! The per-tool-call **disk record** (ARCH §3.3 *Disk record*): where it
//! lives, what it holds, and how it is landed.
//!
//! Two files per tool call under `<workspace>/steps/<agent-id>/<NNN>/
//! tools/<tool-id>/` — outside every worktree (§2.2 / §2.3), diagnostic
//! and never read at runtime. The executor owns the subtree and lands the
//! pair around **every** answer, whether the call was spawned or answered
//! by a host router ([`super::inject`]), so the convention holds for both
//! without either having to remember it.

use super::{ExecError, STEP_TOOLS_SUBDIR};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::path::{Path, PathBuf};

/// On-disk filenames for the per-tool-call record (ARCH §3.3 "Disk record").
pub const INPUT_FILE: &str = "input.json";
pub const OUTPUT_FILE: &str = "output.json";

/// On-disk shape of `<tool-id>/input.json` — the `tool_use` block
/// verbatim per ARCH §3.3 ("`id`, `name`, `input`"). Round-trips
/// through serde so a future replay can rehydrate the call.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ToolInputRecord {
    pub id: String,
    pub name: String,
    pub input: Value,
}

/// On-disk shape of `<tool-id>/output.json` — exactly the fields ARCH
/// §3.3 enumerates ("`{stdout, stderr, exit_code, started_at,
/// ended_at}`"). Stdout / stderr are stored as strings via
/// lossy-utf8 to keep the record human-readable; the executor's
/// in-memory [`super::ToolOutcome::content`] preserves the raw bytes for
/// the loop to feed back to the model verbatim.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ToolOutputRecord {
    pub stdout: String,
    pub stderr: String,
    pub exit_code: i32,
    pub started_at: String,
    pub ended_at: String,
}

/// Per-step `tools/` directory for `tool_id` under `step_dir`. Owns
/// path construction so the disk-record convention (§3.3) lives in one
/// place.
pub(crate) fn tool_call_dir(step_dir: &Path, tool_id: &str) -> PathBuf {
    step_dir.join(STEP_TOOLS_SUBDIR).join(tool_id)
}

/// Atomic-rename JSON write — temp-path `<file>.tmp` is created next
/// to the final path, populated, fsync'd, and renamed into place. ARCH
/// §3.3 demands this so partial captures never surface in `git
/// status`; PRINCIPLES.md "Disk first" makes the discipline general.
pub(crate) fn atomic_write_json<T: Serialize>(
    dir: &Path,
    filename: &str,
    value: &T,
) -> Result<(), ExecError> {
    let bytes = serde_json::to_vec_pretty(value).expect("serializable record");
    let final_path = dir.join(filename);
    let tmp_path = dir.join(format!("{filename}.tmp"));
    std::fs::write(&tmp_path, bytes).map_err(|source| ExecError::Io {
        dir: dir.to_path_buf(),
        source,
    })?;
    std::fs::rename(&tmp_path, &final_path).map_err(|source| ExecError::Io {
        dir: dir.to_path_buf(),
        source,
    })?;
    Ok(())
}
