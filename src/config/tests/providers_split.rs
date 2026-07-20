//! End-to-end integration: load both halves of the model config — a
//! scratch `<harness-root>/models.yaml` and a scratch per-repo
//! `<conv-repo>/providers.yaml` with a `roles:` section — and confirm
//! the cross-validation lands.
//!
//! v0.6 folds the bespoke provider layer into brazen (ARCH §4.4): the
//! global file is `models.yaml` (capabilities / context windows / the
//! optional `adapter:` override — no endpoints or auth, which are
//! brazen's). The test exercises the loader independently of the
//! dispatch path so regressions in cross-validation land visibly.

use crate::config::{LoadError, ModelsConfig};
use crate::harness_root;
use std::path::PathBuf;
use tempfile::TempDir;

const GLOBAL_MODELS: &str = r#"
models:
  claude-sonnet-5:
    provider: anthropic
    model_id: claude-sonnet-5
    capabilities: [tool_use_native, prompt_caching, streaming]
    context_window: 200000
  claude-haiku-4-5:
    provider: anthropic
    model_id: claude-haiku-4-5
    capabilities: [tool_use_native, prompt_caching, streaming]
    context_window: 200000
"#;

const PER_REPO_ROLES: &str = r#"
roles:
  worker:
    provider: anthropic
    model: claude-sonnet-5
    tools: [bash, read_file]
  compactor:
    provider: anthropic
    model: claude-haiku-4-5
"#;

struct Scratch {
    _root: TempDir,
    _repo: TempDir,
    global_path: PathBuf,
    per_repo_path: PathBuf,
}

fn scratch(global: &str, per_repo: &str) -> Scratch {
    let root = TempDir::new().unwrap();
    let global_path = harness_root::models_path(root.path());
    std::fs::write(&global_path, global).unwrap();
    let repo = TempDir::new().unwrap();
    let per_repo_path = repo.path().join("providers.yaml");
    std::fs::write(&per_repo_path, per_repo).unwrap();
    Scratch {
        _root: root,
        _repo: repo,
        global_path,
        per_repo_path,
    }
}

#[test]
fn loads_both_halves_and_cross_validates() {
    let s = scratch(GLOBAL_MODELS, PER_REPO_ROLES);
    let (cfg, warnings) = ModelsConfig::load(&s.global_path, &s.per_repo_path).unwrap();
    assert!(warnings.is_empty(), "unexpected warnings: {warnings:?}");
    assert!(cfg.global.adapter.is_none());
    assert_eq!(cfg.global.models.len(), 2);
    assert_eq!(cfg.per_repo.roles.len(), 2);
    assert_eq!(cfg.per_repo.roles["worker"].model, "claude-sonnet-5");
    assert_eq!(
        cfg.per_repo.roles["worker"].tools,
        vec!["bash", "read_file"]
    );
}

#[test]
fn legacy_blocks_in_per_repo_are_a_load_error() {
    // A v0.2-shaped per-repo file carrying providers:/models: blocks
    // hard-errors at load — those belong to the global file only.
    let per_repo_with_legacy = format!("{GLOBAL_MODELS}\n{PER_REPO_ROLES}");
    let s = scratch(GLOBAL_MODELS, &per_repo_with_legacy);
    let err = ModelsConfig::load(&s.global_path, &s.per_repo_path).unwrap_err();
    match err {
        LoadError::Invalid { key, .. } => assert_eq!(key, "models"),
        other => panic!("expected Invalid, got {other:?}"),
    }
}

#[test]
fn cross_validation_failure_surfaces_at_load() {
    let bad_per_repo = r#"
roles:
  worker:
    provider: anthropic
    model: claude-sonnet-9000
"#;
    let s = scratch(GLOBAL_MODELS, bad_per_repo);
    let err = ModelsConfig::load(&s.global_path, &s.per_repo_path).unwrap_err();
    match err {
        LoadError::UnresolvedRef { key, message } => {
            assert_eq!(key, "roles.worker.model");
            assert!(message.contains("claude-sonnet-9000"));
        }
        other => panic!("expected UnresolvedRef, got {other:?}"),
    }
}

#[test]
fn missing_global_is_a_load_error() {
    let s = scratch(GLOBAL_MODELS, PER_REPO_ROLES);
    std::fs::remove_file(&s.global_path).unwrap();
    let err = ModelsConfig::load(&s.global_path, &s.per_repo_path).unwrap_err();
    assert!(matches!(err, LoadError::Io { .. }));
}
