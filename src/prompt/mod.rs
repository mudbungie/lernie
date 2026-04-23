//! `lernie prompt` — v0.2 exchange-branch backend.
//!
//! v0.2 realizes ARCH §2.3's branch invariant for the exchange case:
//! each invocation spawns `ex/<ts>-<short-id>` off `main`, commits a
//! snapshot (§2.10) before the model call, lands the response as a
//! follow-up commit, dispatches the terminal compactor off the tip
//! (§2.7), and `--no-ff` merges the compacted branch back to `main`
//! (§2.6). Main advances by one merge commit per `lernie prompt`.
//!
//! Provider plumbing follows ARCH §4.4 strictly: `describe` runs once
//! per invocation to pick up the adapter's `endpoint_env` list, then
//! each named env var is set to `providers.<name>.endpoint` before
//! `complete`. The harness never reads or interprets the URL.
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

#[cfg(test)]
mod tests;

pub use adapter::{AdapterRunner, SpawnAdapter};
pub use clock::{Clock, IdGen, NanoIdGen, SystemClock};
pub use compactor::CompactorRequest;
pub use dispatcher::{Dispatcher, SpawnDispatcher};
pub use step::{StepResponse, Usage};

use crate::config::cross::check_agents_against_providers;
use crate::config::{Agents, Providers};
use crate::provider::anthropic::Response;
use crate::template::GitRunner;
use serde_json::Value;
use std::path::{Path, PathBuf};
use thiserror::Error;

/// Role name resolved from `agents.yaml` to drive the exchange. v0.2
/// has one role; v0.4 introduces invocations.
const WORKER_ROLE: &str = "worker";
pub(crate) const AGENT_DIR: &str = ".agent";
const SYSTEM_DIR_IN_AGENT: &str = "system";

/// Every way [`run`] can fail. The taxonomy is intentionally narrower
/// than the provider client's: step-level distinctions (network vs
/// parse vs auth) are the adapter's, not the harness's.
#[derive(Debug, Error)]
pub enum Error {
    #[error("config: {0}")]
    Config(#[from] crate::config::LoadError),
    #[error("agents.yaml has no {0:?} role (required for v0.2)")]
    RoleMissing(String),
    #[error("read system prompt {path}: {source}")]
    SystemPromptRead {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("i/o writing exchange artifact: {0}")]
    Io(#[from] std::io::Error),
    #[error("adapter subprocess: {0}")]
    AdapterSpawn(#[source] std::io::Error),
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
/// stubs inline.
pub struct Deps<'a> {
    pub adapter: &'a dyn AdapterRunner,
    pub git: &'a dyn GitRunner,
    pub clock: &'a dyn Clock,
    pub id_gen: &'a dyn IdGen,
    pub dispatcher: &'a dyn Dispatcher,
}

/// Drive one exchange against `repo`: load configs, spawn the
/// exchange branch, commit the snapshot, invoke the provider adapter,
/// commit the response. Returns the branch name (`ex/<ts>-<id>`) so
/// callers can locate the two commits without a separate lookup.
pub fn run(repo: &Path, user_message: &str, deps: &Deps<'_>) -> Result<String, Error> {
    let agent_dir = repo.join(AGENT_DIR);
    let (providers, _warnings) = Providers::load(&agent_dir.join("providers.yaml"))?;
    let agents = Agents::load(&agent_dir.join("agents.yaml"))?;
    check_agents_against_providers(&agents, &providers)?;

    let role = agents
        .agents
        .get(WORKER_ROLE)
        .ok_or_else(|| Error::RoleMissing(WORKER_ROLE.to_string()))?;
    // Cross-check above guarantees both lookups resolve.
    let model = providers
        .models
        .get(&role.model)
        .expect("cross-check passed, so role.model is in providers.models");
    let provider_name = &model.provider;
    let provider = providers
        .providers
        .get(provider_name)
        .expect("providers.yaml load validates model.provider exists");

    let system_prompt_path = agent_dir
        .join(SYSTEM_DIR_IN_AGENT)
        .join(&role.system_prompt);
    let system_prompt =
        std::fs::read_to_string(&system_prompt_path).map_err(|source| Error::SystemPromptRead {
            path: system_prompt_path.clone(),
            source,
        })?;

    let resolved = dispatch::Resolved {
        model,
        provider_name,
        provider,
        system_prompt,
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
