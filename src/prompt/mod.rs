//! `lernie prompt` — root-conversation backend (ARCH §2.3).
//!
//! Each prompt spawns an `agents/<conv-id>` branch off the default
//! config branch's head (§2.2–§2.3 — there is no `main`), commits the
//! dispatch commit (§2.10) — which also removes the harness-facing
//! control files from the agent's tree (§2.2) — drives the step loop
//! through brazen's `bz` (§4.4), lands each step's response as attempt
//! segments, and dispatches the terminal compactor off the tip — the
//! compaction merge, the one merge left in the system (§2.6, §2.7).
//! Merge-back is gone: the root branch persists on its own ref (§2.4),
//! and a child returns by depositing a result message into its parent's
//! inbox (§2.6).
//!
//! Provider plumbing follows ARCH §4.4: every model call execs `bz`
//! (`bz --json --provider <row>`, canonical request on stdin, `v=1`
//! events on stdout) once per attempt, with the harness owning the retry
//! loop (§2.10). Auth and endpoints are entirely brazen's; the harness
//! references a provider *row* by name and never sees credential
//! material (§4.1). Config: the global `<harness-root>/models.yaml`
//! carries capabilities / context windows / the optional `adapter:`
//! override (§4.2); the config commit's `providers.yaml` carries the
//! role → (provider row, model, tools) mapping (§4.3), read from the
//! governing config commit (§2.2). Retry policy (attempt cap + backoff)
//! is `workflow.yaml`'s (§6).
//!
//! [`run`] is orchestrated against injected [`AdapterRunner`],
//! [`Sleeper`], [`GitRunner`], [`Clock`], and [`IdGen`] so every branch
//! of the flow is exercisable without a live provider or on-disk side
//! effects.

pub mod adapter;
pub mod budget;
pub mod child_dispatch;
pub mod clock;
pub mod compactor;
pub mod dispatch;
pub mod dispatch_cli;
pub mod inbox;
mod resolve;
pub mod role;
pub mod step;
pub mod stop;
pub mod subagent;
pub mod tool;
mod workflow_actions;

#[cfg(test)]
mod tests;

pub use adapter::{AdapterRunner, SpawnAdapter};
pub use child_dispatch::ChildDispatchRequest;
pub use clock::{Clock, IdGen, NanoIdGen, SystemClock};
pub use dispatch::{RealSleeper, Sleeper, install_stop_handler, stop_flag};
pub use tool::{ExecError, SpawnTool, ToolExecutor};

use crate::template::GitRunner;
use std::path::{Path, PathBuf};
use thiserror::Error;

/// Role name resolved from the config commit's `providers.yaml`
/// (`roles:` block, ARCH §4.3) to drive the root conversation.
pub(crate) const WORKER_ROLE: &str = "worker";
/// Directory in the config commit's tree holding the role souls (ARCH
/// §4.3 — soul = `souls/<role>.md` in the governing config commit).
pub(crate) const SOULS_DIR: &str = "souls";
/// Control file naming the role → (provider row, model, tools)
/// assignments (ARCH §4.3). Read from the governing config commit's
/// tree (§2.2), never from a worktree file.
const PER_REPO_PROVIDERS_FILE: &str = "providers.yaml";
/// Global control file naming model capabilities / context windows and
/// the optional `adapter:` override (ARCH §4.2). Lives at the harness
/// root.
const GLOBAL_MODELS_FILE: &str = "models.yaml";

/// The crate manifest, embedded so [`brazen_pin`] derives from the
/// pin's one home (`Cargo.toml`) instead of mirroring it.
const MANIFEST: &str = include_str!("../../Cargo.toml");

