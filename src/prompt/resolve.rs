//! Worker-role config resolution shared by every step-driving verb
//! (ARCH §4.2, §4.3, §6).
//!
//! `lernie prompt` (a fresh root's in-process loop) and `lernie
//! advance` (the §6 hop) issue the same kind of step against the same
//! role config, so the resolution — global `models.yaml`, per-repo
//! `providers.yaml`, `workflow.yaml` policy, the role soul, the adapter
//! binary and its load-time version guard (§4.4) — has one home here.
//! [`WorkerConfig`] is the owned product; [`WorkerConfig::as_resolved`]
//! borrows it into the [`dispatch::Resolved`] shape the step machinery
//! consumes. `lernie advance` resolves *lazily*: a no-op hop (lost
//! acquire, nothing due) exits before any config is read (§6).

use super::{Deps, Error, GLOBAL_MODELS_FILE, PER_REPO_PROVIDERS_FILE, SOULS_DIR, WORKER_ROLE};
use crate::config::{Budgets, Model, ModelsConfig, RetryConfig, Workflow};
use crate::prompt::{AdapterRunner, BRAZEN_PIN, adapter, dispatch};
use std::ffi::OsString;
use std::path::Path;

/// Per-conv-repo control file binding workflow events to actions and
/// declaring the retry policy (ARCH §6).
const WORKFLOW_FILE: &str = "workflow.yaml";

/// The owned resolution of the worker role against one repo: everything
/// a step needs that is not on the branch itself. Owned (not borrowed
/// from a `ModelsConfig`) so callers that resolve lazily — `lernie
/// advance` — can return it from the resolving scope.
#[derive(Clone)]
pub(super) struct WorkerConfig {
    pub(super) model: Model,
    /// brazen provider-row name passed as `bz --provider <row>` (§4.4).
    pub(super) provider_row: String,
    /// The role's declared tool names (§4.3 `tools:`).
    pub(super) tools: Vec<String>,
    pub(super) soul: String,
    /// The adapter binary (`bz` or the `adapter:` override, §4.2).
    pub(super) binary: OsString,
    pub(super) retry: RetryConfig,
    pub(super) budgets: Budgets,
    /// True under an `adapter:` override — the MessageStart.v handshake
    /// governs in place of the version guard (§4.4).
    pub(super) expect_handshake: bool,
}

impl WorkerConfig {
    /// Borrow into the [`dispatch::Resolved`] shape the step machinery
    /// takes (one struct, two drivers — §6 shipped-state note).
    pub(super) fn as_resolved(&self) -> dispatch::Resolved<'_> {
        dispatch::Resolved {
            model: &self.model,
            provider_row: &self.provider_row,
            tools: &self.tools,
            soul: self.soul.clone(),
            binary: self.binary.clone(),
            retry: self.retry,
            budgets: self.budgets,
            expect_handshake: self.expect_handshake,
        }
    }
}

/// Resolve the worker role against `repo`: load configs, run the
/// load-time version guard (§4.4), and read the role soul.
pub(super) fn resolve_worker(repo: &Path, deps: &Deps<'_>) -> Result<WorkerConfig, Error> {
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
        .expect("cross-check passed, so role.model is in models.yaml")
        .clone();

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

    Ok(WorkerConfig {
        model,
        provider_row: assignment.provider.clone(),
        tools: assignment.tools.clone(),
        soul,
        binary,
        retry,
        budgets,
        expect_handshake,
    })
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
