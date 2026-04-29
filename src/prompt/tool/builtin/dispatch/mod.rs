//! `dispatch` built-in (ARCH §2.5, §3.3, §3.4 — v0.4 Phase 2).
//!
//! Stdin is the `tool_use.input` block as JSON: `{ "role": <string>,
//! "goal": <string> }`. The conversation context (which conv-repo,
//! which calling branch) arrives via the `LERNIE_CONV_REPO` and
//! `LERNIE_CONV_BRANCH` env vars the executor sets per ARCH §3.3 — it
//! is not in the model-facing input schema because the model does not
//! pick which conversation it is part of.
//!
//! The tool spawns the subagent through the §3.4 control plane —
//! `lernie dispatch <role> <repo> <branch> --goal <goal>` — rather than
//! calling [`crate::prompt::worker::run`] in-process. That CLI does the
//! Phase 1 work (worktree allocation, dispatch commit) and prints the
//! new subagent branch on stdout; the dispatch tool captures that
//! handle and re-emits it on its own stdout as the `tool_result`
//! payload `{"status":"in_progress","handle":"<sub-branch>"}` (ARCH
//! §2.5 "Async work uses handles"). The subagent's own step loop and
//! merge-back are Phases 3/4/5 — out of scope here.

use serde::{Deserialize, Serialize};
use std::ffi::OsString;
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
use std::process::Command;
use thiserror::Error;

use crate::config::{LoadError, PerRepoProviders};
use crate::prompt::SOULS_DIR;

/// Per-repo `providers.yaml` filename — pinned by ARCH §4.3 (the
/// per-repo config carries the role → (provider, model) mapping).
/// Local copy so the dispatch tool can validate `role` against the
/// `roles:` block without reaching back through the harness library.
const PER_REPO_PROVIDERS_FILE: &str = "providers.yaml";
/// Filename suffix for soul files (ARCH §4.3 — soul =
/// `<conv-repo>/souls/<role>.md`).
const SOUL_SUFFIX: &str = ".md";

/// Wire shape of the input. `serde(deny_unknown_fields)` so a
/// malformed `tool_use.input` surfaces as [`Error::InvalidJson`]
/// rather than silently dropping fields the model meant to pass.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Input {
    role: String,
    goal: String,
}

/// Wire shape of the output — the `tool_result.content` payload the
/// agent sees on its next step. `status` is always `in_progress` here
/// (ARCH §2.5 — the parent agent retrieves the terminal result later
/// via `await(handle)`); `handle` is the subagent's full hyphenated
/// descent branch (`<parent>-<sub-id>`, ARCH §2.2 / §2.3).
#[derive(Debug, Serialize, PartialEq, Eq)]
struct Output<'a> {
    status: &'a str,
    handle: &'a str,
}
const STATUS_IN_PROGRESS: &str = "in_progress";

