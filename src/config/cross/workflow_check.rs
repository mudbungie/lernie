//! `workflow.yaml` `dispatch(<role>)` actions against `agents.yaml`.

use crate::config::action::Action;
use crate::config::agents::Agents;
use crate::config::error::LoadError;
use crate::config::workflow::Workflow;

/// Validate `dispatch(<role>)` actions in `workflow.yaml` against
/// `agents.yaml`. Assumes `workflow` already passed `Workflow::load`.
pub fn check_workflow_against_agents(
    workflow: &Workflow,
    agents: &Agents,
) -> Result<(), LoadError> {
    for (event, actions) in workflow.typed_events() {
        for (i, action) in actions.into_iter().enumerate() {
            if let Action::Dispatch { role, .. } = action
                && !agents.agents.contains_key(&role)
            {
                return Err(LoadError::UnresolvedRef {
                    key: format!("events.{event:?}[{i}]"),
                    message: format!("dispatch({role}) — role not declared in agents.yaml"),
                });
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    fn yaml(s: &str) -> NamedTempFile {
        let mut f = NamedTempFile::new().unwrap();
        f.write_all(s.as_bytes()).unwrap();
        f
    }

    const WORKER_AGENTS: &str = r#"
agents:
  worker:
    model: m
    system_prompt: prompts/w.md
"#;

    fn worker_agents() -> Agents {
        Agents::load(yaml(WORKER_AGENTS).path()).unwrap()
    }

    #[test]
    fn workflow_dispatch_role_resolves() {
        let a = worker_agents();
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
        assert!(check_workflow_against_agents(&w, &a).is_ok());
    }

    #[test]
    fn workflow_dispatch_role_unresolved() {
        let a = worker_agents();
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
        let err = check_workflow_against_agents(&w, &a).unwrap_err();
        match err {
            LoadError::UnresolvedRef { message, .. } => {
                assert!(message.contains("verifier"));
            }
            other => panic!("expected UnresolvedRef, got {other:?}"),
        }
    }

    #[test]
    fn non_dispatch_actions_are_ignored() {
        let a = worker_agents();
        let w = Workflow::load(
            yaml(
                r#"
events:
  user_message:
    - spawn_exchange
    - merge
    - mark_abandoned
    - notify_ui
    - gate_merge_on(verifier.approve)
"#,
            )
            .path(),
        )
        .unwrap();
        assert!(check_workflow_against_agents(&w, &a).is_ok());
    }
}
