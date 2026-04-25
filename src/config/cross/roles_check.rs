//! Per-repo `roles:` section against the global `providers.yaml`
//! (ARCH §4.3). The role's `provider` must exist globally; the role's
//! `model` must be in the global model list and belong to the named
//! provider.

use crate::config::error::LoadError;
use crate::config::per_repo_providers::PerRepoProviders;
use crate::config::providers::Providers;

/// Validate references from the per-repo `roles:` section into the
/// global `providers.yaml`. Per ARCH §4.3, every role names a provider
/// (which must exist globally) and a model id (which must belong to
/// that provider's model list).
pub fn check_roles_against_providers(
    per_repo: &PerRepoProviders,
    providers: &Providers,
) -> Result<(), LoadError> {
    for (role, assignment) in &per_repo.roles {
        if !providers.providers.contains_key(&assignment.provider) {
            return Err(LoadError::UnresolvedRef {
                key: format!("roles.{role}.provider"),
                message: format!(
                    "names provider {:?} which is not declared in the global providers.yaml",
                    assignment.provider
                ),
            });
        }
        let Some(model) = providers.models.get(&assignment.model) else {
            return Err(LoadError::UnresolvedRef {
                key: format!("roles.{role}.model"),
                message: format!(
                    "names model {:?} which is not declared in the global providers.yaml",
                    assignment.model
                ),
            });
        };
        if model.provider != assignment.provider {
            return Err(LoadError::UnresolvedRef {
                key: format!("roles.{role}"),
                message: format!(
                    "model {:?} is served by provider {:?}, not {:?} as declared",
                    assignment.model, model.provider, assignment.provider
                ),
            });
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

    const TWO_PROVIDER_GLOBAL: &str = r#"
providers:
  anthropic:
    endpoint: https://api.anthropic.com
    auth: { type: api_key, env: ANTHROPIC_API_KEY }
  bedrock:
    endpoint: https://bedrock.example
    auth: { type: aws_sigv4, profile: default }
models:
  claude-sonnet-4-7:
    provider: anthropic
    model_id: claude-sonnet-4-7
    capabilities: [tool_use_native]
    context_window: 200000
  claude-haiku-4-5:
    provider: bedrock
    model_id: claude-haiku-4-5
    capabilities: [tool_use_native]
    context_window: 200000
"#;

    fn global() -> Providers {
        Providers::load(yaml(TWO_PROVIDER_GLOBAL).path()).unwrap().0
    }

    fn per_repo(s: &str) -> PerRepoProviders {
        PerRepoProviders::load(yaml(s).path()).unwrap().0
    }

    #[test]
    fn roles_resolve_against_global_providers() {
        let p = global();
        let r = per_repo(
            r#"
roles:
  worker: { provider: anthropic, model: claude-sonnet-4-7 }
  compactor: { provider: bedrock, model: claude-haiku-4-5 }
"#,
        );
        assert!(check_roles_against_providers(&r, &p).is_ok());
    }

    #[test]
    fn roles_unresolved_provider() {
        let p = global();
        let r = per_repo("roles:\n  worker: { provider: phantom, model: claude-sonnet-4-7 }\n");
        let err = check_roles_against_providers(&r, &p).unwrap_err();
        match err {
            LoadError::UnresolvedRef { key, message } => {
                assert_eq!(key, "roles.worker.provider");
                assert!(message.contains("phantom"));
            }
            other => panic!("expected UnresolvedRef, got {other:?}"),
        }
    }

    #[test]
    fn roles_unresolved_model() {
        let p = global();
        let r = per_repo("roles:\n  worker: { provider: anthropic, model: claude-sonnet-9000 }\n");
        let err = check_roles_against_providers(&r, &p).unwrap_err();
        match err {
            LoadError::UnresolvedRef { key, message } => {
                assert_eq!(key, "roles.worker.model");
                assert!(message.contains("claude-sonnet-9000"));
            }
            other => panic!("expected UnresolvedRef, got {other:?}"),
        }
    }

    #[test]
    fn roles_model_provider_mismatch() {
        // claude-haiku-4-5 is served by bedrock in the global file;
        // declaring it under provider 'anthropic' must surface as a
        // distinct error so users can fix the config without guessing.
        let p = global();
        let r = per_repo("roles:\n  worker: { provider: anthropic, model: claude-haiku-4-5 }\n");
        let err = check_roles_against_providers(&r, &p).unwrap_err();
        match err {
            LoadError::UnresolvedRef { key, message } => {
                assert_eq!(key, "roles.worker");
                assert!(message.contains("bedrock"));
                assert!(message.contains("anthropic"));
            }
            other => panic!("expected UnresolvedRef, got {other:?}"),
        }
    }

    #[test]
    fn empty_roles_resolve_trivially() {
        assert!(check_roles_against_providers(&PerRepoProviders::default(), &global()).is_ok());
    }
}
