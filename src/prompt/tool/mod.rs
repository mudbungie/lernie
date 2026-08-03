//! Tool executor (ARCH §3.2 second bullet, §3.3).
//!
//! The harness emits a `tool_use` block from the model and the executor
//! turns it into a `tool_result` block — by spawning the tool's binary
//! per the §3.3 stdio contract, capturing stdout / stderr / exit, and
//! landing the per-call disk record (`input.json`, `output.json` under
//! `<step_dir>/tools/<tool-id>/`). The executor is one direction of the
//! disk-as-bus contract (§3.1): the harness writes the request via the
//! `tool_use` block in `response.json`; the per-call record makes the
//! result inspectable; the loop reads it back to assemble the next
//! step's request (§3.3 "Wire `tool_result` framing is application-layer").
//!
//! [`ToolExecutor`] is the trait the loop holds as `&dyn`. [`SpawnTool`]
//! is the production implementation; tests construct it directly against
//! a tempdir-rooted `harness_root` and shell-script fixture tools. Its
//! third resolution hop re-enters the command surface at the
//! binding-injected driver target (`cmd::Fx::driver_target`, §2.11) —
//! the library resolves no binary path by name.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::atomic::AtomicBool;
use std::time::Duration;
use thiserror::Error;

mod bound;
pub mod builtin;
pub mod control;
mod envelope;
pub mod spawn;
mod subprocess;

#[cfg(test)]
mod tests;

pub use spawn::SpawnTool;

/// SIGTERM-to-SIGKILL grace pinned by ARCH §3.3 (mirrors §4.4). The
/// tool has 5 seconds after SIGTERM to flush state and exit cleanly
/// before SIGKILL follows.
pub const DEFAULT_TOOL_DEADLINE: Duration = Duration::from_secs(5);

/// Subdirectory under the harness root where externalized tool binaries
/// live (ARCH §3.3 — "discovery mirrors §4.4: looks up `lernie-tool-<name>`
/// at `<harness-root>/tools/`").
pub const TOOLS_DIR: &str = "tools";

/// Name prefix for externalized tool binaries (ARCH §3.3, mirroring
/// §4.4's `lernie-provider-<name>` convention).
pub const EXTERNAL_PREFIX: &str = "lernie-tool-";

/// Argv used when invoking an in-process tool via the lernie binary
/// (ARCH §3.3 — "addressed as `lernie tool <name>`").
pub const IN_PROCESS_SUBCOMMAND: &str = "tool";

/// Env var conveying the conversation-repo root path to the tool
/// subprocess (ARCH §3.3 env-var bullet). Pinned here so the executor
/// (the writer) and the `dispatch` built-in (the reader) cannot drift.
pub const ENV_CONV_REPO: &str = "LERNIE_CONV_REPO";
/// Env var conveying the calling conversation's branch name (== full
/// hyphenated descent / conv-id, ARCH §2.2) to the tool subprocess.
/// Same provenance as [`ENV_CONV_REPO`].
pub const ENV_CONV_BRANCH: &str = "LERNIE_CONV_BRANCH";

/// Per-step subdirectory holding tool-call records (ARCH §3.3 "Disk
/// record" → `steps/<conv-id>/<NNN>/tools/<tool-id>/`).
pub const STEP_TOOLS_SUBDIR: &str = "tools";

/// On-disk filenames for the per-call record (ARCH §3.3 "Disk record").
pub const INPUT_FILE: &str = "input.json";
pub const OUTPUT_FILE: &str = "output.json";

/// One tool invocation as the model emitted it — the `id`, `name`, and
/// `input` fields of a `tool_use` content block (ARCH §3.3 stdin
/// contract). Borrowed because the loop owns the response the call
/// lives inside; the executor only reads it.
pub struct ToolCall<'a> {
    /// `tool_use.id` from the wire (e.g. `toolu_01abc…`); also the
    /// per-call directory name on disk per §3.3.
    pub id: &'a str,
    /// Tool name as the model spelled it; resolved against the harness
    /// root and PATH per §3.3.
    pub name: &'a str,
    /// `tool_use.input` JSON object; passed verbatim on the tool's
    /// stdin (§3.3 "Stdin").
    pub input: &'a Value,
}

/// Outcome of one tool invocation. The loop turns this into a wire
/// `tool_result` block on the next step's request, per ARCH §3.3
/// "Wire `tool_result` framing is application-layer".
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ToolOutcome {
    /// Bytes destined for the `tool_result.content` field. For a tool
    /// the executor ran, this is the §3.3 *result envelope*
    /// ([`envelope::render`]): the exit code stated, the child's stdout,
    /// and its stderr under a marker whenever the child wrote any —
    /// success included — each stream first bounded per the governing
    /// `tool_output:` policy ([`bound::apply`], §3.3 *Bounded transcript
    /// projection*); the full capture stays in `output.json`. A call declined before the executor was entered
    /// (§3.3 *declaring is not permitting*) carries the harness's own
    /// decline text instead: no child ran, so there is no exit code to
    /// state and none is invented.
    pub content: Vec<u8>,
    /// Maps to `tool_result.is_error` per §3.3: `false` for exit 0,
    /// `true` otherwise.
    pub is_error: bool,
}

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
/// in-memory [`ToolOutcome::content`] preserves the raw bytes for the
/// loop to feed back to the model verbatim.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ToolOutputRecord {
    pub stdout: String,
    pub stderr: String,
    pub exit_code: i32,
    pub started_at: String,
    pub ended_at: String,
}

