//! Cross-config validation: references that span files.
//!
//! - Workflow `dispatch(<role>)` actions are checked against the
//!   per-repo `roles:` section — see [`check_workflow_against_roles`].
//!
//! There is no roles-against-models check any more (bl-35e2): the
//! global `models.yaml` carries no `models:` table, a role's
//! `providers.yaml` assignment is the single home of its (provider row,
//! model id) pointer, and id validity is brazen's fact caught at the
//! first live model call (ARCH §4.2).

mod workflow_check;

pub use workflow_check::check_workflow_against_roles;
