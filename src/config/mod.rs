//! Configuration files for a conversation repo.
//!
//! Each submodule owns one file per ARCH §2.2 — see [`version`],
//! [`providers`] (global, at the harness root), [`per_repo_providers`]
//! (per-repo `roles:` section), [`manifest`], [`workflow`]. [`cross`]
//! enforces references across files: `roles.*.{provider,model}` must
//! resolve against the global `providers.yaml`, and a workflow's
//! `dispatch(<role>)` action must name a role declared in the
//! per-repo `roles:` section.

pub mod action;
pub mod cross;
pub mod error;
pub mod manifest;
pub mod models;
pub mod per_repo_providers;
pub mod schemas;
pub mod version;
pub mod workflow;

pub use action::{Action, DispatchMode};
pub use error::{LoadError, Warning};
pub use manifest::{Manifest, OverflowPolicy, RoleRules};
pub use models::{Capabilities, Model, Models};
pub use per_repo_providers::{PerRepoProviders, RoleAssignment};
pub use version::Version;
pub use workflow::{Backoff, CompactionTrigger, Event, RetryConfig, Workflow};

use std::path::Path;

/// The two halves of the model configuration loaded together: the
/// global `models.yaml` (capabilities, context windows, and the
/// optional `adapter:` override — owned by the harness root, ARCH §4.2)
/// and the per-repo `roles:` section (frozen at conversation creation,
/// ARCH §4.3). Cross-references are validated as part of the load — a
/// successful return means every role resolves to a model defined
/// globally whose provider row matches (§4.3).
#[derive(Debug)]
pub struct ModelsConfig {
    pub global: Models,
    pub per_repo: PerRepoProviders,
}

impl ModelsConfig {
    /// Load both halves and cross-validate them in one call. Warnings
    /// come from the global file (capability advice). The per-repo
    /// file is hard-erroring: a legacy `providers:`/`models:` block
    /// fails the load rather than warning.
    pub fn load(
        global_path: &Path,
        per_repo_path: &Path,
    ) -> Result<(Self, Vec<Warning>), LoadError> {
        let (global, warnings) = Models::load(global_path)?;
        let per_repo = PerRepoProviders::load(per_repo_path)?;
        cross::check_roles_against_models(&per_repo, &global)?;
        Ok((Self { global, per_repo }, warnings))
    }
}
