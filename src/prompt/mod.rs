//! `lernie prompt` — v0.1 one-exchange-per-invocation backend.
//!
//! The success criterion for ARCH §12 v0.1: "A single prompt is sent to a
//! provider endpoint, the response is written to disk, and is visible in
//! the conversation repo as a commit. No git branching, no tools, no
//! invocations." [`run`] is that path, orchestrated against injected
//! [`AdapterRunner`], [`GitRunner`], [`Clock`], and [`IdGen`] so every
//! branch is exercisable without a live provider or on-disk side effects.
//!
//! v0.1 is the explicit exception to the branch invariant (ARCH §2.3): the
//! exchange is written to `exchanges/<ts>-<short-id>.json` and committed
//! directly on `main`. Branching, steps-as-commits, compaction, and merges
//! land with v0.2.

pub mod adapter;
pub mod clock;
pub mod record;

#[cfg(test)]
mod tests;

pub use adapter::{AdapterRunner, SpawnAdapter};
pub use clock::{Clock, IdGen, NanoIdGen, SystemClock};
pub use record::{ExchangeRecord, Usage};

use crate::config::cross::check_agents_against_providers;
use crate::config::{Agents, Providers};
use crate::provider::anthropic::Response;
use crate::template::GitRunner;
use serde_json::Value;
use std::ffi::OsString;
use std::path::{Path, PathBuf};
use thiserror::Error;

/// Role name resolved from `agents.yaml` to drive the exchange. v0.1 has
/// one role; v0.4 introduces invocations.
const WORKER_ROLE: &str = "worker";
const AGENT_DIR: &str = ".agent";
const SYSTEM_DIR_IN_AGENT: &str = "system";
const EXCHANGES_DIR: &str = "exchanges";
/// Per-request `max_tokens` cap. v0.1 is not opinionated about budget —
/// this is a defensible default the user can outgrow before we add a
/// config surface for it.
const DEFAULT_MAX_TOKENS: u32 = 4096;

/// Every way [`run`] can fail. The taxonomy is intentionally narrower than
/// the provider client's: step-level distinctions (network vs parse vs auth)
/// are the adapter's, not the harness's.
#[derive(Debug, Error)]
pub enum Error {
    #[error("config: {0}")]
    Config(#[from] crate::config::LoadError),
    #[error("agents.yaml has no {0:?} role (required for v0.1)")]
    RoleMissing(String),
    #[error("read system prompt {path}: {source}")]
    SystemPromptRead {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("i/o writing exchange record: {0}")]
    Io(#[from] std::io::Error),
    #[error("adapter subprocess: {0}")]
    AdapterSpawn(#[source] std::io::Error),
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

/// Dependencies [`run`] orchestrates over. Held as `&dyn` so the struct
/// itself carries no generic parameters and tests can pass stubs inline.
pub struct Deps<'a> {
    pub adapter: &'a dyn AdapterRunner,
    pub git: &'a dyn GitRunner,
    pub clock: &'a dyn Clock,
    pub id_gen: &'a dyn IdGen,
}

/// Drive one exchange against `repo`: load configs, invoke the provider
/// adapter, persist the result as a commit. Returns the commit SHA.
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

    let request = serde_json::json!({
        "model": model.model_id,
        "max_tokens": DEFAULT_MAX_TOKENS,
        "system": system_prompt,
        "messages": [{"role": "user", "content": user_message}],
    });
    let request_bytes = serde_json::to_vec(&request).expect("Value is always serializable");

    let started_at = deps.clock.now_iso8601();
    let binary: OsString = format!("lernie-provider-{provider_name}").into();
    // Pass endpoint via CLI — v0.1 pragma. ARCH §4.4 reserves endpoint
    // interpretation to the adapter; a v0.2 refinement (e.g. env-var handoff
    // or describe-driven discovery) can restore the strict reading.
    let args = ["complete", "--endpoint", provider.endpoint.as_str()];
    let stdout_bytes = deps
        .adapter
        .run(&binary, &args, &request_bytes)
        .map_err(Error::AdapterSpawn)?;
    let ended_at = deps.clock.now_iso8601();

    let response = parse_adapter_stdout(&stdout_bytes)?;

    let short_id = deps.id_gen.short();
    let filename = format!(
        "{ts}-{id}.json",
        ts = deps.clock.now_compact(),
        id = short_id
    );
    let record = ExchangeRecord {
        user_message: user_message.to_string(),
        assistant_response: response.text(),
        model_id: model.model_id.clone(),
        provider: provider_name.clone(),
        usage: Usage {
            input_tokens: response.usage.input_tokens,
            output_tokens: response.usage.output_tokens,
        },
        stop_reason: response.stop_reason.clone(),
        started_at,
        ended_at,
    };

    let exchanges_dir = repo.join(EXCHANGES_DIR);
    std::fs::create_dir_all(&exchanges_dir)?;
    let record_bytes = serde_json::to_vec_pretty(&record).expect("record is always serializable");
    std::fs::write(exchanges_dir.join(&filename), &record_bytes)?;

    let rel_path = format!("{EXCHANGES_DIR}/{filename}");
    deps.git
        .run(repo, &["add", rel_path.as_str()])
        .map_err(|source| Error::Git { op: "add", source })?;
    // Commit message references ARCH §12 so the v0.1 exception is
    // traceable in git history; v0.2's first commit drops it.
    let msg = format!("exchange {short_id} [ARCH §12 v0.1]");
    deps.git
        .run(repo, &["commit", "-m", msg.as_str()])
        .map_err(|source| Error::Git {
            op: "commit",
            source,
        })?;
    let sha = deps
        .git
        .run_capture(repo, &["rev-parse", "HEAD"])
        .map_err(|source| Error::Git {
            op: "rev-parse",
            source,
        })?;
    Ok(sha)
}

/// Parse the adapter's stdout bytes into either a
/// [`Response`] or the [`Error::AdapterError`] case. Per ARCH §4.4 the
/// adapter distinguishes error from success by a top-level `{"type":
/// "error"}` sentinel, not by exit code.
fn parse_adapter_stdout(bytes: &[u8]) -> Result<Response, Error> {
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
