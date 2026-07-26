//! `manifest.yaml` — context assembly rules per ARCH §5.2.
//!
//! Role-keyed since v0.3: each role declares its own pinned + ordered
//! includes, budget, and overflow policy. Paths are relative to the
//! branch's worktree (§5.1). The runtime consumer is
//! `prompt::dispatch::assembler` (§5.2 context assembly), handed one
//! role's [`RoleRules`] by `prompt::resolve`, which reads this file from
//! the governing config commit's tree — never a worktree file (§2.2) —
//! hence the content-in-hand [`Manifest::parse`] seam and no path-taking
//! loader, like its control-file siblings.

use crate::config::error::LoadError;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
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
/// Every variant here is an act assembly can perform on the tree it was
/// handed (ARCH §5.2) — model-driven shedding is the compaction
/// checkpoint's, declared once in `workflow.yaml` `compaction:` (§6), and
/// is deliberately absent from this vocabulary.
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
    Drop,
}

impl Manifest {
    /// Parse and shape-check a `manifest.yaml` body already in hand
    /// (ARCH §2.2 — control is read from the config commit's tree, so
    /// there is no path-taking loader). `origin` is the content's one
    /// true address — `<commit>:manifest.yaml` — used in errors. Path
    /// existence of pinned/ordered entries is intentionally not checked:
    /// `goal.md`, `soul.md`, and `summary/**` are written at dispatch
    /// time (ARCH §2.3, §2.7).
    pub fn parse(raw: &str, origin: &Path) -> Result<Self, LoadError> {
        let parsed: Self = serde_yaml_ng::from_str(raw).map_err(|source| LoadError::Yaml {
            path: origin.to_path_buf(),
            source,
        })?;
        parsed.validate(origin)?;
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

    /// Parse a manifest body against a fixed test origin (the §2.2
    /// `<commit>:<path>` label a real control read supplies).
    fn parse(s: &str) -> Result<Manifest, LoadError> {
        Manifest::parse(s, Path::new("deadbeef:manifest.yaml"))
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
        let m = parse(ARCH_EXAMPLE).unwrap();
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
            "drop",
        ] {
            let yaml = format!(
                "roles:\n  r:\n    pinned: []\n    order: []\n    budget_tokens: 1\n    overflow: {variant}\n"
            );
            assert!(parse(&yaml).is_ok(), "variant {variant} failed");
        }
    }

    /// `summarize` was subtracted from the vocabulary (bl-a1a1): every
    /// remaining policy is an act assembly can perform on the tree it
    /// holds, and model-driven shedding is the `workflow.yaml`
    /// `compaction:` clock's alone (ARCH §5.2, §6). A manifest still
    /// naming it is declined rather than silently treated as a no-op.
    #[test]
    fn rejects_retired_summarize_overflow() {
        let err = parse(
            "roles:\n  r:\n    pinned: []\n    order: []\n    budget_tokens: 1\n    overflow: summarize\n",
        )
        .unwrap_err();
        match err {
            LoadError::Yaml { source, .. } => {
                assert!(source.to_string().contains("summarize"), "{source}")
            }
            other => panic!("expected Yaml, got {other:?}"),
        }
    }

    #[test]
    fn empty_roles_section_is_ok() {
        // An empty manifest is structurally valid; cross-checks elsewhere
        // catch the (likely) real bug — no roles wired.
        let m = parse("roles: {}\n").unwrap();
        assert!(m.roles.is_empty());
    }

    #[test]
    fn missing_roles_section_loads_empty() {
        let m = parse("# nothing yet\n").unwrap();
        assert!(m.roles.is_empty());
    }

    #[test]
    fn rejects_absolute_pinned_path() {
        let err = parse(
            "roles:\n  r:\n    pinned: [/etc/secret]\n    order: []\n    budget_tokens: 1\n    overflow: drop\n",
        )
        .unwrap_err();
        match err {
            LoadError::Invalid { key, .. } => assert_eq!(key, "roles.r.pinned[0]"),
            other => panic!("expected Invalid, got {other:?}"),
        }
    }

    #[test]
    fn rejects_parent_dir_in_order() {
        let err = parse(
            "roles:\n  r:\n    pinned: []\n    order: [\"../escape/**\"]\n    budget_tokens: 1\n    overflow: drop\n",
        )
        .unwrap_err();
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
        let err = parse(
            "roles:\n  r:\n    pinned: [\"\"]\n    order: []\n    budget_tokens: 1\n    overflow: drop\n",
        )
        .unwrap_err();
        assert!(matches!(err, LoadError::Invalid { .. }));
    }

    #[test]
    fn rejects_zero_budget() {
        let err = parse(
            "roles:\n  r:\n    pinned: []\n    order: []\n    budget_tokens: 0\n    overflow: drop\n",
        )
        .unwrap_err();
        match err {
            LoadError::Invalid { key, .. } => assert_eq!(key, "roles.r.budget_tokens"),
            other => panic!("expected Invalid, got {other:?}"),
        }
    }

    #[test]
    fn surfaces_yaml_errors_with_the_origin() {
        match parse("not yaml: [").unwrap_err() {
            LoadError::Yaml { path, .. } => {
                assert_eq!(path, Path::new("deadbeef:manifest.yaml"));
            }
            other => panic!("expected Yaml, got {other:?}"),
        }
    }
}