/// Every way [`ToolExecutor::execute`] can fail. The taxonomy
/// distinguishes harness-level faults (lost subprocess, I/O failure
/// landing the record) from in-band tool errors. Resolution itself
/// cannot fail — the §3.3 third hop is the injected driver target, so
/// an unanswerable name is declined behind the front door as an
/// ordinary non-zero tool exit, not as a variant here. In-band
/// tool errors are not failures here — they surface as
/// [`ToolOutcome::is_error`] and travel back to the agent via the
/// `tool_result` block (§3.3 "Exit code").
#[derive(Debug, Error)]
pub enum ExecError {
    /// Spawn failed (bad executable, permission, fork limits, etc.).
    #[error("spawn tool {name:?}: {source}")]
    Spawn {
        name: String,
        #[source]
        source: io::Error,
    },
    /// Tool died from a signal (SIGSEGV, SIGABRT, …). Per ARCH §3.3 /
    /// §2.10 this is a harness-level fault, not a semantic tool failure
    /// delivered to the model — *except* when it is the executor's own
    /// group SIGTERM mid-stop: `run_tool_calls` (§2.9 step 3) reads that
    /// case as the stop, not a fault, by the stop flag, so a `KilledBySignal`
    /// only ever *propagates* out of the stop path.
    #[error(
        "tool {name:?} terminated by signal {signal} (not harness SIGTERM): \
        harness-level fault per ARCH §2.10"
    )]
    KilledBySignal { name: String, signal: i32 },
    /// The calling agent's worktree — the working directory every tool
    /// subprocess runs in (§3.3 *Working directory*) — could not be
    /// resolved from `step_dir`: either the path is not the §2.2
    /// `<workspace>/steps/<agent-id>/<NNN>` shape, or the worktree it
    /// names is not a live directory. Declined rather than falling back
    /// to the harness's inherited cwd: a tool left in whatever directory
    /// its launcher happened to be sitting in writes its side effects
    /// outside the agent's branch entirely.
    #[error(
        "no worktree for tool {name:?}: cannot resolve the calling agent's worktree \
        from step dir {step_dir:?} (ARCH §3.3 Working directory)"
    )]
    NoWorktree { name: String, step_dir: PathBuf },
    /// Landing `input.json` or `output.json` failed (disk full,
    /// permission, etc.). The executor uses temp-path + atomic rename
    /// so a failure here never leaves a half-written file in `git
    /// status`.
    #[error("i/o landing tool record under {dir:?}: {source}")]
    Io {
        dir: PathBuf,
        #[source]
        source: io::Error,
    },
}

/// Trait the v0.3 loop will hold as `&dyn ToolExecutor` (per the
/// `prompt::Deps` pattern in [`crate::prompt`]). The contract is
/// minimal so the loop can integrate-test against an in-process stub
/// without spawning subprocesses.
pub trait ToolExecutor {
    /// Resolve `call.name` to a binary, invoke it per the §3.3 stdio
    /// contract, land the per-call record under `<step_dir>/tools/
    /// <call.id>/`, and return the outcome the loop needs to assemble
    /// the next step's `tool_result` block.
    ///
    /// `step_dir` is the absolute conv-repo-rooted path of the
    /// `steps/<conv-id>/<NNN>/` directory the call belongs to — at the
    /// conversation-repo root, outside every worktree (ARCH §2.2 /
    /// §2.3), the directory `request.json` / `response.json` /
    /// `meta.json` are written into per [`crate::prompt::step`]. The
    /// executor owns the `tools/<tool-id>/` subtree below it.
    ///
    /// `stop` is the harness-wide cancel flag (PRINCIPLES "Stops are
    /// aggressive and cascading"): when set mid-execution, the
    /// executor sends SIGTERM and waits up to its deadline before
    /// SIGKILL. The flag is `&AtomicBool` rather than `&dyn`-of-
    /// trait because the only producer is the harness signal handler
    /// and the only consumer is the executor's polling loop —
    /// pretending it could be anything else would be premature
    /// abstraction.
    ///
    /// `output_bound` is the governing `workflow.yaml`'s `tool_output:`
    /// policy (ARCH §3.3 *Bounded transcript projection*, §6): each
    /// captured stream is bounded to its head+tail before the result
    /// envelope is rendered around them ([`bound::apply`]), while the
    /// full bytes still land in `output.json`. `None` — the block
    /// absent — projects the streams unbounded. Passed per call rather
    /// than held by the executor because it is the *calling agent's*
    /// policy, read from its governing config commit (§2.2), and the
    /// executor is constructed before any agent is resolved.
    fn execute(
        &self,
        call: ToolCall<'_>,
        step_dir: &Path,
        stop: &AtomicBool,
        output_bound: Option<crate::config::ToolOutputBound>,
    ) -> Result<ToolOutcome, ExecError>;
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
