//! Cross-config validation: references that span files.
//!
//! - `roles.<role>.{provider,model}` in the per-repo `providers.yaml`
//!   must resolve against the global `providers.yaml`, and the named
//!   model must belong to the named provider — see
//!   [`check_roles_against_providers`].
//! - `models.<name>.provider` is enforced inside
//!   `providers::Providers::load`, since both halves live in the same
//!   file.
//! - Workflow `dispatch(<role>)` actions are checked against the
//!   per-repo `roles:` section — see [`check_workflow_against_roles`].

mod roles_check;
mod workflow_check;

pub use roles_check::check_roles_against_providers;
pub use workflow_check::check_workflow_against_roles;
