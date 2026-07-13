//! `lernie prompt` — root-conversation backend (ARCH §2.3).
//!
//! Each prompt spawns a `<conv-id>` branch off `main`, commits the
//! dispatch snapshot (§2.10), drives the step loop through brazen's `bz`
//! (§4.4), lands each step's response as attempt segments, and dispatches
//! the terminal compactor off the tip — the compaction merge, the one
//! merge left in the system (§2.6, §2.7). Merge-back is gone: the root
//! branch persists on its own ref (§2.4), and a child returns by
//! depositing a result message into its parent's inbox (§2.6).
//!
//! Provider plumbing follows ARCH §4.4: every model call execs `bz`
//! (`bz --json --provider <row>`, canonical request on stdin, `v=1`
//! events on stdout) once per attempt, with the harness owning the retry
//! loop (§2.10). Auth and endpoints are entirely brazen's; the harness
//! references a provider *row* by name and never sees credential
//! material (§4.1). Config: the global `<harness-root>/models.yaml`
//! carries capabilities / context windows / the optional `adapter:`
//! override (§4.2); the per-repo `<conv-repo>/providers.yaml` carries
//! the role → (provider row, model, tools) mapping (§4.3). Retry policy
//! (attempt cap + backoff) is `workflow.yaml`'s (§6).
//!
//! [`run`] is orchestrated against injected [`AdapterRunner`],
//! [`Sleeper`], [`GitRunner`], [`Clock`], and [`IdGen`] so every branch
//! of the flow is exercisable without a live provider or on-disk side
//! effects.

pub mod adapter;
pub mod budget;
pub mod clock;
pub mod compactor;
pub mod dispatch;
pub mod dispatcher;
pub mod inbox;
pub mod step;
pub mod stop;
pub mod subagent;
pub mod tool;
pub mod worker;

#[cfg(test)]
mod tests;

pub use adapter::{AdapterRunner, SpawnAdapter};
pub use clock::{Clock, IdGen, NanoIdGen, SystemClock};
pub use compactor::CompactorRequest;
pub use dispatch::{RealSleeper, Sleeper, install_stop_handler, stop_flag};
pub use dispatcher::{Dispatcher, SpawnDispatcher};
pub use tool::{ExecError, SpawnTool, ToolCall, ToolExecutor, ToolOutcome};
pub use worker::WorkerRequest;

use crate::config::{Budgets, ModelsConfig, RetryConfig, Workflow};
use crate::template::GitRunner;
use std::ffi::OsString;
use std::path::{Path, PathBuf};
use thiserror::Error;

/// Role name resolved from per-repo `providers.yaml` (`roles:` block,
/// ARCH §4.3) to drive the root conversation.
const WORKER_ROLE: &str = "worker";
/// Per-conv-repo directory holding the role souls (ARCH §4.3 — soul =
/// `<conv-repo>/souls/<role>.md` by convention).
pub(crate) const SOULS_DIR: &str = "souls";
/// Per-conv-repo control file naming the role → (provider row, model,
/// tools) assignments (ARCH §4.3). Lives at the conv-repo root, outside
/// any worktree (§2.2 control vs data plane).
const PER_REPO_PROVIDERS_FILE: &str = "providers.yaml";
/// Global control file naming model capabilities / context windows and
/// the optional `adapter:` override (ARCH §4.2). Lives at the harness
/// root.
const GLOBAL_MODELS_FILE: &str = "models.yaml";
/// Per-conv-repo control file binding workflow events to actions and
/// declaring the retry policy (ARCH §6).
const WORKFLOW_FILE: &str = "workflow.yaml";

/// The exact brazen crate version lernie links (`brazen = "=0.0.2"` in
/// `Cargo.toml`). The load-time version guard rejects a `bz` whose
/// `--version` differs (§4.4 "Version skew is guarded"). This pin and
/// the `Cargo.toml` dependency are the two homes of the number; keep
/// them in lockstep (also the `make install` pin).
pub const BRAZEN_PIN: &str = "0.0.2";

/// Every way [`run`] can fail. The taxonomy is intentionally narrower
/// than brazen's: wire-level distinctions are brazen's, surfaced in-band
/// as the `CanonicalError` this enum folds into [`Error::AdapterError`].
#[derive(Debug, Error)]
pub enum Error {
    #[error("config: {0}")]
    Config(#[from] crate::config::LoadError),
    #[error("providers.yaml has no {0:?} role")]
    RoleMissing(String),
    #[error(
        "model id {0:?} collides with the reserved transcript origin token `tool` (§2.3); \
         rename the model row"
    )]
    ReservedModelId(String),
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
    #[error("adapter emitted malformed v=1 event JSON: {0}")]
    AdapterJson(#[source] serde_json::Error),
    #[error("provider error ({kind}): {message}")]
    AdapterError { kind: String, message: String },
    #[error("adapter stream ended without a terminal `end` (killed mid-stream, §2.9)")]
    AdapterHalfStream,
    #[error(
        "bz version {found:?} does not match the linked brazen crate {expected:?} \
         (§4.4 — install the pinned binary: cargo install brazen --version ={expected})"
    )]
    VersionSkew { found: String, expected: String },
    #[error("adapter-override handshake failed: MessageStart.v={found:?}, expected {expected}")]
    HandshakeMismatch { found: Option<u8>, expected: u8 },
    #[error("git {op}: {source}")]
    Git {
        op: &'static str,
        #[source]
        source: std::io::Error,
    },
    #[error("acquire executor lock on inbox {path}: {source}")]
    ExecutorLock {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("deposit initial user message: {0}")]
    Deposit(#[from] inbox::DepositError),
    #[error("tool {name} schema unreadable at {path}: {source}")]
    ToolSchemaIo {
        name: String,
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("tool {name} schema at {path} is not valid JSON: {source}")]
    ToolSchemaJson {
        name: String,
        path: PathBuf,
        #[source]
        source: serde_json::Error,
    },
    #[error("tool {name} skill frontmatter unreadable at {path}: {source}")]
    SkillFrontmatterIo {
        name: String,
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("tool {name} skill frontmatter at {path} is malformed: {source}")]
    SkillFrontmatter {
        name: String,
        path: PathBuf,
        #[source]
        source: serde_yaml_ng::Error,
    },
}