/// The exact brazen pin in a manifest — the version inside
/// `brazen = "=<pin>"`, in either the inline spelling of the source
/// `Cargo.toml` or the `[dependencies.brazen]` / `version = "=<pin>"`
/// table spelling cargo normalizes published manifests into. `None`
/// when the manifest carries no exact brazen pin.
fn parse_brazen_pin(manifest: &str) -> Option<&str> {
    let mut in_brazen_table = false;
    for line in manifest.lines().map(str::trim) {
        if let Some(rest) = line.strip_prefix("brazen = \"=") {
            return rest.strip_suffix('"');
        }
        if line.starts_with('[') {
            in_brazen_table = line == "[dependencies.brazen]";
        } else if in_brazen_table && let Some(rest) = line.strip_prefix("version = \"=") {
            return rest.strip_suffix('"');
        }
    }
    None
}

/// The exact brazen crate version lernie links, read from the
/// `brazen = "=<pin>"` dependency in the embedded `Cargo.toml` — the
/// number's one home (the `make install` pin derives from the same
/// line). The load-time version guard rejects a `bz` whose `--version`
/// differs (§4.4 "Version skew is guarded").
pub fn brazen_pin() -> &'static str {
    parse_brazen_pin(MANIFEST).expect("Cargo.toml pins brazen as `brazen = \"=<version>\"`")
}

/// Every way [`run`] can fail. The taxonomy is intentionally narrower
/// than brazen's: wire-level distinctions are brazen's, surfaced in-band
/// as the `CanonicalError` this enum folds into [`Error::AdapterError`].
#[derive(Debug, Error)]
// `AdapterError` deliberately keeps the suffix; renaming would churn every
// call site for no clarity. The lint only surfaced once the §3.4 narrowing
// made this enum non-exported.
#[allow(clippy::enum_variant_names)]
pub enum Error {
    #[error("config: {0}")]
    Config(#[from] crate::config::LoadError),
    #[error("harness root: {0}")]
    HarnessRoot(#[from] crate::harness_root::Error),
    #[error("providers.yaml has no {0:?} role")]
    RoleMissing(String),
    #[error(transparent)]
    Layout(#[from] crate::workspace::LayoutError),
    #[error("read control {path} from the config commit (§2.2): {source}")]
    ControlRead {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error(
        "model id {0:?} collides with the reserved transcript origin token `tool` (§2.3); \
         rename the model row"
    )]
    ReservedModelId(String),
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
    #[error(
        "adopt LERNIE_LOCK_FD lease for {agent}: {detail} — a bad fd means a defective \
         launcher; declined, never silently reacquired (§6)"
    )]
    LeaseAdopt { agent: String, detail: String },
    #[error(
        "branch {branch} tip is an assistant entry with tool_use unmatched by committed \
         tool results — a mid-step crash after the assistant entry landed; tool side \
         effects are not replayable, so this is declined (§6). Recover by fork-from-history \
         (§2.3)."
    )]
    UnpairedToolUse { branch: String },
    #[error(
        "workflow action {action:?} is not yet interpreted at the {event} event (§6 \
         binding interpreter — shipped subset is the terminal ref marks); the action is \
         in the closed set but its executor is a tracked follow-on of bl-6a3b"
    )]
    ActionUnsupported { action: String, event: &'static str },
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
    /// (the detached `lernie advance` spawn, §2.11/§6); tests inject
    /// a recording launcher so the launch decision and its ordering are
    /// observable.
    pub launcher: &'a dyn inbox::Launcher,
}

/// Drive one root conversation against the workspace at `repo`: check
/// the layout (§2.2 — pre-v1 clean break on the retired
/// per-conversation layout), resolve the worker role against the
/// default config branch's head (the commit the new agent forks off,
/// §2.3), run the load-time version guard, spawn the agent branch, and
/// drive the step loop through `bz`. Returns the agent id (the full
/// hyphenated descent — the branch ref is `agents/<id>`, ARCH §2.3).
pub fn run(repo: &Path, user_message: &str, deps: &Deps<'_>) -> Result<String, Error> {
    crate::workspace::require(repo)?;
    let source = resolve::ConfigSource::ConfigBranch(crate::workspace::DEFAULT_CONFIG_REF);
    let cfg = resolve::resolve_worker(repo, source, deps)?;
    dispatch::run_exchange(repo, user_message, &cfg.as_resolved(), deps)
}
