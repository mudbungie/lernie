//! Built-in tools — the in-process implementations behind
//! `lernie tool <name>` (ARCH §3.3, §12 v0.3 toolset).
//!
//! Each tool is a pure function over [`Read`]/[`Write`] so unit tests
//! drive it without touching real stdio. The `lernie tool` subcommand
//! is a thin shim that locks the process's stdio handles and delegates
//! to [`run`]; the §3.3 stdio contract (stdin = `tool_use.input` JSON,
//! stdout = raw result bytes, exit code = is_error) is enforced here.
//!
//! v0.3 shipped two built-ins (`read_file`, `bash`); v0.4 Phase 2 adds
//! [`dispatch`] (the subagent-spawning tool, ARCH §2.5). A dispatch
//! returns the child's address immediately and never blocks: the
//! child's result arrives later as an inbox deposit (§2.11), so there
//! is no polling half to pair with it. Adding a new one is a match arm
//! in [`run`] plus a sibling module.

use std::io::{Read, Write};
use thiserror::Error;

pub mod bash;
pub mod dispatch;
pub mod read_file;

/// Reasons [`run`] can fail. Each in-process tool surfaces its own
/// error variant; an unknown tool name is the dispatcher-level case.
#[derive(Debug, Error)]
pub enum Error {
    /// The lernie binary was invoked as `lernie tool <name>` for a
    /// `<name>` that isn't a built-in. The harness only routes here
    /// after external resolution misses (§3.3), so this is "no tool
    /// of that name exists at all".
    #[error("unknown built-in tool: {0:?}")]
    Unknown(String),
    /// `read_file` failed; carries the inner reason for the operator's
    /// `eprintln!`. The §3.3 stdio contract concats stderr after
    /// stdout into `tool_result.content` when exit code is non-zero,
    /// so the message reaches the model verbatim.
    #[error(transparent)]
    ReadFile(#[from] read_file::Error),
    /// `bash` failed at the harness layer (bad input JSON, spawn
    /// failure, broken pipe, etc.). In-band shell failures — the
    /// command ran and exited non-zero — are *not* this variant; they
    /// flow through the returned exit code.
    #[error(transparent)]
    Bash(#[from] bash::Error),
    /// `dispatch` failed (bad input JSON, missing role / soul,
    /// `lernie dispatch <role>` exit non-zero, etc., per
    /// [`dispatch::Error`]). The §3.3 stdio contract concats stderr
    /// after stdout so the agent sees the failure verbatim.
    #[error(transparent)]
    Dispatch(#[from] dispatch::Error),
}

/// Dispatch one in-process tool call. `name` is the tool name as the
/// model spelled it (and as the harness passed via `lernie tool
/// <name>`); `stdin` carries the `tool_use.input` JSON; `stdout`
/// receives the bytes the executor will surface as
/// `tool_result.content` on success; `stderr` receives the bytes that
/// — per §3.3 — concatenate after stdout when the exit code is
/// non-zero. The returned `i32` is the desired process exit code:
/// `read_file` always returns 0 on success and lets [`Error`] carry
/// failure; `bash` propagates the shell's own exit code so a non-zero
/// command can flow through without being misclassified as a harness
/// fault.
pub fn run<R: Read, W: Write, E: Write>(
    name: &str,
    stdin: &mut R,
    stdout: &mut W,
    stderr: &mut E,
) -> Result<i32, Error> {
    // `current_exe` failure here is exotic (mostly unusual platforms
    // / `proc` mounts); panicking is consistent with the harness-wide
    // pattern for unrecoverable startup invariants.
    let spawner = dispatch::SubprocessSpawner::new().expect("current_exe resolves");
    run_with(name, stdin, stdout, stderr, &dispatch::ProcessEnv, &spawner)
}

/// Same as [`run`] but with the `dispatch`-tool dependencies (env
/// lookup + subprocess spawner) injected. Production wires these to
/// [`dispatch::ProcessEnv`] + [`dispatch::SubprocessSpawner`] via
/// [`run`]; tests inject stubs to exercise the dispatch arm without
/// real subprocess fan-out.
pub fn run_with<R: Read, W: Write, E: Write>(
    name: &str,
    stdin: &mut R,
    stdout: &mut W,
    stderr: &mut E,
    env: &dyn dispatch::EnvLookup,
    spawner: &dyn dispatch::Spawner,
) -> Result<i32, Error> {
    if name == "read_file" {
        return read_file::run(stdin, stdout)
            .map(|()| 0)
            .map_err(Error::ReadFile);
    }
    if name == "bash" {
        return bash::run(stdin, stdout, stderr).map_err(Error::Bash);
    }
    if name == "dispatch" {
        return dispatch::run(stdin, stdout, env, spawner)
            .map(|()| 0)
            .map_err(Error::Dispatch);
    }
    Err(Error::Unknown(name.to_string()))
}

#[cfg(test)]
mod tests;
