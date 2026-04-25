//! `agents.yaml` against the (legacy) per-repo `providers.yaml` —
//! retired by Phase 4 of the v0.3 layout migration once `agents.yaml`
//! itself goes away.

use crate::config::agents::Agents;
use crate::config::error::LoadError;
use crate::config::providers::Providers;

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::providers::Providers;
    use std::io::Write;
    use tempfile::NamedTempFile;

    fn yaml(s: &str) -> NamedTempFile {
        let mut f = NamedTempFile::new().unwrap();
        f.write_all(s.as_bytes()).unwrap();
        f
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

    fn load_providers(s: &str) -> Providers {
        Providers::load(yaml(s).path()).unwrap().0
    }

    #[test]
    fn agents_reference_resolves() {
        let p = load_providers(PROVIDERS);
        let a = Agents::load(
            yaml(
                r#"
agents:
  worker:
    model: claude-sonnet-4-7
    system_prompt: prompts/worker.md
"#,
            )
            .path(),
        )
        .unwrap();
        assert!(check_agents_against_providers(&a, &p).is_ok());
    }

    #[test]
    fn agents_reference_unresolved() {
        let p = load_providers(PROVIDERS);
        let a = Agents::load(
            yaml(
                r#"
agents:
  worker:
    model: claude-sonnet-9000
    system_prompt: prompts/worker.md
"#,
            )
            .path(),
        )
        .unwrap();
        let err = check_agents_against_providers(&a, &p).unwrap_err();
        match err {
            LoadError::UnresolvedRef { key, message } => {
                assert_eq!(key, "agents.worker.model");
                assert!(message.contains("claude-sonnet-9000"));
            }
            other => panic!("expected UnresolvedRef, got {other:?}"),
        }
    }
}
