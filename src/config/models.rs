//! `<harness-root>/models.yaml` — model capabilities and context
//! windows (ARCH §4.2), plus the optional `adapter:` binary override
//! (§4.2, §4.4 Extensibility).
//!
//! v0.6 folds the bespoke provider layer into brazen (§4.4): endpoints,
//! auth, and wire dialects are brazen's facts, declared in brazen's own
//! config as named provider *rows*. lernie references a row by name and
//! never reads endpoint or credential material (§4.1). So the global
//! file no longer carries a `providers:` map — it carries only the
//! facts lernie's behavior relies on and brazen does not own: per-model
//! capabilities and context windows. `provider:` on a model is a brazen
//! row name, unvalidated here (a missing row is a brazen load-time
//! failure at call time, §4.1).

use crate::config::error::{LoadError, Warning};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

/// Top-level shape of the harness-root `models.yaml` (ARCH §4.2).
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq, Default)]
pub struct Models {
    /// Optional provider-adapter binary override (§4.2, §4.4). Default
    /// (`None`) resolves `bz` on `PATH`. Any binary honoring the same
    /// pipe contract slots in verbatim; the load-time version guard is
    /// skipped under an override and the in-band `MessageStart.v`
    /// handshake governs instead (§4.4).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub adapter: Option<PathBuf>,
    #[serde(default)]
    pub models: BTreeMap<String, Model>,
}

/// One model: `(provider row, model_id, capabilities, context_window)`
/// (ARCH §4.2). `provider` is a brazen provider-row name — the entire
/// provider surface lernie sees (§4.1); endpoint and auth resolve inside
/// brazen at call time.
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

impl Models {
    /// Read, parse, and shallow-validate `models.yaml` at `path`.
    /// Cross-file references (`roles.*.{provider,model}`) are validated
    /// separately via [`crate::config::cross`].
    pub fn load(path: &Path) -> Result<(Self, Vec<Warning>), LoadError> {
        let raw = fs::read_to_string(path).map_err(|source| LoadError::Io {
            path: path.to_path_buf(),
            source,
        })?;
        let parsed: Self = serde_yaml_ng::from_str(&raw).map_err(|source| LoadError::Yaml {
            path: path.to_path_buf(),
            source,
        })?;
        let warnings = parsed.collect_warnings(path);
        Ok((parsed, warnings))
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
                            "capability {cap:?} is not in the seeded registry; \
                             add it if intended (extend-only)"
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
models:
  claude-sonnet-5:
    provider: anthropic
    model_id: claude-sonnet-5
    capabilities: [tool_use_native, prompt_caching, streaming, stop_sequences]
    context_window: 1000000
"#;

    #[test]
    fn parses_arch_example() {
        let f = write_yaml(ARCH_EXAMPLE);
        let (m, warnings) = Models::load(f.path()).unwrap();
        assert!(warnings.is_empty());
        assert!(m.adapter.is_none());
        assert_eq!(m.models.len(), 1);
        assert_eq!(m.models["claude-sonnet-5"].provider, "anthropic");
        assert_eq!(m.models["claude-sonnet-5"].context_window, 1_000_000);
    }

    #[test]
    fn parses_adapter_override() {
        let yaml = r#"
adapter: /usr/local/bin/bz
models: {}
"#;
        let f = write_yaml(yaml);
        let (m, _) = Models::load(f.path()).unwrap();
        assert_eq!(m.adapter.as_deref(), Some(Path::new("/usr/local/bin/bz")));
    }

    #[test]
    fn warns_on_unknown_capability() {
        let yaml = r#"
models:
  acme-llm:
    provider: acme
    model_id: llm-1
    capabilities: [time_travel]
    context_window: 8000
"#;
        let f = write_yaml(yaml);
        let (_, warnings) = Models::load(f.path()).unwrap();
        assert_eq!(warnings.len(), 1);
        assert!(warnings[0].message.contains("time_travel"));
    }

    #[test]
    fn surfaces_yaml_parse_errors() {
        let f = write_yaml("not: [valid: yaml");
        let err = Models::load(f.path()).unwrap_err();
        assert!(matches!(err, LoadError::Yaml { .. }));
    }

    #[test]
    fn surfaces_io_errors() {
        let err = Models::load(Path::new("/no/such/models.yaml")).unwrap_err();
        assert!(matches!(err, LoadError::Io { .. }));
    }
}
