//! Configuration files for a conversation repo.
//!
//! Each submodule owns one file per ARCH §2.2 — see [`version`],
//! [`providers`] (global, at the harness root), [`per_repo_providers`]
//! (per-repo `roles:` section), [`manifest`], [`workflow`]. [`cross`]
//! enforces references across files: `roles.*.{provider,model}` must
//! resolve against the global `providers.yaml`, and a workflow's
//! `dispatch(<role>)` action must name a role declared in the
//! per-repo `roles:` section.

#[cfg(test)]
mod tests;

pub mod action;
pub mod cross;
pub mod error;
pub mod manifest;
pub mod models;
pub mod per_repo_providers;
pub mod schemas;
pub mod tool_output;
pub mod version;
pub mod workflow;

pub use action::Action;
pub use error::{LoadError, Warning};
pub use models::{Model, Models};
pub use per_repo_providers::PerRepoProviders;
pub use tool_output::ToolOutputBound;
pub use workflow::{Budgets, CompactionConfig, CompactionTrigger, Event, RetryConfig, Workflow};

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
    /// Load the global half from disk and the per-repo half from content
    /// already in hand — the governing-config read path (ARCH §2.2:
    /// `providers.yaml` is read from the config commit's tree, never
    /// from a worktree file). `per_repo_origin` labels per-repo parse
    /// errors (e.g. `<config-commit>:providers.yaml`).
    pub fn load_with_per_repo(
        global_path: &Path,
        per_repo_raw: &str,
        per_repo_origin: &Path,
    ) -> Result<(Self, Vec<Warning>), LoadError> {
        let (global, warnings) = Models::load(global_path)?;
        let per_repo = PerRepoProviders::parse(per_repo_raw, per_repo_origin)?;
        cross::check_roles_against_models(&per_repo, &global)?;
        Ok((Self { global, per_repo }, warnings))
    }
}