/// Dependencies [`run`] orchestrates over. Held as `&dyn` so the
/// struct itself carries no generic parameters and tests can pass
/// stubs inline. `config_root` is the config-lifetime harness root
/// (ARCH §2.2), which holds the global `models.yaml` (ARCH §4.2);
/// production passes [`crate::harness_root::Roots::config`], tests pass
/// a temp dir. The data-lifetime root reaches [`run`] only through
/// `tool_executor`, which already carries it.
pub struct Deps<'a> {
    pub adapter: &'a dyn AdapterRunner,
    pub sleeper: &'a dyn Sleeper,
    pub git: &'a dyn GitRunner,
    pub clock: &'a dyn Clock,
    pub id_gen: &'a dyn IdGen,
    pub dispatcher: &'a dyn Dispatcher,
    pub tool_executor: &'a dyn ToolExecutor,
    pub config_root: &'a Path,
    /// The executor's SIGTERM flag (ARCH §2.9 step 3), observed at the
    /// step-loop check points. Production wires the process-wide
    /// [`dispatch::stop_flag`] after [`dispatch::install_stop_handler`];
    /// tests inject a constructed [`std::sync::atomic::AtomicBool`] so the
    /// stopped-deposit path is exercised without a real signal.
    pub stop: &'a std::sync::atomic::AtomicBool,
    /// The driver launcher for the §2.11 exit protocol's self-directed
    /// launch, fired after the executor releases its lock on a
    /// final-response exit. Production wires [`inbox::AdvanceLauncher`]
    /// (the documented no-op pending `lernie advance`, §6); tests inject
    /// a recording launcher so the launch decision and its ordering are
    /// observable.
    pub launcher: &'a dyn inbox::Launcher,
}

/// Drive one root conversation against `repo`: load configs, run the
/// load-time version guard, spawn the conversation branch, drive the
/// step loop through `bz`, and merge back. Returns the branch name (the
/// bare `<conv-id>`, ARCH §2.3).
pub fn run(repo: &Path, user_message: &str, deps: &Deps<'_>) -> Result<String, Error> {
    let global_path = deps.config_root.join(GLOBAL_MODELS_FILE);
    let per_repo_path = repo.join(PER_REPO_PROVIDERS_FILE);
    let (cfg, _warnings) = ModelsConfig::load(&global_path, &per_repo_path)?;

    let assignment = cfg
        .per_repo
        .roles
        .get(WORKER_ROLE)
        .ok_or_else(|| Error::RoleMissing(WORKER_ROLE.to_string()))?;
    // Cross-check inside ModelsConfig::load guarantees this resolves.
    let model = cfg
        .global
        .models
        .get(&assignment.model)
        .expect("cross-check passed, so role.model is in models.yaml");

    // Adapter resolution (§4.2): the optional `adapter:` override, else
    // `bz` on PATH. The version guard runs only for the default binary;
    // an override is governed by the in-band MessageStart.v handshake.
    let adapter_override = cfg.global.adapter.as_deref();
    let binary = adapter::resolve_binary(adapter_override);
    let expect_handshake = adapter_override.is_some();
    if !expect_handshake {
        check_bz_version(deps.adapter, &binary)?;
    }

    let (retry, budgets) = load_workflow_policy(repo)?;

    let soul_path = repo.join(SOULS_DIR).join(format!("{WORKER_ROLE}.md"));
    let soul = std::fs::read_to_string(&soul_path).map_err(|source| Error::SoulRead {
        path: soul_path.clone(),
        source,
    })?;

    let resolved = dispatch::Resolved {
        model,
        provider_row: &assignment.provider,
        tools: &assignment.tools,
        soul,
        binary,
        retry,
        budgets,
        expect_handshake,
    };
    dispatch::run_exchange(repo, user_message, &resolved, deps)
}

/// Load the harness-owned retry policy and the per-conversation budgets
/// from `workflow.yaml` (§6; retry §2.10, budgets §6). Parsed together
/// from the one frozen copy so the file is read once.
fn load_workflow_policy(repo: &Path) -> Result<(RetryConfig, Budgets), Error> {
    let workflow = Workflow::load(&repo.join(WORKFLOW_FILE))?;
    Ok((workflow.retry, workflow.budgets))
}

/// Load-time version guard (§4.4): `bz --version` must report the exact
/// version of the linked brazen crate ([`BRAZEN_PIN`]); a mismatch is
/// declined (PRINCIPLES "Decline illegal operations") rather than
/// silently downgraded.
fn check_bz_version(adapter: &dyn AdapterRunner, binary: &OsString) -> Result<(), Error> {
    let out =
        adapter::capture_stdout(adapter, binary, &["--version"]).map_err(Error::AdapterSpawn)?;
    // `bz --version` prints e.g. `bz 0.0.2`; the version is the last
    // whitespace token.
    let found = out.split_whitespace().last().unwrap_or("").to_string();
    if found != BRAZEN_PIN {
        return Err(Error::VersionSkew {
            found,
            expected: BRAZEN_PIN.to_string(),
        });
    }
    Ok(())
}
