//! Cross-config validation: references that span files.
//!
//! - `roles.<role>.{provider,model}` in the per-repo `providers.yaml`
//!   must resolve against the global `models.yaml`, and the named
//!   model's provider row must match the role's declared provider — see
//!   [`check_roles_against_models`].
//! - Workflow `dispatch(<role>)` actions are checked against the
//!   per-repo `roles:` section — see [`check_workflow_against_roles`].

mod roles_check;
mod workflow_check;

pub use roles_check::check_roles_against_models;
