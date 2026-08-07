//! [`SpawnTool`]'s split of one tool call into the part that needs the
//! executor's own dependencies and the part that only blocks.
//!
//! `execute` is three phases: **prepare** (resolve the caller's
//! worktree, land the input record, resolve the binary — all of it
//! reaching `self.git`, `self.path_lookup`, `self.clock`), **run** (one
//! blocking [`spawn_and_capture`], reaching no `self` at all), and
//! **finish** (bound the streams, render the envelope, land the output
//! record — `self` again, via the caller's record path).
//!
//! Splitting them is what lets [`SpawnTool::execute_all`] (ARCH §3.3
//! *The multi-tool*, `execution: "parallel"`) overlap N calls without
//! sharing the executor across threads: only the middle phase crosses
//! into the scope, and it carries nothing but owned bytes, `&Path`, and
//! the `&AtomicBool` stop flag. The clock, the git runner and the PATH
//! lookup stay on the calling thread and need no `Sync` bound
//! (PRINCIPLES, severability).

use super::caller::Caller;
use super::{
    ExecError, INPUT_FILE, OUTPUT_FILE, SpawnArgs, SpawnTool, ToolCall, ToolInputRecord,
    ToolOutcome, ToolOutputRecord, atomic_write_json, bound, envelope, killed_by_signal,
    tool_call_dir,
};
use crate::config::ToolOutputBound;
use std::ffi::OsString;
use std::path::PathBuf;
use std::sync::atomic::AtomicBool;
use std::time::Duration;

/// Everything one call needs to spawn, resolved and owned so the
/// blocking phase borrows nothing from the executor.
pub(super) struct Prepared {
    dir: PathBuf,
    caller: Caller,
    binary: OsString,
    args: Vec<OsString>,
    stdin: Vec<u8>,
    extra_env: Vec<(&'static str, OsString)>,
    name: String,
}

impl Prepared {
    /// Borrow this call's owned parts into the request
    /// [`spawn_and_capture`] takes. `stop` is the only thing shared
    /// with the rest of the harness, and `&AtomicBool` is `Sync`.
    pub(super) fn spawn_args<'x>(
        &'x self,
        stop: &'x AtomicBool,
        deadline: Duration,
        etxtbsy_budget: u32,
    ) -> SpawnArgs<'x> {
        SpawnArgs {
            binary: &self.binary,
            args: &self.args,
            stdin_bytes: &self.stdin,
            extra_env: &self.extra_env,
            cwd: &self.caller.cwd,
            stop,
            deadline,
            etxtbsy_budget,
            tool_name: &self.name,
        }
    }
}

impl<'a> SpawnTool<'a> {
    /// Phase 1: resolve the calling agent's worktree, create the
    /// per-tool-call record directory, land `input.json`, and resolve the
    /// binary. Fails before any process is started.
    pub(super) fn prepare(
        &self,
        call: ToolCall<'_>,
        step_dir: &std::path::Path,
    ) -> Result<Prepared, ExecError> {
        let caller =
            Caller::resolve(step_dir, &*self.git).ok_or_else(|| ExecError::NoWorktree {
                name: call.name.to_string(),
                step_dir: step_dir.to_path_buf(),
            })?;
        let dir = tool_call_dir(step_dir, call.id);
        std::fs::create_dir_all(&dir).map_err(|source| ExecError::Io {
            dir: dir.clone(),
            source,
        })?;
        let input_record = ToolInputRecord {
            id: call.id.to_string(),
            name: call.name.to_string(),
            input: call.input.clone(),
        };
        atomic_write_json(&dir, INPUT_FILE, &input_record)?;
        let (binary, args) = self.resolve(call.name);
        let extra_env = caller.env();
        Ok(Prepared {
            dir,
            caller,
            binary,
            args,
            stdin: serde_json::to_vec(call.input).expect("Value is always serializable"),
            extra_env,
            name: call.name.to_string(),
        })
    }

    /// Phase 3: bound the captured streams (§3.3 *Bounded transcript
    /// projection* — before the envelope is rendered around them, since
    /// the envelope's header is structure and never cappable content),
    /// render the result envelope, and land `output.json` with the full
    /// bytes.
    pub(super) fn finish(
        &self,
        prepared: &Prepared,
        captured: super::Captured,
        output_bound: Option<ToolOutputBound>,
        started_at: &str,
        ended_at: &str,
    ) -> Result<ToolOutcome, ExecError> {
        let exit_code = match captured.status.code() {
            Some(code) => code,
            None => return Err(killed_by_signal(&prepared.name, &captured.status)),
        };
        let record = prepared.caller.record_rel(&prepared.dir).join(OUTPUT_FILE);
        let stdout = bound::apply(&captured.stdout, "stdout", output_bound, &record);
        let stderr = bound::apply(&captured.stderr, "stderr", output_bound, &record);
        let content = envelope::render(exit_code, &stdout, &stderr);
        let output_record = ToolOutputRecord {
            stdout: String::from_utf8_lossy(&captured.stdout).into_owned(),
            stderr: String::from_utf8_lossy(&captured.stderr).into_owned(),
            exit_code,
            started_at: started_at.to_string(),
            ended_at: ended_at.to_string(),
        };
        atomic_write_json(&prepared.dir, OUTPUT_FILE, &output_record)?;
        Ok(ToolOutcome {
            content,
            is_error: exit_code != 0,
        })
    }
}