/// Every way [`run`] can fail. Each variant prints its own stderr
/// message; per ARCH §3.3 stderr is concatenated after stdout into
/// `tool_result.content` when exit is non-zero, so the model sees the
/// failure verbatim.
#[derive(Debug, Error)]
pub enum Error {
    /// Stdin handed back bytes that did not parse as the documented
    /// `{role, goal}` shape — wrong type, missing field, extra field.
    #[error("invalid input JSON: {0}")]
    InvalidJson(#[source] serde_json::Error),
    /// The harness's stdin pipe failed mid-read.
    #[error("read input from stdin: {0}")]
    StdinRead(#[source] io::Error),
    /// Required env var (`LERNIE_CONV_REPO` / `LERNIE_CONV_BRANCH`)
    /// not set. Production callers always set these; the variant
    /// exists so a hand-invoked `lernie tool dispatch` outside a
    /// step gets a clear message instead of a confusing soul-read
    /// failure.
    #[error("missing env var {0:?} (set by the harness per ARCH §3.3)")]
    MissingEnv(&'static str),
    /// `providers.yaml` parsed but does not list the requested role
    /// in its `roles:` block.
    #[error("role {role:?} is not defined in {path}", path = path.display())]
    RoleMissing { role: String, path: PathBuf },
    /// Soul file for the requested role does not exist.
    #[error("soul {path} does not exist", path = path.display())]
    SoulMissing { path: PathBuf },
    /// `providers.yaml` parse / I/O surfaced via the harness's config
    /// loader.
    #[error("providers.yaml: {0}")]
    Config(#[from] LoadError),
    /// `lernie dispatch <role>` failed to spawn (binary missing,
    /// fork limits, etc.).
    #[error("spawn lernie dispatch {role:?}: {source}")]
    Spawn {
        role: String,
        #[source]
        source: io::Error,
    },
    /// `lernie dispatch <role>` exited non-zero. The subprocess's
    /// stderr is folded into the message so the failure reaches the
    /// agent verbatim.
    #[error("lernie dispatch {role:?} failed (exit {exit}): {stderr}")]
    DispatchExit {
        role: String,
        exit: i32,
        stderr: String,
    },
    /// `lernie dispatch <role>` exited 0 but printed no branch name —
    /// indicates a CLI contract regression (Phase 1 always prints the
    /// sub-branch on stdout for the worker role).
    #[error("lernie dispatch {role:?} produced no handle on stdout")]
    EmptyHandle { role: String },
    /// Writing the JSON output to stdout failed.
    #[error("write to stdout: {0}")]
    Write(#[source] io::Error),
}

/// Trait for invoking `lernie dispatch <role>`. Production wires
/// [`SubprocessSpawner`]; tests inject a stub that fabricates the
/// stdout (sub-branch name) without spawning a real subprocess.
pub trait Spawner {
    /// Run `lernie dispatch <role> <repo> <branch> --goal <goal>` and
    /// return the captured stdout (which Phase 1's CLI sets to the
    /// new sub-branch name on the worker role).
    fn dispatch(
        &self,
        role: &str,
        repo: &Path,
        branch: &str,
        goal: &str,
    ) -> Result<DispatchOutput, io::Error>;
}

/// Captured outcome of `lernie dispatch <role>`. Mirrors the
/// `Captured` shape used elsewhere in the executor but stays local to
/// the dispatch tool because the contract is tighter — we always read
/// stdout as text and we report exit non-zero as a typed [`Error`].
#[derive(Debug)]
pub struct DispatchOutput {
    pub stdout: String,
    pub stderr: String,
    pub exit: i32,
}

/// Production [`Spawner`] — re-enters a `lernie` binary as
/// `lernie dispatch <role>`. The dispatch tool itself is `lernie tool
/// dispatch` running in-process; re-entering the same binary keeps
/// the §3.4 "everyone uses the front door" rule intact. The exe path
/// is a field so tests can pin it to `true`/`false` and exercise the
/// wrapper without spawning the real `lernie`; production constructs
/// via [`SubprocessSpawner::new`], which uses `current_exe`.
pub struct SubprocessSpawner {
    exe: PathBuf,
}

impl SubprocessSpawner {
    /// Re-enter the currently running `lernie` binary. Fails when the
    /// OS cannot resolve the current executable (rare; mostly unusual
    /// platforms). Mirrors [`crate::prompt::dispatcher::SpawnDispatcher::new`].
    pub fn new() -> io::Result<Self> {
        Ok(Self {
            exe: std::env::current_exe()?,
        })
    }

    /// Explicit binary path — exposed for tests and embedded callers.
    pub fn with_exe(exe: PathBuf) -> Self {
        Self { exe }
    }
}

impl Spawner for SubprocessSpawner {
    fn dispatch(
        &self,
        role: &str,
        repo: &Path,
        branch: &str,
        goal: &str,
    ) -> Result<DispatchOutput, io::Error> {
        let out = Command::new(&self.exe)
            .args(["dispatch", role])
            .arg(repo)
            .arg(branch)
            .args(["--goal", goal])
            .output()?;
        Ok(DispatchOutput {
            stdout: String::from_utf8_lossy(&out.stdout).into_owned(),
            stderr: String::from_utf8_lossy(&out.stderr).into_owned(),
            exit: out.status.code().unwrap_or(-1),
        })
    }
}

/// Trait for env-var lookup. Production reads `std::env::var`; tests
/// inject a fixed map so the conv-repo / conv-branch values are not
/// dependent on global process state (which `cargo test` runs in
/// parallel with).
pub trait EnvLookup {
    fn get(&self, key: &str) -> Option<OsString>;
}

/// Production [`EnvLookup`] — reads the live process environment.
pub struct ProcessEnv;

impl EnvLookup for ProcessEnv {
    fn get(&self, key: &str) -> Option<OsString> {
        std::env::var_os(key)
    }
}

/// Pure entry point: parse stdin, validate, spawn through `dispatcher`,
/// write the `{status, handle}` JSON to `stdout`. The `lernie tool
/// dispatch` shim wires this to the live process's stdio plus
/// [`ProcessEnv`] + [`SubprocessSpawner`].
pub fn run<R: Read, W: Write>(
    stdin: &mut R,
    stdout: &mut W,
    env: &dyn EnvLookup,
    dispatcher: &dyn Spawner,
) -> Result<(), Error> {
    let mut buf = Vec::new();
    stdin.read_to_end(&mut buf).map_err(Error::StdinRead)?;
    let input: Input = serde_json::from_slice(&buf).map_err(Error::InvalidJson)?;

    let repo = require_env(env, super::super::ENV_CONV_REPO)?;
    let branch = require_env(env, super::super::ENV_CONV_BRANCH)?;
    let repo_path = PathBuf::from(repo);
    let branch_str = branch
        .into_string()
        .map_err(|_| Error::MissingEnv(super::super::ENV_CONV_BRANCH))?;

    validate_role(&repo_path, &input.role)?;

    let captured = dispatcher
        .dispatch(&input.role, &repo_path, &branch_str, &input.goal)
        .map_err(|source| Error::Spawn {
            role: input.role.clone(),
            source,
        })?;
    if captured.exit != 0 {
        return Err(Error::DispatchExit {
            role: input.role,
            exit: captured.exit,
            stderr: captured.stderr,
        });
    }
    let handle = captured.stdout.trim();
    if handle.is_empty() {
        return Err(Error::EmptyHandle { role: input.role });
    }

    let payload = Output {
        status: STATUS_IN_PROGRESS,
        handle,
    };
    let bytes = serde_json::to_vec(&payload).expect("Output is always serializable");
    stdout.write_all(&bytes).map_err(Error::Write)
}

fn require_env(env: &dyn EnvLookup, key: &'static str) -> Result<OsString, Error> {
    env.get(key).ok_or(Error::MissingEnv(key))
}

/// Confirm the role is defined in the conv-repo's `providers.yaml`
/// (`roles:` block, ARCH §4.3) AND that its soul exists at
/// `<conv-repo>/souls/<role>.md` (§4.3 — no path override). Both
/// checks land before the spawn so we fail with a clean typed error
/// instead of a noisy subprocess exit on a doomed call.
fn validate_role(repo: &Path, role: &str) -> Result<(), Error> {
    let providers_path = repo.join(PER_REPO_PROVIDERS_FILE);
    let providers = PerRepoProviders::load(&providers_path)?;
    if !providers.roles.contains_key(role) {
        return Err(Error::RoleMissing {
            role: role.to_string(),
            path: providers_path,
        });
    }
    let soul_path = repo.join(SOULS_DIR).join(format!("{role}{SOUL_SUFFIX}"));
    if !soul_path.exists() {
        return Err(Error::SoulMissing { path: soul_path });
    }
    Ok(())
}

#[cfg(test)]
mod tests;
