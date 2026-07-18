//! Per-repo `roles:` section against the global `models.yaml`
//! (ARCH §4.3). The role's `model` must be declared in `models.yaml`,
//! and the model's `provider` (a brazen row name) must match the role's
//! declared `provider`. The provider row itself is *not* validated here
//! — its existence is brazen's fact, resolved at call time (§4.1); a
//! missing row is a brazen load-time failure, never a silent fallback.

use crate::config::error::LoadError;
use crate::config::models::Models;
use crate::config::per_repo_providers::PerRepoProviders;

/// Validate references from the per-repo `roles:` section into the
/// global `models.yaml`. Per ARCH §4.3, every role names a model id
/// (which must be declared globally) whose `provider` matches the role's
/// declared provider row.
pub fn check_roles_against_models(
    per_repo: &PerRepoProviders,
    models: &Models,
) -> Result<(), LoadError> {
    for (role, assignment) in &per_repo.roles {
        let Some(model) = models.models.get(&assignment.model) else {
            return Err(LoadError::UnresolvedRef {
                key: format!("roles.{role}.model"),
                message: format!(
                    "names model {:?} which is not declared in the global models.yaml",
                    assignment.model
                ),
            });
        };
        if model.provider != assignment.provider {
            return Err(LoadError::UnresolvedRef {
                key: format!("roles.{role}"),
                message: format!(
                    "model {:?} is served by provider row {:?}, not {:?} as declared",
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

    const TWO_MODEL_GLOBAL: &str = r#"
models:
  claude-sonnet-5:
    provider: anthropic
    model_id: claude-sonnet-5
    capabilities: [tool_use_native]
    context_window: 200000
  claude-haiku-4-5:
    provider: bedrock
    model_id: claude-haiku-4-5
    capabilities: [tool_use_native]
    context_window: 200000
"#;

    fn global() -> Models {
        Models::load(yaml(TWO_MODEL_GLOBAL).path()).unwrap().0
    }

    fn per_repo(s: &str) -> PerRepoProviders {
        PerRepoProviders::load(yaml(s).path()).unwrap()
    }

    #[test]
    fn roles_resolve_against_global_models() {
        let m = global();
        let r = per_repo(
            r#"
roles:
  worker: { provider: anthropic, model: claude-sonnet-5 }
  compactor: { provider: bedrock, model: claude-haiku-4-5 }
"#,
        );
        assert!(check_roles_against_models(&r, &m).is_ok());
    }

    #[test]
    fn roles_unresolved_model() {
        let m = global();
        let r = per_repo("roles:\n  worker: { provider: anthropic, model: claude-sonnet-9000 }\n");
        let err = check_roles_against_models(&r, &m).unwrap_err();
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
        // claude-haiku-4-5 is served by the bedrock row in models.yaml;
        // declaring it under provider 'anthropic' must surface as a
        // distinct error so users can fix the config without guessing.
        let m = global();
        let r = per_repo("roles:\n  worker: { provider: anthropic, model: claude-haiku-4-5 }\n");
        let err = check_roles_against_models(&r, &m).unwrap_err();
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
        assert!(check_roles_against_models(&PerRepoProviders::default(), &global()).is_ok());
    }
}
