//! `.agent/manifest.yaml` — context assembly rules per ARCH §5.1.

use crate::config::error::LoadError;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

/// Top-level `manifest.yaml` shape.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct Manifest {
    pub context: ContextRules,
}

/// Rules that determine what files appear in the assembled context, in what
/// order, with what budget, and what to do on overflow.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct ContextRules {
    /// Always included regardless of budget.
    #[serde(default)]
    pub pinned: Vec<String>,
    /// Globs included in declared order, subject to budget.
    #[serde(default)]
    pub include: Vec<String>,
    pub budget_tokens: u32,
    pub overflow: OverflowPolicy,
}

/// Closed set of overflow strategies. Adding one is intentionally a code
/// change.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum OverflowPolicy {
    DropOldestExchanges,
    TruncateOldest,
    Summarize,
    Drop,
}

impl Manifest {
    /// Read, parse, and shape-check `manifest.yaml` at `path`. Path
    /// existence is intentionally not checked: pinned paths such as
    /// `.agent/goal.md` are written at dispatch time (ARCH §2.8).
    pub fn load(path: &Path) -> Result<Self, LoadError> {
        let raw = fs::read_to_string(path).map_err(|source| LoadError::Io {
            path: path.to_path_buf(),
            source,
        })?;
        let parsed: Self = serde_yaml_ng::from_str(&raw).map_err(|source| LoadError::Yaml {
            path: path.to_path_buf(),
            source,
        })?;
        parsed.validate(path)?;
        Ok(parsed)
    }

    fn validate(&self, path: &Path) -> Result<(), LoadError> {
        for (i, p) in self.context.pinned.iter().enumerate() {
            check_path_shape(path, &format!("context.pinned[{i}]"), p)?;
        }
        for (i, g) in self.context.include.iter().enumerate() {
            check_path_shape(path, &format!("context.include[{i}]"), g)?;
        }
        if self.context.budget_tokens == 0 {
            return Err(LoadError::Invalid {
                path: path.to_path_buf(),
                key: "context.budget_tokens".into(),
                message: "must be positive".into(),
            });
        }
        Ok(())
    }
}

fn check_path_shape(file: &Path, key: &str, value: &str) -> Result<(), LoadError> {
    if value.is_empty() {
        return Err(LoadError::Invalid {
            path: file.to_path_buf(),
            key: key.into(),
            message: "must not be empty".into(),
        });
    }
    if value.starts_with('/') {
        return Err(LoadError::Invalid {
            path: file.to_path_buf(),
            key: key.into(),
            message: format!("must be relative to the conversation repo, got {value:?}"),
        });
    }
    if value.split('/').any(|seg| seg == "..") {
        return Err(LoadError::Invalid {
            path: file.to_path_buf(),
            key: key.into(),
            message: format!("may not contain '..', got {value:?}"),
        });
    }
    Ok(())
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
context:
  pinned:
    - .agent/goal.md
    - .agent/system/prompts/base.md
  include:
    - exchanges/**
    - artifacts/**
    - invocations/*/result.md
  budget_tokens: 150000
  overflow: drop_oldest_exchanges
"#;

    #[test]
    fn parses_arch_example() {
        let f = write_yaml(ARCH_EXAMPLE);
        let m = Manifest::load(f.path()).unwrap();
        assert_eq!(m.context.budget_tokens, 150_000);
        assert_eq!(m.context.overflow, OverflowPolicy::DropOldestExchanges);
        assert_eq!(m.context.pinned.len(), 2);
        assert_eq!(m.context.include.len(), 3);
    }

    #[test]
    fn accepts_each_overflow_variant() {
        for variant in [
            "drop_oldest_exchanges",
            "truncate_oldest",
            "summarize",
            "drop",
        ] {
            let yaml = format!(
                "context:\n  pinned: []\n  include: []\n  budget_tokens: 1\n  overflow: {variant}\n"
            );
            let f = write_yaml(&yaml);
            assert!(Manifest::load(f.path()).is_ok(), "variant {variant} failed");
        }
    }

    #[test]
    fn rejects_absolute_pinned_path() {
        let f = write_yaml(
            "context:\n  pinned: [/etc/secret]\n  include: []\n  budget_tokens: 1\n  overflow: drop\n",
        );
        let err = Manifest::load(f.path()).unwrap_err();
        match err {
            LoadError::Invalid { key, .. } => assert_eq!(key, "context.pinned[0]"),
            other => panic!("expected Invalid, got {other:?}"),
        }
    }

    #[test]
    fn rejects_parent_dir_in_include() {
        let f = write_yaml(
            "context:\n  pinned: []\n  include: [\"../escape/**\"]\n  budget_tokens: 1\n  overflow: drop\n",
        );
        let err = Manifest::load(f.path()).unwrap_err();
        match err {
            LoadError::Invalid { key, message, .. } => {
                assert_eq!(key, "context.include[0]");
                assert!(message.contains(".."));
            }
            other => panic!("expected Invalid, got {other:?}"),
        }
    }

    #[test]
    fn rejects_empty_path() {
        let f = write_yaml(
            "context:\n  pinned: [\"\"]\n  include: []\n  budget_tokens: 1\n  overflow: drop\n",
        );
        let err = Manifest::load(f.path()).unwrap_err();
        assert!(matches!(err, LoadError::Invalid { .. }));
    }

    #[test]
    fn rejects_zero_budget() {
        let f = write_yaml(
            "context:\n  pinned: []\n  include: []\n  budget_tokens: 0\n  overflow: drop\n",
        );
        let err = Manifest::load(f.path()).unwrap_err();
        match err {
            LoadError::Invalid { key, .. } => assert_eq!(key, "context.budget_tokens"),
            other => panic!("expected Invalid, got {other:?}"),
        }
    }

    #[test]
    fn surfaces_io_and_yaml_errors() {
        assert!(matches!(
            Manifest::load(Path::new("/no/such/manifest.yaml")),
            Err(LoadError::Io { .. })
        ));
        let f = write_yaml("not yaml: [");
        assert!(matches!(
            Manifest::load(f.path()),
            Err(LoadError::Yaml { .. })
        ));
    }
}
