//! Production [`super::ToolExecutor`] — resolves the tool binary,
//! delegates spawn / capture / cascade to [`super::subprocess`], and
//! lands the per-call disk record under `<step_dir>/tools/<tool-id>/`.
//!
//! Resolution order, per ARCH §3.3:
//!
//! 1. `<data_root>/tools/lernie-tool-<name>` (installed by `make
//!    install`).
//! 2. `lernie-tool-<name>` on `PATH` (mirroring §4.4 adapter discovery).
//! 3. In-process fallback: `<driver target> tool <name>` — re-entry
//!    into the same dispatcher, matching PRINCIPLES "Everyone uses the
//!    front door". The target is the one the binding injected
//!    (`cmd::Fx::driver_target`), never a name this module resolves:
//!    ARCH §2.11, "the driver target is injected at the binding, not
//!    resolved by name". Under the exec binding that is the `lernie`
//!    image; under a linked host it is the host's own re-exec target or
//!    a PATH-resolved `lernie` — never the host binary itself, which
//!    carries no `tool` verb of its own.

mod caller;

use caller::Caller;

use super::subprocess::{SpawnArgs, spawn_and_capture};
use super::{
    ExecError, IN_PROCESS_SUBCOMMAND, INPUT_FILE, OUTPUT_FILE, ToolCall, ToolExecutor,
    ToolInputRecord, ToolOutcome, ToolOutputRecord, atomic_write_json, envelope, tool_call_dir,
};
use crate::prompt::Clock;
use crate::template::{GitRunner, RealGit};
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
    driver_target: &'a Path,
    deadline: Duration,
    etxtbsy_budget: u32,
    path_lookup: Box<dyn PathLookup + 'a>,
    git: Box<dyn GitRunner + 'a>,
}

/// Indirection for the §3.3 second hop so tests can drive the PATH
/// lookup without manipulating the process env. Production wires
/// [`EnvPath`], which reads the live `PATH`. The third hop needs no
/// indirection: its target is injected, not looked up.
pub trait PathLookup {
    /// PATH lookup for the externalized tool binary
    /// (`lernie-tool-<name>`), the second hop in §3.3 resolution.
    fn which_on_path(&self, prefixed_name: &str) -> Option<PathBuf>;
}

/// Real-process lookup: the live `PATH`, via [`which_in_path`].
pub struct EnvPath;

impl PathLookup for EnvPath {
    fn which_on_path(&self, prefixed_name: &str) -> Option<PathBuf> {
        which_in_path(prefixed_name)
    }
}

impl<'a> SpawnTool<'a> {
    /// Build a [`SpawnTool`] over the live `PATH` and the default §3.3
    /// deadline. `driver_target` is the binding-injected re-entry path
    /// (`cmd::Fx::driver_target`) the third hop addresses as
    /// `<driver_target> tool <name>`.
    pub fn new(data_root: &'a Path, clock: &'a dyn Clock, driver_target: &'a Path) -> Self {
        Self {
            data_root,
            clock,
            driver_target,
            deadline: super::DEFAULT_TOOL_DEADLINE,
            etxtbsy_budget: super::subprocess::ETXTBSY_RETRY_ATTEMPTS,
            path_lookup: Box::new(EnvPath),
            git: Box::new(RealGit::new()),
        }
    }

    /// Override how many spawn attempts ride out `ETXTBSY` — an attempt
    /// count, never a wall-clock deadline (README's determinism rule,
    /// bl-edf6). A test that means to exercise the retry arm sets a
    /// count its fixture's hold cannot outlast, and one that means to
    /// exercise the give-up arm sets a small count against a permanent
    /// hold — both arms are then structural, with no clock in the
    /// verdict at all (bl-7a3f).
    #[cfg(test)] // test-only builder
    pub fn with_etxtbsy_budget(mut self, attempts: u32) -> Self {
        self.etxtbsy_budget = attempts;
        self
    }

    /// Override the SIGTERM-to-SIGKILL grace. Tests use a sub-second
    /// deadline so the cascade is observable without a 5s wait.
    #[cfg(test)] // test-only builder
    pub fn with_deadline(mut self, d: Duration) -> Self {
        self.deadline = d;
        self
    }

    /// Override the PATH lookup — used by tests to drive the second hop
    /// without mutating the live `PATH`.
    #[cfg(test)] // test-only builder
    pub fn with_path_lookup(mut self, l: Box<dyn PathLookup + 'a>) -> Self {
        self.path_lookup = l;
        self
    }

    /// Override the git runner the working-directory mark is read through
    /// (§3.3) — tests drive the moved-cwd arms without founding a repo.
    #[cfg(test)] // test-only builder
    pub fn with_git(mut self, g: Box<dyn GitRunner + 'a>) -> Self {
        self.git = g;
        self
    }

    /// Apply the §3.3 resolution order. Returns `(binary, args)` so
    /// the caller can spawn it without re-deciding the in-process
    /// case. Total: the third hop is the injected driver target, so
    /// there is no unresolvable case — a name no binary answers to is
    /// declined by the dispatcher behind the front door
    /// (`builtin::Error::Unknown`), not by this lookup.
    fn resolve(&self, name: &str) -> (OsString, Vec<OsString>) {
        let external_name = format!("{}{}", super::EXTERNAL_PREFIX, name);
        let harness_path = self.data_root.join(super::TOOLS_DIR).join(&external_name);
        if harness_path.is_file() {
            return (harness_path.into_os_string(), Vec::new());
        }
        if let Some(p) = self.path_lookup.which_on_path(&external_name) {
            return (p.into_os_string(), Vec::new());
        }
        let args = vec![OsString::from(IN_PROCESS_SUBCOMMAND), OsString::from(name)];
        (self.driver_target.as_os_str().to_owned(), args)
    }
}

impl<'a> ToolExecutor for SpawnTool<'a> {
    fn execute(
        &self,
        call: ToolCall<'_>,
        step_dir: &Path,
        stop: &AtomicBool,
    ) -> Result<ToolOutcome, ExecError> {
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
        let stdin = serde_json::to_vec(call.input).expect("Value is always serializable");
        let extra_env = caller.env();

        let binary_ref = &binary;
        let req = SpawnArgs {
            binary: binary_ref,
            args: &args,
            stdin_bytes: &stdin,
            extra_env: &extra_env,
            cwd: &caller.cwd,
            stop,
            deadline: self.deadline,
            etxtbsy_budget: self.etxtbsy_budget,
            tool_name: call.name,
        };
        let started_at = self.clock.now_iso8601();
        let captured = spawn_and_capture(&req)?;
        let ended_at = self.clock.now_iso8601();

        let exit_code = match captured.status.code() {
            Some(c) => c,
            None => return Err(killed_by_signal(call.name, &captured.status)),
        };

        let content = envelope::render(exit_code, &captured.stdout, &captured.stderr);
        let is_error = exit_code != 0;

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
