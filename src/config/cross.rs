//! Cross-config validation: references that span files.
//!
//! - `agents.<role>.model` must resolve to a model declared in
//!   `providers.yaml`.
//! - `models.<name>.provider` is enforced inside `providers::Providers::load`,
//!   since both halves live in the same file.
//! - Workflow `dispatch(<role>)` actions are checked against `agents.yaml`.

use crate::config::action::Action;
use crate::config::agents::Agents;
use crate::config::error::LoadError;
use crate::config::providers::Providers;
use crate::config::workflow::Workflow;

/// Validate references from `agents.yaml` into `providers.yaml`.
pub fn check_agents_against_providers(
    agents: &Agents,
    providers: &Providers,
) -> Result<(), LoadError> {
    for (role, definition) in &agents.agents {
        if !providers.models.contains_key(&definition.model) {
            return Err(LoadError::UnresolvedRef {
                key: format!("agents.{role}.model"),
                message: format!(
                    "names model {:?} which is not declared in providers.yaml",
                    definition.model
                ),
            });
        }
    }
    Ok(())
}

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
    use crate::config::providers::Providers;
    use std::io::Write;
    use std::path::Path;
    use tempfile::NamedTempFile;

    fn yaml(s: &str) -> NamedTempFile {
        let mut f = NamedTempFile::new().unwrap();
        f.write_all(s.as_bytes()).unwrap();
        f
    }

    fn load_providers(path: &Path) -> Providers {
        Providers::load(path).unwrap().0
    }

    const PROVIDERS: &str = r#"
providers:
  anthropic:
    endpoint: https://api.anthropic.com
    auth: { type: api_key, env: ANTHROPIC_API_KEY }
models:
  claude-sonnet-4-7:
    provider: anthropic
    model_id: claude-sonnet-4-7
    capabilities: [tool_use_native]
    context_window: 200000
"#;

    #[test]
    fn agents_reference_resolves() {
        let p = load_providers(yaml(PROVIDERS).path());
        let a_file = yaml(
            r#"
agents:
  worker:
    model: claude-sonnet-4-7
    system_prompt: prompts/worker.md
"#,
        );
        let a = Agents::load(a_file.path()).unwrap();
        assert!(check_agents_against_providers(&a, &p).is_ok());
    }

    #[test]
    fn agents_reference_unresolved() {
        let p = load_providers(yaml(PROVIDERS).path());
        let a_file = yaml(
            r#"
agents:
  worker:
    model: claude-sonnet-9000
    system_prompt: prompts/worker.md
"#,
        );
        let a = Agents::load(a_file.path()).unwrap();
        let err = check_agents_against_providers(&a, &p).unwrap_err();
        match err {
            LoadError::UnresolvedRef { key, message } => {
                assert_eq!(key, "agents.worker.model");
                assert!(message.contains("claude-sonnet-9000"));
            }
            other => panic!("expected UnresolvedRef, got {other:?}"),
        }
    }

    #[test]
    fn workflow_dispatch_role_resolves() {
        let a_file = yaml(
            r#"
agents:
  worker:
    model: m
    system_prompt: prompts/w.md
"#,
        );
        let a = Agents::load(a_file.path()).unwrap();
        let w_file = yaml(
            r#"
events:
  user_message:
    - dispatch(worker)
"#,
        );
        let w = Workflow::load(w_file.path()).unwrap();
        assert!(check_workflow_against_agents(&w, &a).is_ok());
    }

    #[test]
    fn workflow_dispatch_role_unresolved() {
        let a_file = yaml(
            r#"
agents:
  worker:
    model: m
    system_prompt: prompts/w.md
"#,
        );
        let a = Agents::load(a_file.path()).unwrap();
        let w_file = yaml(
            r#"
events:
  user_message:
    - dispatch(verifier)
"#,
        );
        let w = Workflow::load(w_file.path()).unwrap();
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
        let a_file = yaml(
            r#"
agents:
  worker:
    model: m
    system_prompt: prompts/w.md
"#,
        );
        let a = Agents::load(a_file.path()).unwrap();
        let w_file = yaml(
            r#"
events:
  user_message:
    - spawn_exchange
    - merge
    - mark_abandoned
    - notify_ui
    - gate_merge_on(verifier.approve)
"#,
        );
        let w = Workflow::load(w_file.path()).unwrap();
        assert!(check_workflow_against_agents(&w, &a).is_ok());
    }
}
