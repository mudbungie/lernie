//! `lernie prompt` — v0.3 root-conversation backend.
//!
//! v0.3 realizes ARCH §2.3's branch invariant for the root-conversation
//! (user-message) case: each prompt spawns a `<conv-id>` branch off
//! `main` (no `ex/` prefix — the hyphenated descent in the name is
//! self-describing per §2.3), commits a snapshot (§2.10) before the
//! model call, lands the response as a follow-up commit, dispatches the
//! terminal compactor off the tip (§2.7), and `--no-ff` merges the
//! compacted branch back to `main` (§2.6). Main advances by one merge
//! commit per `lernie prompt`.
//!
//! Provider plumbing follows ARCH §4.4 strictly: `describe` runs once
//! per invocation to pick up the adapter's `endpoint_env` list, then
//! each named env var is set to `providers.<name>.endpoint` before
//! `complete`. The harness never reads or interprets the URL.
//!
//! Configuration follows the v0.3 layout (ARCH §2.2, §4.1, §4.3): the
//! per-repo `<conv-repo>/providers.yaml` carries the role → (provider,
//! model) mapping; the global `<harness-root>/providers.yaml` carries
//! endpoints, auth, and model capabilities. Souls live at
//! `<conv-repo>/souls/<role>.md` (§4.3 — no per-role path override).
//!
//! [`run`] is orchestrated against injected [`AdapterRunner`],
//! [`GitRunner`], [`Clock`], and [`IdGen`] so every branch of the
//! flow is exercisable without a live provider or on-disk side
//! effects.

pub mod adapter;
pub mod clock;
pub mod compactor;
pub mod dispatch;
pub mod dispatcher;
pub mod merge;
pub mod step;
pub mod tool;

#[cfg(test)]
mod tests;

pub use adapter::{AdapterRunner, SpawnAdapter};
pub use clock::{Clock, IdGen, NanoIdGen, SystemClock};
pub use compactor::CompactorRequest;
pub use dispatcher::{Dispatcher, SpawnDispatcher};
pub use step::{StepResponse, Usage};
pub use tool::{ExecError, SpawnTool, ToolCall, ToolExecutor, ToolOutcome};

use crate::config::ProvidersConfig;
use crate::provider::wire::Response;
use crate::template::GitRunner;
use serde_json::Value;
use std::path::{Path, PathBuf};
use thiserror::Error;

/// Role name resolved from per-repo `providers.yaml` (`roles:` block,
/// ARCH §4.3) to drive the root conversation. v0.3 has one role; v0.4
/// introduces subagent dispatch which uses the same lookup against
/// other role names.
const WORKER_ROLE: &str = "worker";
/// Per-conv-repo directory holding the role souls (ARCH §4.3 — soul =
/// `<conv-repo>/souls/<role>.md` by convention).
const SOULS_DIR: &str = "souls";
/// Per-conv-repo control file naming the role → (provider, model)
/// assignments (ARCH §4.3). Lives at the conv-repo root, outside any
/// worktree (§2.2 control vs data plane).
const PER_REPO_PROVIDERS_FILE: &str = "providers.yaml";
/// Global control file naming endpoints, auth, and model capabilities
/// (ARCH §4.1). Lives at the harness root and rotates independently of
/// any conversation repo.
const GLOBAL_PROVIDERS_FILE: &str = "providers.yaml";

