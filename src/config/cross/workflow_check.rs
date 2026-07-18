//! `workflow.yaml` `dispatch(<role>)` actions against the per-repo
//! `providers.yaml` `roles:` section (ARCH §4.3). v0.3 collapses the
//! v0.2 split where role identity lived in `agents.yaml`: the
//! per-repo `roles:` map is the single source of truth for which role
//! names a workflow can dispatch.

use crate::config::action::Action;
use crate::config::error::LoadError;
use crate::config::per_repo_providers::PerRepoProviders;
use crate::config::workflow::Workflow;

/// Validate `dispatch(<role>)` actions in `workflow.yaml` against the
/// per-repo `providers.yaml` `roles:` section. Assumes `workflow`
/// already passed `Workflow::load`.
pub fn check_workflow_against_roles(
    workflow: &Workflow,
    per_repo: &PerRepoProviders,
) -> Result<(), LoadError> {
    for (event, actions) in workflow.typed_events() {
        for (i, action) in actions.into_iter().enumerate() {
            if let Action::Dispatch { role, .. } = action
                && !per_repo.roles.contains_key(&role)
            {
                return Err(LoadError::UnresolvedRef {
                    key: format!("events.{event:?}[{i}]"),
                    message: format!(
                        "dispatch({role}) — role not declared in providers.yaml roles:"
                    ),
                });
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::per_repo_providers::{PerRepoProviders, RoleAssignment};
    use std::collections::BTreeMap;
    use std::io::Write;
    use tempfile::NamedTempFile;

    fn yaml(s: &str) -> NamedTempFile {
        let mut f = NamedTempFile::new().unwrap();
        f.write_all(s.as_bytes()).unwrap();
        f
    }

    fn worker_roles() -> PerRepoProviders {
        let mut roles = BTreeMap::new();
        roles.insert(
            "worker".to_string(),
            RoleAssignment {
                provider: "anthropic".to_string(),
                model: "claude-sonnet-5".to_string(),
                tools: Vec::new(),
            },
        );
        PerRepoProviders { roles }
    }

    #[test]
    fn workflow_dispatch_role_resolves() {
        let r = worker_roles();
        let w = Workflow::load(
            yaml(
                r#"
events:
  user_message:
    - dispatch(worker)
"#,
            )
            .path(),
        )
        .unwrap();
        assert!(check_workflow_against_roles(&w, &r).is_ok());
    }

    #[test]
    fn workflow_dispatch_role_unresolved() {
        let r = worker_roles();
        let w = Workflow::load(
            yaml(
                r#"
events:
  user_message:
    - dispatch(verifier)
"#,
            )
            .path(),
        )
        .unwrap();
        let err = check_workflow_against_roles(&w, &r).unwrap_err();
        match err {
            LoadError::UnresolvedRef { message, .. } => {
                assert!(message.contains("verifier"));
                assert!(message.contains("providers.yaml"));
            }
            other => panic!("expected UnresolvedRef, got {other:?}"),
        }
    }

    #[test]
    fn non_dispatch_actions_are_ignored() {
        let r = worker_roles();
        let w = Workflow::load(
            yaml(
                r#"
events:
  user_message:
    - spawn_exchange
    - compaction_merge
    - deliver_result
    - mark_abandoned
    - notify_ui
    - gate_return_on(verifier.approve)
"#,
            )
            .path(),
        )
        .unwrap();
        assert!(check_workflow_against_roles(&w, &r).is_ok());
    }
}
