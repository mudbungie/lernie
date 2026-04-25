//! Per-repo `providers.yaml` — role → (provider, model) assignments
//! frozen at conversation creation (ARCH §4.3).
//!
//! The conversation-repo file carries only the `roles:` section: which
//! provider name and which model id each role dispatches to. Endpoint,
//! auth, and model capabilities live in the global
//! `<harness-root>/providers.yaml` and rotate independently (ARCH §4.1
//! "Two-file config split").
//!
//! Phase 1 transitional behavior: if a per-repo file still carries
//! legacy `providers:` or `models:` blocks (the v0.2 shape), the loader
//! warns rather than rejecting — the v0.2 template is removed in Phase
//! 2 and the warning becomes an error in Phase 4.

use crate::config::error::{LoadError, Warning};
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

/// One role's assignment: which provider (by name in the global
/// `providers.yaml`) and which model (by id in that provider's model
/// list). Both are validated cross-file in
/// [`crate::config::cross::check_roles_against_providers`].
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct RoleAssignment {
    pub provider: String,
    pub model: String,
}

const LEGACY_KEYS: &[&str] = &["providers", "models"];

impl PerRepoProviders {
    /// Read and parse the per-repo `providers.yaml` at `path`. Cross-file
    /// references are validated separately via
    /// [`crate::config::cross::check_roles_against_providers`].
    ///
    /// Returns warnings for legacy `providers:` / `models:` blocks left
    /// over from the v0.2 layout — they parse but are ignored at this
    /// level since the global file owns those sections (ARCH §4.1).
    pub fn load(path: &Path) -> Result<(Self, Vec<Warning>), LoadError> {
        let raw = fs::read_to_string(path).map_err(|source| LoadError::Io {
            path: path.to_path_buf(),
            source,
        })?;
        let doc: serde_yaml_ng::Value =
            serde_yaml_ng::from_str(&raw).map_err(|source| LoadError::Yaml {
                path: path.to_path_buf(),
                source,
            })?;

        let mut warnings = Vec::new();
        if let Some(map) = doc.as_mapping() {
            for legacy in LEGACY_KEYS {
                if map.contains_key(*legacy) {
                    warnings.push(Warning::new(
                        path,
                        (*legacy).to_string(),
                        format!(
                            "{legacy:?} block belongs in the global \
                             <harness-root>/providers.yaml; the per-repo file should \
                             only carry the 'roles:' section (ARCH §4.1)",
                        ),
                    ));
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

        Ok((Self { roles }, warnings))
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
    model: claude-sonnet-4-7
  compactor:
    provider: anthropic
    model: claude-haiku-4-5
"#;

    #[test]
    fn parses_roles_only() {
        let f = write_yaml(ROLES_ONLY);
        let (p, warnings) = PerRepoProviders::load(f.path()).unwrap();
        assert!(warnings.is_empty());
        assert_eq!(p.roles.len(), 2);
        assert_eq!(p.roles["worker"].provider, "anthropic");
        assert_eq!(p.roles["worker"].model, "claude-sonnet-4-7");
    }

    #[test]
    fn missing_roles_section_loads_empty() {
        // A yaml with neither 'roles:' nor any legacy block should parse
        // as an empty map. It is structurally valid and cross-validation
        // is what catches the (likely) real bug — no roles wired.
        let f = write_yaml("# nothing yet\n");
        let (p, warnings) = PerRepoProviders::load(f.path()).unwrap();
        assert!(p.roles.is_empty());
        assert!(warnings.is_empty());
    }

    #[test]
    fn warns_on_legacy_providers_block() {
        let yaml = r#"
providers:
  anthropic:
    endpoint: https://api.anthropic.com
    auth: { type: api_key, env: ANTHROPIC_API_KEY }
roles:
  worker:
    provider: anthropic
    model: claude-sonnet-4-7
"#;
        let f = write_yaml(yaml);
        let (p, warnings) = PerRepoProviders::load(f.path()).unwrap();
        assert_eq!(p.roles.len(), 1);
        assert_eq!(warnings.len(), 1);
        assert_eq!(warnings[0].key, "providers");
        assert!(warnings[0].message.contains("harness-root"));
    }

    #[test]
    fn warns_on_legacy_models_block() {
        let yaml = r#"
models:
  claude-sonnet-4-7:
    provider: anthropic
    model_id: claude-sonnet-4-7
    capabilities: []
    context_window: 1000
"#;
        let f = write_yaml(yaml);
        let (p, warnings) = PerRepoProviders::load(f.path()).unwrap();
        assert!(p.roles.is_empty());
        assert_eq!(warnings.len(), 1);
        assert_eq!(warnings[0].key, "models");
    }

    #[test]
    fn warns_on_both_legacy_blocks() {
        // Mirrors the v0.2 template shape verbatim; the loader is
        // expected to surface both warnings in a stable order.
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
        let (_, warnings) = PerRepoProviders::load(f.path()).unwrap();
        assert_eq!(warnings.len(), 2);
        assert_eq!(warnings[0].key, "providers");
        assert_eq!(warnings[1].key, "models");
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
    fn top_level_non_mapping_is_ignored_for_legacy_warnings() {
        // A scalar at the top level cannot have legacy keys; the loader
        // skips warning collection gracefully and reports an empty
        // roles map (since 'roles' is a missing field on a non-map).
        let f = write_yaml("\"a string\"\n");
        let (p, warnings) = PerRepoProviders::load(f.path()).unwrap();
        assert!(p.roles.is_empty());
        assert!(warnings.is_empty());
    }
}
