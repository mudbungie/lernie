//! Every way driving an agent can fail (ARCH §2, §4.4, §6).
//!
//! One taxonomy for the whole executor — the step loop, the config
//! reads, the adapter, the dispatch gate, the inbox — deliberately
//! narrower than brazen's: wire-level distinctions are brazen's, spoken
//! in band as the `CanonicalError` this enum folds into
//! [`Error::AdapterError`] (§4.4). It lives beside [`super::run`] rather
//! than inside it because it is the module's shared vocabulary, not one
//! function's.

use super::{budget, dispatch, fork_point, inbox};
use crate::prompt::ExecError;
use std::path::PathBuf;
use thiserror::Error as ThisError;

/// Every way [`run`] can fail. The taxonomy is intentionally narrower
/// than brazen's: wire-level distinctions are brazen's, surfaced in-band
/// as the `CanonicalError` this enum folds into [`Error::AdapterError`].
#[derive(Debug, ThisError)]
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
    /// The start named a fork point that resolves to nothing, or named
    /// two (§2.3, [`fork_point`]) — declined before the branch exists.
    #[error(transparent)]
    ForkPoint(#[from] fork_point::Error),
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
    /// The adapter binary is not there. `NotFound` at the spawn is the
    /// one launch failure the user can act on, and the first real
    /// command of every binary-install user hits it — neither `cargo
    /// install lernie` nor the release tarball lays down `bz`. So it
    /// gets the version guard's voice rather than a bare errno: the
    /// binary, the fact, the section, and the literal fix-it command
    /// carrying the linked pin ([`brazen_pin`], the number's one home).
    /// The errno trails as detail.
    #[error(
        "provider adapter {binary:?} not found (§4.4 — the default adapter is `bz` on \
         PATH; install the pinned binary: cargo install brazen --version ={pin} --locked, \
         or name an adapter you have with `adapter:` in the harness root's models.yaml): \
         {source}"
    )]
    AdapterMissing {
        binary: String,
        pin: String,
        #[source]
        source: std::io::Error,
    },
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
    #[error(
        "adapter stream ended without a terminal `end` (killed mid-stream, §2.9); \
         adapter stderr tail: {tail} (full capture: {stderr_log})"
    )]
    AdapterHalfStream { stderr_log: PathBuf, tail: String },
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
    /// The §6 budget gate refused a dispatch before it forked
    /// ([`child_dispatch::run`]) — the declared ceiling would be breached
    /// by the child that does not exist yet. Distinct from the
    /// `budget-exhausted` *terminal* state (§6, [`budget::mark_exhausted`]),
    /// which retires a branch that already exists: nothing was created
    /// here, so there is no branch to mark and no epitaph to deposit.
    #[error(
        "dispatch of {child} from {parent} refused: {exhausted} (§6 budgets — \
         the limit is declared in the governing config's workflow.yaml)"
    )]
    DispatchRefused {
        child: String,
        parent: String,
        exhausted: budget::Exhausted,
    },
    /// A grant the governing config commit does not describe (§3.3),
    /// refused at the fork rather than composed into a smaller toolset.
    #[error(transparent)]
    GrantUndescribed(#[from] dispatch::Undescribed),
    /// A `--name` malformed, id-shaped or taken (§2.3) — refused pre-fork.
    #[error(transparent)]
    NameUnavailable(#[from] crate::workspace::agent_name::Unavailable),
    /// The hop's target has no `agents/*` ref — the shared existence
    /// decline ([`crate::workspace::require_agent`]), fired *before* the
    /// lease so the refusal leaves no inbox directory behind. It is
    /// deliberately distinct from the §2.11 lost-lease no-op: that one
    /// is a live agent already being driven, this one is no agent at all.
    #[error(transparent)]
    UnknownAgent(#[from] crate::workspace::UnknownAgent),
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
