//! Per-repo `providers.yaml` — role → (provider, model) assignments
//! frozen at conversation creation (ARCH §4.3).
//!
//! The conversation-repo file carries only the `roles:` section: which
//! provider name and which model id each role dispatches to. Endpoint
//! and auth resolve inside brazen at call time (never a harness file);
//! model capabilities live in the global `<harness-root>/models.yaml`
//! and rotate independently (ARCH §4.1).
//!
//! A legacy `providers:` or `models:` block (the v0.2 shape) is a hard
//! load error: those sections belong to the global file only, and a
//! per-repo file carrying them is structurally wrong rather than just
//! noisy. (Phase 1 of the v0.3 layout migration warned; Phase 4
//! escalated to error once the v0.2 template was retired.)

use crate::config::error::LoadError;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

/// Top-level shape of the per-repo `providers.yaml`.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq, Default)]
pub struct PerRepoProviders {
    #[serde(default)]
    pub roles: BTreeMap<String, RoleAssignment>,
}

/// One role's assignment: which provider (by brazen row name) and which
/// model (by id in the global `models.yaml`), plus the role's enabled
/// tools (ARCH §4.3). `provider`/`model` are validated cross-file in
/// [`crate::config::cross::check_roles_against_models`]; endpoint and
/// auth resolve inside brazen at call time (§4.1 — no `auth_env` /
/// `endpoint_env` here). `tools` selects which tools the role's agent
/// may call (§3.3); omitted or empty means none.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct RoleAssignment {
    pub provider: String,
    pub model: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tools: Vec<String>,
}

const LEGACY_KEYS: &[&str] = &["providers", "models"];

impl PerRepoProviders {
    /// Read and parse the per-repo `providers.yaml` at `path`. Cross-file
    /// references are validated separately via
    /// [`crate::config::cross::check_roles_against_providers`].
    ///
    /// Hard-errors if the file carries legacy `providers:` or `models:`
    /// blocks — those belong to the global `<harness-root>/models.yaml`
    /// only (ARCH §4.1).
    pub fn load(path: &Path) -> Result<Self, LoadError> {
        let raw = fs::read_to_string(path).map_err(|source| LoadError::Io {
            path: path.to_path_buf(),
            source,
        })?;
        Self::parse(&raw, path)
    }

