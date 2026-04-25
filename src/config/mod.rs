//! Configuration files for a conversation repo.
//!
//! Each submodule owns one file per ARCH §2.2 — see [`version`],
//! [`providers`] (global, at the harness root), [`per_repo_providers`]
//! (per-repo `roles:` section), [`agents`], [`manifest`], [`workflow`].
//! [`cross`] enforces references across files (e.g. `agents.*.model`
//! must resolve to a defined model; `roles.*.model` must belong to the
//! role's named provider).

pub mod action;
pub mod agents;
pub mod cross;
pub mod error;
pub mod manifest;
pub mod per_repo_providers;
pub mod providers;
pub mod schemas;
pub mod version;
pub mod workflow;

pub use action::{Action, DispatchMode};
pub use agents::{AgentRole, Agents};
pub use error::{LoadError, Warning};
pub use manifest::{Manifest, OverflowPolicy, RoleRules};
pub use per_repo_providers::{PerRepoProviders, RoleAssignment};
pub use providers::{Auth, Capabilities, Model, Provider, Providers};
pub use version::Version;
pub use workflow::{CompactionTrigger, Event, Workflow};

use std::path::Path;

/// The two halves of the provider configuration loaded together: the
/// global file (endpoints, auth, model capabilities — owned by the
/// harness root, ARCH §4.1) and the per-repo `roles:` section (frozen
/// at conversation creation, ARCH §4.3). Cross-references are
/// validated as part of the load — a successful return means every
/// role resolves to a provider/model defined globally.
#[derive(Debug)]
pub struct ProvidersConfig {
    pub global: Providers,
    pub per_repo: PerRepoProviders,
}

impl ProvidersConfig {
    /// Load both halves and cross-validate them in one call. Warnings
    /// from each half are concatenated; capability warnings come first
    /// (global), then legacy-block warnings (per-repo).
    pub fn load(
        global_path: &Path,
        per_repo_path: &Path,
    ) -> Result<(Self, Vec<Warning>), LoadError> {
        let (global, mut warnings) = Providers::load(global_path)?;
        let (per_repo, more) = PerRepoProviders::load(per_repo_path)?;
        warnings.extend(more);
        cross::check_roles_against_providers(&per_repo, &global)?;
        Ok((Self { global, per_repo }, warnings))
    }
}
