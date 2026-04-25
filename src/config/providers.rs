//! `<harness-root>/providers.yaml` — inference providers and the
//! models served from them. Schema follows ARCH §4.1 and §4.2.
//!
//! Decision: the arch examples did not commit on whether `models:` lives in
//! its own file. We keep them together in `providers.yaml` so a model is one
//! key away from the (endpoint, auth) pair it depends on.

use crate::config::error::{LoadError, Warning};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;

/// Top-level shape of the harness-root `providers.yaml` (ARCH §4.1).
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct Providers {
    pub providers: BTreeMap<String, Provider>,
    #[serde(default)]
    pub models: BTreeMap<String, Model>,
}

/// One inference provider: an (endpoint, auth) pair.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct Provider {
    pub endpoint: String,
    pub auth: Auth,
}

/// Auth strategy for a provider. The closed set tracks ARCH §4.1.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Auth {
    ApiKey { env: String },
    AwsSigv4 { profile: String },
}

/// One model declared by some provider.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct Model {
    pub provider: String,
    pub model_id: String,
    pub capabilities: Capabilities,
    pub context_window: u32,
}

/// Capability names declared on a model. Names are extend-only (§4.2): the
/// loader seeds a known registry and warns (does not fail) on names outside
/// it, so a new provider may declare new capabilities without blocking.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(transparent)]
pub struct Capabilities(pub Vec<String>);

/// Capability names seeded from the arch examples. Unknown names produce a
/// warning, never an error.
pub fn known_capabilities() -> BTreeSet<&'static str> {
    [
        "tool_use_native",
        "prompt_caching",
        "streaming",
        "stop_sequences",
    ]
    .into_iter()
    .collect()
}

impl Providers {
    /// Read, parse, and shallow-validate `providers.yaml` at `path`.
    /// Cross-file references (e.g. `agents.*.model`) are validated separately
    /// via [`crate::config::cross`].
    pub fn load(path: &Path) -> Result<(Self, Vec<Warning>), LoadError> {
        let raw = fs::read_to_string(path).map_err(|source| LoadError::Io {
            path: path.to_path_buf(),
            source,
        })?;
        let parsed: Self = serde_yaml_ng::from_str(&raw).map_err(|source| LoadError::Yaml {
            path: path.to_path_buf(),
            source,
        })?;
        parsed.validate(path)?;
        let warnings = parsed.collect_warnings(path);
        Ok((parsed, warnings))
    }

    fn validate(&self, path: &Path) -> Result<(), LoadError> {
        for (name, model) in &self.models {
            if !self.providers.contains_key(&model.provider) {
                return Err(LoadError::Invalid {
                    path: path.to_path_buf(),
                    key: format!("models.{name}.provider"),
                    message: format!("names provider {:?} which is not declared", model.provider),
                });
            }
        }
        Ok(())
    }

    fn collect_warnings(&self, path: &Path) -> Vec<Warning> {
        let known = known_capabilities();
        let mut out = Vec::new();
        for (name, model) in &self.models {
            for cap in &model.capabilities.0 {
                if !known.contains(cap.as_str()) {
                    out.push(Warning::new(
                        path,
                        format!("models.{name}.capabilities"),
                        format!(
                            "capability {:?} is not in the seeded registry; \
                             add it if intended (extend-only)",
                            cap
                        ),
                    ));
                }
            }
        }
        out
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

    const ARCH_EXAMPLE: &str = r#"
providers:
  anthropic:
    endpoint: https://api.anthropic.com
    auth:
      type: api_key
      env: ANTHROPIC_API_KEY
  bedrock:
    endpoint: https://bedrock-runtime.us-east-1.amazonaws.com
    auth:
      type: aws_sigv4
      profile: default
models:
  claude-sonnet-4-7:
    provider: anthropic
    model_id: claude-sonnet-4-7
    capabilities: [tool_use_native, prompt_caching, streaming, stop_sequences]
    context_window: 200000
"#;

    #[test]
    fn parses_arch_example() {
        let f = write_yaml(ARCH_EXAMPLE);
        let (p, warnings) = Providers::load(f.path()).unwrap();
        assert!(warnings.is_empty());
        assert_eq!(p.providers.len(), 2);
        assert!(matches!(p.providers["anthropic"].auth, Auth::ApiKey { .. }));
        assert!(matches!(p.providers["bedrock"].auth, Auth::AwsSigv4 { .. }));
        assert_eq!(p.models["claude-sonnet-4-7"].context_window, 200_000);
    }

    #[test]
    fn warns_on_unknown_capability() {
        let yaml = r#"
providers:
  acme:
    endpoint: https://example.com
    auth:
      type: api_key
      env: ACME_KEY
models:
  acme-llm:
    provider: acme
    model_id: llm-1
    capabilities: [time_travel]
    context_window: 8000
"#;
        let f = write_yaml(yaml);
        let (_, warnings) = Providers::load(f.path()).unwrap();
        assert_eq!(warnings.len(), 1);
        assert!(warnings[0].message.contains("time_travel"));
    }

    #[test]
    fn rejects_model_with_undeclared_provider() {
        let yaml = r#"
providers:
  anthropic:
    endpoint: https://api.anthropic.com
    auth: { type: api_key, env: ANTHROPIC_API_KEY }
models:
  ghost-model:
    provider: phantom
    model_id: x
    capabilities: []
    context_window: 1000
"#;
        let f = write_yaml(yaml);
        let err = Providers::load(f.path()).unwrap_err();
        match err {
            LoadError::Invalid { key, message, .. } => {
                assert_eq!(key, "models.ghost-model.provider");
                assert!(message.contains("phantom"));
            }
            other => panic!("expected Invalid, got {other:?}"),
        }
    }

    #[test]
    fn surfaces_yaml_parse_errors() {
        let f = write_yaml("not: [valid: yaml");
        let err = Providers::load(f.path()).unwrap_err();
        assert!(matches!(err, LoadError::Yaml { .. }));
    }

    #[test]
    fn surfaces_io_errors() {
        let err = Providers::load(Path::new("/no/such/providers.yaml")).unwrap_err();
        assert!(matches!(err, LoadError::Io { .. }));
    }
}