    /// Parse `providers.yaml` content already in hand — the
    /// governing-config read path (ARCH §2.2: control is read from the
    /// config commit's tree, never from a worktree file). `origin`
    /// labels errors (e.g. `<config-commit>:providers.yaml`).
    pub fn parse(raw: &str, path: &Path) -> Result<Self, LoadError> {
        let doc: serde_yaml_ng::Value =
            serde_yaml_ng::from_str(raw).map_err(|source| LoadError::Yaml {
                path: path.to_path_buf(),
                source,
            })?;

        if let Some(map) = doc.as_mapping() {
            for legacy in LEGACY_KEYS {
                if map.contains_key(*legacy) {
                    return Err(LoadError::Invalid {
                        path: path.to_path_buf(),
                        key: (*legacy).to_string(),
                        message: format!(
                            "{legacy:?} block belongs in the global \
                             <harness-root>/models.yaml; the per-repo file must \
                             only carry the 'roles:' section (ARCH §4.1)",
                        ),
                    });
                }
            }
        }

        let roles_value = doc
            .as_mapping()
            .and_then(|m| m.get("roles"))
            .cloned()
            .unwrap_or(serde_yaml_ng::Value::Null);
        let roles: BTreeMap<String, RoleAssignment> = if roles_value.is_null() {
            BTreeMap::new()
        } else {
            serde_yaml_ng::from_value(roles_value).map_err(|source| LoadError::Yaml {
                path: path.to_path_buf(),
                source,
            })?
        };

        Ok(Self { roles })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    fn write_yaml(s: &str) -> NamedTempFile {
        let mut f = NamedTempFile::new().unwrap();
        f.write_all(s.as_bytes()).unwrap();
        f
    }

    const ROLES_ONLY: &str = r#"
roles:
  worker:
    provider: anthropic
    model: claude-sonnet-5
    tools: [bash, read_file]
  compactor:
    provider: anthropic
    model: claude-haiku-4-5
"#;

    #[test]
    fn parses_roles_only() {
        let f = write_yaml(ROLES_ONLY);
        let p = PerRepoProviders::load(f.path()).unwrap();
        assert_eq!(p.roles.len(), 2);
        assert_eq!(p.roles["worker"].provider, "anthropic");
        assert_eq!(p.roles["worker"].model, "claude-sonnet-5");
        // The role's `tools:` list (ARCH §4.3) parses; an omitted list
        // defaults empty (the compactor's toolset is built-in, §2.7).
        assert_eq!(p.roles["worker"].tools, vec!["bash", "read_file"]);
        assert!(p.roles["compactor"].tools.is_empty());
    }

    #[test]
    fn missing_roles_section_loads_empty() {
        // A yaml with neither 'roles:' nor any legacy block should parse
        // as an empty map. It is structurally valid and cross-validation
        // is what catches the (likely) real bug — no roles wired.
        let f = write_yaml("# nothing yet\n");
        let p = PerRepoProviders::load(f.path()).unwrap();
        assert!(p.roles.is_empty());
    }

    #[test]
    fn rejects_legacy_providers_block() {
        let yaml = r#"
providers:
  anthropic:
    endpoint: https://api.anthropic.com
    auth: { type: api_key, env: ANTHROPIC_API_KEY }
roles:
  worker:
    provider: anthropic
    model: claude-sonnet-5
"#;
        let f = write_yaml(yaml);
        let err = PerRepoProviders::load(f.path()).unwrap_err();
        match err {
            LoadError::Invalid { key, message, .. } => {
                assert_eq!(key, "providers");
                assert!(message.contains("harness-root"));
            }
            other => panic!("expected Invalid, got {other:?}"),
        }
    }

    #[test]
    fn rejects_legacy_models_block() {
        let yaml = r#"
models:
  claude-sonnet-5:
    provider: anthropic
    model_id: claude-sonnet-5
    capabilities: []
    context_window: 1000
"#;
        let f = write_yaml(yaml);
        let err = PerRepoProviders::load(f.path()).unwrap_err();
        match err {
            LoadError::Invalid { key, .. } => assert_eq!(key, "models"),
            other => panic!("expected Invalid, got {other:?}"),
        }
    }

    #[test]
    fn rejects_first_legacy_block_seen() {
        // The legacy 'providers' key is checked before 'models', so a
        // file carrying both fails on 'providers' rather than reporting
        // both — one error is enough to send the user back to fix it.
        let yaml = r#"
providers:
  anthropic:
    endpoint: x
    auth: { type: api_key, env: K }
models:
  m: { provider: anthropic, model_id: m, capabilities: [], context_window: 1 }
roles: {}
"#;
        let f = write_yaml(yaml);
        let err = PerRepoProviders::load(f.path()).unwrap_err();
        match err {
            LoadError::Invalid { key, .. } => assert_eq!(key, "providers"),
            other => panic!("expected Invalid, got {other:?}"),
        }
    }

    #[test]
    fn surfaces_yaml_parse_errors() {
        let f = write_yaml("not: [valid: yaml");
        let err = PerRepoProviders::load(f.path()).unwrap_err();
        assert!(matches!(err, LoadError::Yaml { .. }));
    }

    #[test]
    fn surfaces_io_errors() {
        let err = PerRepoProviders::load(Path::new("/no/such/per_repo.yaml")).unwrap_err();
        assert!(matches!(err, LoadError::Io { .. }));
    }

    #[test]
    fn malformed_role_entry_surfaces_yaml_error() {
        // A role missing the required 'model' field should fail
        // structurally — the per-repo loader does not silently fill in
        // defaults for required fields.
        let yaml = r#"
roles:
  worker:
    provider: anthropic
"#;
        let f = write_yaml(yaml);
        let err = PerRepoProviders::load(f.path()).unwrap_err();
        assert!(matches!(err, LoadError::Yaml { .. }));
    }

    #[test]
    fn top_level_non_mapping_loads_empty_roles() {
        // A scalar at the top level cannot have legacy keys; the loader
        // skips the legacy-block check gracefully and reports an empty
        // roles map (since 'roles' is a missing field on a non-map).
        let f = write_yaml("\"a string\"\n");
        let p = PerRepoProviders::load(f.path()).unwrap();
        assert!(p.roles.is_empty());
    }
}
