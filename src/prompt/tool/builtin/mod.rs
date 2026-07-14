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
//! [`dispatch`] (the subagent-spawning tool, ARCH §2.5), and the inbox
//! substrate adds [`message`] (deposit content into an existing agent's
//! inbox, ARCH §2.11). [`load_skill`] realizes Body-on-demand (§3.3):
//! it copies a pooled skill directory into the worktree at
//! `skills/<name>/`, committed with the tool result so the next
//! assembly composes it. A dispatch returns the child's address
//! immediately and never blocks; a message deposits synchronously and
//! returns `{status: deposited}`; a load_skill copies and returns
//! `{status: loaded|already_loaded}`. All derive the calling agent's
//! identity from `LERNIE_CONV_BRANCH` (§3.3), never from model input.
//! Adding a new one is a match arm in [`run`] plus a sibling module.

use std::io::{Read, Write};
use thiserror::Error;

pub mod bash;
pub mod dispatch;
pub mod load_skill;
pub mod message;
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
    /// `message` failed (bad input JSON, missing env, `lernie message`
    /// exit non-zero, etc., per [`message::Error`]). Same stderr-concat
    /// contract as the other arms.
    #[error(transparent)]
    Message(#[from] message::Error),
    /// `load_skill` failed (bad input JSON, missing env, unknown skill,
    /// copy failure, etc., per [`load_skill::Error`]). An unknown skill
    /// is a decline that reaches the model as an `is_error` `tool_result`
    /// naming the available pool (§3.3). Same stderr-concat contract.
    #[error(transparent)]
    LoadSkill(#[from] load_skill::Error),
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
// `#[rustfmt::skip]` keeps the `run_with` tail call on one line: exploded
// across arg lines, tarpaulin's llvm engine mis-attributes the argument
// lines as uncovered (a known multi-line-call quirk), and every line here
// is exercised by the routing tests in [`tests`].
#[rustfmt::skip]
pub fn run<R: Read, W: Write, E: Write>(
    name: &str,
    stdin: &mut R,
    stdout: &mut W,
    stderr: &mut E,
) -> Result<i32, Error> {
    // `current_exe` failure here is exotic (mostly unusual platforms /
    // `proc` mounts); panicking is consistent with the harness-wide
    // pattern for unrecoverable startup invariants.
    let spawner = dispatch::SubprocessSpawner::new().expect("current_exe resolves");
    let sender = message::SubprocessSender::new().expect("current_exe resolves");
    run_with(name, stdin, stdout, stderr, &dispatch::ProcessEnv, &spawner, &sender)
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
    sender: &dyn message::Sender,
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
    if name == "message" {
        return message::run(stdin, stdout, env, sender)
            .map(|()| 0)
            .map_err(Error::Message);
    }
    if name == "load_skill" {
        return load_skill::run(stdin, stdout, env)
            .map(|()| 0)
            .map_err(Error::LoadSkill);
    }
    Err(Error::Unknown(name.to_string()))
}

#[cfg(test)]
mod tests;
