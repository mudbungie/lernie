//! `cd` built-in (ARCH §3.3 *Working directory*) — the one way an
//! agent's working directory changes.
//!
//! Stdin is the `tool_use.input` block as JSON: `{ "path": <string> }`.
//! Stdout is `{ "cwd": "<absolute path>" }`, the directory every
//! subsequent tool call of this agent will run in.
//!
//! **Relative paths need no resolution here.** The executor spawned this
//! process *in* the agent's current working directory (§3.3), so
//! `canonicalize` resolves a relative `path` against exactly the
//! directory the model meant, symlinks and `..` included — the kernel's
//! answer, not a re-derivation of one. A path that names nothing, or
//! names something that is not a directory, is declined as an `is_error`
//! `tool_result`; nothing is stored and the agent stays where it was.
//!
//! **The new directory is stored as the agent's working-directory mark**
//! ([`crate::workspace::cwd`], `refs/lernie/cwd/<agent-id>`), read back
//! by the executor at every later spawn. The calling agent's workspace +
//! branch arrive via `LERNIE_CONV_REPO` / `LERNIE_CONV_BRANCH` (§3.3,
//! harness-derived) — never from model input, so an agent can move only
//! itself.
//!
//! **No containment.** The target may be any directory on the machine:
//! v1.0 bounds a tool's authority nowhere (§3.6 defers that to the v1.1
//! sandbox, on the artifact and uniformly), and `bash` could already
//! reach outside the worktree with an absolute path. What moving does
//! change is the **work-product boundary**: the tool commit stages the
//! worktree (`git add -A`, §3.3), so edits an agent makes elsewhere are
//! real but uncommitted — off its branch, invisible to a parent (§2.6)
//! and absent from replay (§9.2). The tool definition says so.

use serde::{Deserialize, Serialize};
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
use thiserror::Error;

use super::super::{ENV_CONV_BRANCH, ENV_CONV_REPO};
use super::dispatch::EnvLookup;
use crate::template::{GitRunner, RealGit};
use crate::workspace;

/// Wire shape of the input. `deny_unknown_fields` so a malformed
/// `tool_use.input` surfaces as [`Error::InvalidJson`] rather than a
/// silent drop.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Input {
    path: String,
}

/// Wire shape of the output — the one fact the call produces. There is no
/// `status` field: the call either moved the agent or declined, and a
/// constant carries nothing.
#[derive(Debug, Serialize, PartialEq, Eq)]
struct Output {
    cwd: String,
}

/// Every way [`run`] can fail. Each prints its own stderr message; per
/// §3.3 stderr concatenates after stdout into `tool_result.content` on a
/// non-zero exit, so the model reads the decline verbatim.
#[derive(Debug, Error)]
pub enum Error {
    #[error("invalid input JSON: {0}")]
    InvalidJson(#[source] serde_json::Error),
    #[error("read input from stdin: {0}")]
    StdinRead(#[source] io::Error),
    #[error("missing env var {0:?} (set by the harness per ARCH §3.3)")]
    MissingEnv(&'static str),
    #[error("no such directory {path:?}: {source}")]
    NoSuchDir {
        path: String,
        #[source]
        source: io::Error,
    },
    #[error("{path:?} is not a directory — cd takes a directory (ARCH §3.3)")]
    NotADir { path: String },
    #[error("store the working directory: {0}")]
    Mark(#[source] io::Error),
    #[error("write to stdout: {0}")]
    Write(#[source] io::Error),
}

/// Production entry point invoked by `lernie tool cd`. The mark is
/// written through the real git, injected for tests by [`run_with`].
pub fn run<R: Read, W: Write>(
    stdin: &mut R,
    stdout: &mut W,
    env: &dyn EnvLookup,
) -> Result<(), Error> {
    run_with(stdin, stdout, env, &RealGit::new())
}

/// [`run`] with the git runner injected.
pub fn run_with<R: Read, W: Write>(
    stdin: &mut R,
    stdout: &mut W,
    env: &dyn EnvLookup,
    git: &dyn GitRunner,
) -> Result<(), Error> {
    let mut buf = Vec::new();
    stdin.read_to_end(&mut buf).map_err(Error::StdinRead)?;
    let input: Input = serde_json::from_slice(&buf).map_err(Error::InvalidJson)?;

    let repo = env
        .get(ENV_CONV_REPO)
        .ok_or(Error::MissingEnv(ENV_CONV_REPO))?;
    let branch = env
        .get(ENV_CONV_BRANCH)
        .and_then(|v| v.into_string().ok())
        .ok_or(Error::MissingEnv(ENV_CONV_BRANCH))?;
    let target = resolve(&input.path)?;

    let workspace = PathBuf::from(repo);
    workspace::cwd::write(&workspace, &branch, &target, git).map_err(Error::Mark)?;
    emit(stdout, &target)
}

/// The absolute directory `path` names, resolved against this process's
/// own cwd — which the executor set to the agent's current working
/// directory (§3.3). Declines a path that names nothing and a path that
/// names a non-directory, separately: they are different mistakes.
fn resolve(path: &str) -> Result<PathBuf, Error> {
    let abs = std::fs::canonicalize(path).map_err(|source| Error::NoSuchDir {
        path: path.to_owned(),
        source,
    })?;
    match abs.is_dir() {
        true => Ok(abs),
        false => Err(Error::NotADir {
            path: path.to_owned(),
        }),
    }
}

/// Serialize the `{cwd}` result to `stdout` (§3.3). The path is UTF-8 by
/// construction — [`workspace::cwd::write`] declined it otherwise.
fn emit<W: Write>(stdout: &mut W, dir: &Path) -> Result<(), Error> {
    let payload = Output {
        cwd: dir.to_string_lossy().into_owned(),
    };
    let bytes = serde_json::to_vec(&payload).expect("Output is always serializable");
    stdout.write_all(&bytes).map_err(Error::Write)
}

#[cfg(test)]
mod tests;