/// Every way [`run`] can fail. The taxonomy is intentionally narrower
/// than the provider client's: step-level distinctions (network vs
/// parse vs auth) are the adapter's, not the harness's.
#[derive(Debug, Error)]
pub enum Error {
    #[error("config: {0}")]
    Config(#[from] crate::config::LoadError),
    #[error("providers.yaml has no {0:?} role (required for v0.3)")]
    RoleMissing(String),
    #[error("read soul {path}: {source}")]
    SoulRead {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("i/o writing conversation artifact: {0}")]
    Io(#[from] std::io::Error),
    #[error("adapter subprocess: {0}")]
    AdapterSpawn(#[source] std::io::Error),
    #[error("tool {tool}: {source}")]
    ToolExec {
        tool: String,
        #[source]
        source: ExecError,
    },
    #[error("dispatch {role}: {source}")]
    DispatchFailed {
        role: &'static str,
        #[source]
        source: std::io::Error,
    },
    #[error("adapter returned malformed JSON: {0}")]
    AdapterJson(#[source] serde_json::Error),
    #[error("adapter error ({kind}): {message}")]
    AdapterError {
        kind: String,
        message: String,
        http_status: Option<u16>,
    },
    #[error("git {op}: {source}")]
    Git {
        op: &'static str,
        #[source]
        source: std::io::Error,
    },
}

/// Dependencies [`run`] orchestrates over. Held as `&dyn` so the
/// struct itself carries no generic parameters and tests can pass
/// stubs inline. `harness_root` resolves the global `providers.yaml`
/// (ARCH §4.1); production passes [`crate::harness_root::resolve`]'s
/// result, tests pass a temp dir.
pub struct Deps<'a> {
    pub adapter: &'a dyn AdapterRunner,
    pub git: &'a dyn GitRunner,
    pub clock: &'a dyn Clock,
    pub id_gen: &'a dyn IdGen,
    pub dispatcher: &'a dyn Dispatcher,
    pub tool_executor: &'a dyn ToolExecutor,
    pub harness_root: &'a Path,
}

/// Drive one root conversation against `repo`: load configs, spawn the
/// conversation branch, commit the snapshot, invoke the provider
/// adapter, commit the response. Returns the branch name (the bare
/// `<conv-id>`, ARCH §2.3) so callers can locate the two commits
/// without a separate lookup.
pub fn run(repo: &Path, user_message: &str, deps: &Deps<'_>) -> Result<String, Error> {
    let global_path = deps.harness_root.join(GLOBAL_PROVIDERS_FILE);
    let per_repo_path = repo.join(PER_REPO_PROVIDERS_FILE);
    let (cfg, _warnings) = ProvidersConfig::load(&global_path, &per_repo_path)?;

    let assignment = cfg
        .per_repo
        .roles
        .get(WORKER_ROLE)
        .ok_or_else(|| Error::RoleMissing(WORKER_ROLE.to_string()))?;
    // Cross-check inside ProvidersConfig::load guarantees both lookups
    // resolve.
    let model = cfg
        .global
        .models
        .get(&assignment.model)
        .expect("cross-check passed, so role.model is in providers.models");
    let provider = cfg
        .global
        .providers
        .get(&assignment.provider)
        .expect("cross-check passed, so role.provider is in providers.providers");

    let soul_path = repo.join(SOULS_DIR).join(format!("{WORKER_ROLE}.md"));
    let soul = std::fs::read_to_string(&soul_path).map_err(|source| Error::SoulRead {
        path: soul_path.clone(),
        source,
    })?;

    let resolved = dispatch::Resolved {
        model,
        provider_name: &assignment.provider,
        provider,
        soul,
    };
    dispatch::run_exchange(repo, user_message, &resolved, deps)
}

/// Parse the `endpoint_env` field out of the adapter's `describe`
/// JSON. Missing field is an empty list — an adapter that does not
/// advertise `endpoint_env` opts out of harness-set endpoints and
/// uses its built-in default. Wrong-typed values surface as
/// [`Error::AdapterJson`].
pub(super) fn parse_endpoint_env(bytes: &[u8]) -> Result<Vec<String>, Error> {
    let value: Value = serde_json::from_slice(bytes).map_err(Error::AdapterJson)?;
    let Some(field) = value.get("endpoint_env") else {
        return Ok(Vec::new());
    };
    serde_json::from_value(field.clone()).map_err(Error::AdapterJson)
}

/// Parse the adapter's stdout bytes into either a [`Response`] or the
/// [`Error::AdapterError`] case. Per ARCH §4.4 the adapter
/// distinguishes error from success by a top-level `{"type":
/// "error"}` sentinel, not by exit code.
pub(super) fn parse_adapter_stdout(bytes: &[u8]) -> Result<Response, Error> {
    let value: Value = serde_json::from_slice(bytes).map_err(Error::AdapterJson)?;
    if value.get("type").and_then(Value::as_str) == Some("error") {
        return Err(Error::AdapterError {
            kind: value
                .get("kind")
                .and_then(Value::as_str)
                .unwrap_or("unknown")
                .to_string(),
            message: value
                .get("message")
                .and_then(Value::as_str)
                .unwrap_or("")
                .to_string(),
            http_status: value
                .get("http_status")
                .and_then(Value::as_u64)
                .map(|n| n as u16),
        });
    }
    serde_json::from_value(value).map_err(Error::AdapterJson)
}
