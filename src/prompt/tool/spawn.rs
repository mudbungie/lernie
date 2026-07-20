//! Production [`super::ToolExecutor`] — resolves the tool binary,
//! delegates spawn / capture / cascade to [`super::subprocess`], and
//! lands the per-call disk record under `<step_dir>/tools/<tool-id>/`.
//!
//! Resolution order, per ARCH §3.3:
//!
//! 1. `<data_root>/tools/lernie-tool-<name>` (installed by `make
//!    install`).
//! 2. `lernie-tool-<name>` on `PATH` (mirroring §4.4 adapter discovery).
//! 3. In-process fallback: `<lernie binary> tool <name>`. The binary
//!    path is `std::env::current_exe()` — re-entry into the same
//!    dispatcher, matching PRINCIPLES "Everyone uses the front door".

use super::subprocess::{SpawnArgs, spawn_and_capture};
use super::{
    ExecError, IN_PROCESS_SUBCOMMAND, INPUT_FILE, OUTPUT_FILE, ToolCall, ToolExecutor,
    ToolInputRecord, ToolOutcome, ToolOutputRecord, atomic_write_json, tool_call_dir,
};
use crate::prompt::Clock;
use std::ffi::{OsStr, OsString};
use std::os::unix::process::ExitStatusExt;
use std::path::{Path, PathBuf};
use std::sync::atomic::AtomicBool;
use std::time::Duration;

/// Production [`ToolExecutor`]. Constructed per-call by the loop so
/// the borrow of `data_root` and `clock` stays scoped to one
/// invocation.
pub struct SpawnTool<'a> {
    data_root: &'a Path,
    clock: &'a dyn Clock,
    deadline: Duration,
    binary_resolver: Box<dyn BinaryResolver + 'a>,
}

/// Indirection for tool-binary resolution so tests can drive both the
/// PATH lookup and the in-process fallback without manipulating the
/// process env. Production wires [`CurrentExeResolver`], whose
/// methods read the live `PATH` and `current_exe`.
pub trait BinaryResolver {
    /// Path to the `lernie` binary for in-process tool dispatch, or
    /// `None` if it cannot be determined. A `None` here surfaces as
    /// [`ExecError::NotFound`] when external lookup also missed.
    fn lernie_binary(&self) -> Option<PathBuf>;

    /// PATH lookup for the externalized tool binary
    /// (`lernie-tool-<name>`), the second hop in §3.3 resolution.
    /// Default delegates to [`which_in_path`]; tests override to
    /// control PATH content without mutating the live env.
    fn which_on_path(&self, prefixed_name: &str) -> Option<PathBuf> {
        which_in_path(prefixed_name)
    }
}

/// Real-process resolver: `std::env::current_exe()` is the actively
/// running binary's path on every platform we care about; PATH is
/// inherited via the default [`BinaryResolver::which_on_path`].
///
/// PHASE-3 (bl-231c follow-on): this is the one `current_exe` left in
/// the library — the §3.3 tool-resolution third hop (`<lernie> tool
/// <name>`), a *separate* seam from the §2.11/§6 driver-target family
/// (`cmd::Fx::driver_target`), which no longer touches `current_exe`.
/// Unify it with the injected driver target when `SpawnTool::new` is
/// re-signed to take the binding's binary path (a change that migrates
/// this module's ~18 `SpawnTool::new` unit-test call sites, out of
/// scope for the command-surface port).
pub struct CurrentExeResolver;

impl BinaryResolver for CurrentExeResolver {
    fn lernie_binary(&self) -> Option<PathBuf> {
        std::env::current_exe().ok()
    }
}

impl<'a> SpawnTool<'a> {
    /// Build a [`SpawnTool`] backed by [`CurrentExeResolver`] and the
    /// default §3.3 deadline.
    pub fn new(data_root: &'a Path, clock: &'a dyn Clock) -> Self {
        Self {
            data_root,
            clock,
            deadline: super::DEFAULT_TOOL_DEADLINE,
            binary_resolver: Box::new(CurrentExeResolver),
        }
    }

    /// Override the SIGTERM-to-SIGKILL grace. Tests use a sub-second
    /// deadline so the cascade is observable without a 5s wait.
    #[cfg(test)] // test-only builder
    pub fn with_deadline(mut self, d: Duration) -> Self {
        self.deadline = d;
        self
    }

    /// Override the in-process binary resolver — used by tests to
    /// inject a known-bad or known-good lernie path without depending
    /// on `std::env::current_exe()`'s value under cargo.
    #[cfg(test)] // test-only builder
    pub fn with_resolver(mut self, r: Box<dyn BinaryResolver + 'a>) -> Self {
        self.binary_resolver = r;
        self
    }

