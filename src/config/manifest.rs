//! `<conv-repo>/manifest.yaml` — context assembly rules per ARCH §5.2.
//!
//! Role-keyed since v0.3: each role declares its own pinned + ordered
//! includes, budget, and overflow policy. Paths are relative to the
//! branch's worktree (§5.1).

use crate::config::error::LoadError;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

/// Top-level `manifest.yaml` shape.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct Manifest {
    #[serde(default)]
    pub roles: BTreeMap<String, RoleRules>,
}

/// One role's context-assembly rules.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct RoleRules {
    /// Always included regardless of budget.
    #[serde(default)]
    pub pinned: Vec<String>,
    /// Globs included in declared order, subject to budget.
    #[serde(default)]
    pub order: Vec<String>,
    pub budget_tokens: u32,
    pub overflow: OverflowPolicy,
}

/// Closed set of overflow strategies. Adding one is intentionally a code
/// change.
///
/// `DropOldestSteps` is retained for backward compatibility with
/// pre-v0.3.1 manifests; new manifests should prefer
/// `DropOldestSummaries` since step records no longer appear in
/// context-assembly order (ARCH §2.3 — they live outside every
/// worktree).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum OverflowPolicy {
    DropOldestSteps,
    DropOldestSummaries,
    Truncate,
    Summarize,
    Drop,
}

impl Manifest {
    /// Read, parse, and shape-check `manifest.yaml` at `path`. Path
    /// existence of pinned/ordered entries is intentionally not checked:
    /// `goal.md`, `soul.md`, and `summary/**` are written at dispatch
    /// time (ARCH §2.3, §2.7).
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
        for (role, rules) in &self.roles {
            for (i, p) in rules.pinned.iter().enumerate() {
                check_path_shape(path, &format!("roles.{role}.pinned[{i}]"), p)?;
            }
            for (i, g) in rules.order.iter().enumerate() {
                check_path_shape(path, &format!("roles.{role}.order[{i}]"), g)?;
            }
            if rules.budget_tokens == 0 {
                return Err(LoadError::Invalid {
                    path: path.to_path_buf(),
                    key: format!("roles.{role}.budget_tokens"),
                    message: "must be positive".into(),
                });
            }
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
            message: format!("must be relative to the branch worktree, got {value:?}"),
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
roles:
  worker:
    pinned:
      - goal.md
      - soul.md
      - descriptions/**
    order:
      - summary/**
      - skills/**
    budget_tokens: 150000
    overflow: drop_oldest_summaries
  compactor:
    pinned:
      - goal.md
      - soul.md
    order: []
    budget_tokens: 50000
    overflow: truncate
"#;

    #[test]
    fn parses_arch_example() {
        let f = write_yaml(ARCH_EXAMPLE);
        let m = Manifest::load(f.path()).unwrap();
        assert_eq!(m.roles.len(), 2);
        let worker = &m.roles["worker"];
        assert_eq!(worker.budget_tokens, 150_000);
        assert_eq!(worker.overflow, OverflowPolicy::DropOldestSummaries);
        assert_eq!(worker.pinned.len(), 3);
        // ARCH §5.2 amended (v0.3.1): step records are not context.
        // `worker.order` lists only manifest-eligible paths (summary,
        // skills) — `steps/**` MUST NOT appear, which the location
        // physically enforces (steps/ is at the conv-repo root,
        // outside every worktree, §2.2 / §2.3).
        assert_eq!(worker.order, vec!["summary/**", "skills/**"]);
        let compactor = &m.roles["compactor"];
        assert_eq!(compactor.overflow, OverflowPolicy::Truncate);
        assert!(compactor.order.is_empty());
    }

    #[test]
    fn accepts_each_overflow_variant() {
        for variant in [
            "drop_oldest_steps",
            "drop_oldest_summaries",
            "truncate",
            "summarize",
            "drop",
        ] {
            let yaml = format!(
                "roles:\n  r:\n    pinned: []\n    order: []\n    budget_tokens: 1\n    overflow: {variant}\n"
            );
            let f = write_yaml(&yaml);
            assert!(Manifest::load(f.path()).is_ok(), "variant {variant} failed");
        }
    }

    #[test]
    fn empty_roles_section_is_ok() {
        // An empty manifest is structurally valid; cross-checks elsewhere
        // catch the (likely) real bug — no roles wired.
        let f = write_yaml("roles: {}\n");
        let m = Manifest::load(f.path()).unwrap();
        assert!(m.roles.is_empty());
    }

    #[test]
    fn missing_roles_section_loads_empty() {
        let f = write_yaml("# nothing yet\n");
        let m = Manifest::load(f.path()).unwrap();
        assert!(m.roles.is_empty());
    }

    #[test]
    fn rejects_absolute_pinned_path() {
        let f = write_yaml(
            "roles:\n  r:\n    pinned: [/etc/secret]\n    order: []\n    budget_tokens: 1\n    overflow: drop\n",
        );
        let err = Manifest::load(f.path()).unwrap_err();
        match err {
            LoadError::Invalid { key, .. } => assert_eq!(key, "roles.r.pinned[0]"),
            other => panic!("expected Invalid, got {other:?}"),
        }
    }

    #[test]
    fn rejects_parent_dir_in_order() {
        let f = write_yaml(
            "roles:\n  r:\n    pinned: []\n    order: [\"../escape/**\"]\n    budget_tokens: 1\n    overflow: drop\n",
        );
        let err = Manifest::load(f.path()).unwrap_err();
        match err {
            LoadError::Invalid { key, message, .. } => {
                assert_eq!(key, "roles.r.order[0]");
                assert!(message.contains(".."));
            }
            other => panic!("expected Invalid, got {other:?}"),
        }
    }

    #[test]
    fn rejects_empty_path() {
        let f = write_yaml(
            "roles:\n  r:\n    pinned: [\"\"]\n    order: []\n    budget_tokens: 1\n    overflow: drop\n",
        );
        let err = Manifest::load(f.path()).unwrap_err();
        assert!(matches!(err, LoadError::Invalid { .. }));
    }

    #[test]
    fn rejects_zero_budget() {
        let f = write_yaml(
            "roles:\n  r:\n    pinned: []\n    order: []\n    budget_tokens: 0\n    overflow: drop\n",
        );
        let err = Manifest::load(f.path()).unwrap_err();
        match err {
            LoadError::Invalid { key, .. } => assert_eq!(key, "roles.r.budget_tokens"),
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