    /// Apply the §3.3 resolution order. Returns `(binary, args)` so
    /// the caller can spawn it without re-deciding the in-process
    /// case.
    fn resolve(&self, name: &str) -> Result<(OsString, Vec<OsString>), ExecError> {
        let external_name = format!("{}{}", super::EXTERNAL_PREFIX, name);
        let tools_root = self.data_root.join(super::TOOLS_DIR);
        let harness_path = tools_root.join(&external_name);
        if harness_path.is_file() {
            return Ok((harness_path.into_os_string(), Vec::new()));
        }
        if let Some(p) = self.binary_resolver.which_on_path(&external_name) {
            return Ok((p.into_os_string(), Vec::new()));
        }
        let resolver = &self.binary_resolver;
        let lernie = resolver
            .lernie_binary()
            .ok_or_else(|| ExecError::NotFound {
                name: name.to_string(),
                harness_path,
            })?;
        let args = vec![OsString::from(IN_PROCESS_SUBCOMMAND), OsString::from(name)];
        Ok((lernie.into_os_string(), args))
    }
}

impl<'a> ToolExecutor for SpawnTool<'a> {
    fn execute(
        &self,
        call: ToolCall<'_>,
        step_dir: &Path,
        stop: &AtomicBool,
    ) -> Result<ToolOutcome, ExecError> {
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

        let (binary, args) = self.resolve(call.name)?;
        let stdin = serde_json::to_vec(call.input).expect("Value is always serializable");
        let extra_env = harness_env_for(step_dir);

        let binary_ref = &binary;
        let req = SpawnArgs {
            binary: binary_ref,
            args: &args,
            stdin_bytes: &stdin,
            extra_env: &extra_env,
            stop,
            deadline: self.deadline,
            tool_name: call.name,
        };
        let started_at = self.clock.now_iso8601();
        let captured = spawn_and_capture(&req)?;
        let ended_at = self.clock.now_iso8601();

        let exit_code = match captured.status.code() {
            Some(c) => c,
            None => return Err(killed_by_signal(call.name, &captured.status)),
        };

        let mut content = captured.stdout.clone();
        let is_error = exit_code != 0;
        if is_error {
            content.extend_from_slice(&captured.stderr);
        }

        let output_record = ToolOutputRecord {
            stdout: String::from_utf8_lossy(&captured.stdout).into_owned(),
            stderr: String::from_utf8_lossy(&captured.stderr).into_owned(),
            exit_code,
            started_at,
            ended_at,
        };
        atomic_write_json(&dir, OUTPUT_FILE, &output_record)?;

        Ok(ToolOutcome { content, is_error })
    }
}

/// Env vars the harness conveys to every tool subprocess per ARCH §3.3
/// (the env-var bullet on the stdio contract). Names are pinned in
/// [`super::builtin::dispatch`] (the dispatch built-in is the v0.4
/// reader); tools that do not need them ignore them. Both are derived
/// from `step_dir = <conv-repo>/steps/<conv-id>/<NNN>` so the executor
/// stays the single source of truth and no caller has to hand them in.
fn harness_env_for(step_dir: &Path) -> Vec<(&'static str, std::ffi::OsString)> {
    let mut env: Vec<(&'static str, std::ffi::OsString)> = Vec::new();
    // step_dir = <conv-repo>/steps/<conv-id>/<NNN>; ascend three to
    // reach the conv-repo, two for the conv-id segment.
    if let Some(conv_id_dir) = step_dir.parent() {
        if let Some(conv_id) = conv_id_dir.file_name() {
            env.push((super::ENV_CONV_BRANCH, conv_id.to_owned()));
        }
        if let Some(conv_repo) = conv_id_dir.parent().and_then(Path::parent) {
            env.push((super::ENV_CONV_REPO, conv_repo.as_os_str().to_owned()));
        }
    }
    env
}

/// Build the §3.3 / §2.10 "killed by a signal that was not the
/// harness's SIGTERM" fault. Extracts the signal number from the
/// kernel-reported [`ExitStatus`] (defaulting to 0 if some platform
/// reports neither code nor signal — should not happen on Linux).
fn killed_by_signal(name: &str, status: &std::process::ExitStatus) -> ExecError {
    let signal = status.signal().unwrap_or(0);
    ExecError::KilledBySignal {
        name: name.to_string(),
        signal,
    }
}

/// PATH lookup for `name` against the live process env. Wraps
/// [`which_in_path_env`] so the env-var read sits in one place; tests
/// drive `which_in_path_env` directly with a constructed path and
/// invoke this wrapper once for the env-read branch.
pub(super) fn which_in_path(name: &str) -> Option<PathBuf> {
    which_in_path_env(name, std::env::var_os("PATH").as_deref())
}

/// PATH lookup that takes the path string as a parameter. First hit
/// wins. Returns an absolute path so the spawn is unambiguous.
pub(super) fn which_in_path_env(name: &str, path: Option<&OsStr>) -> Option<PathBuf> {
    let path = path?;
    for dir in std::env::split_paths(path) {
        let candidate = dir.join(name);
        if candidate.is_file() {
            return Some(candidate);
        }
    }
    None
}
