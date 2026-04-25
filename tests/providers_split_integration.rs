//! End-to-end integration: load both halves of the provider config —
//! a scratch `<harness-root>/providers.yaml` and a scratch per-repo
//! `providers.yaml` with a `roles:` section — and confirm the
//! cross-validation lands.
//!
//! Phase 1 of the v0.3 layout migration (bl-d7b1, child of bl-7c23).
//! The dispatch path is not yet wired through this loader; this test
//! exists so the loader chain is exercised and regressions land
//! visibly even before Phase 2 moves the per-repo file out of `.agent/`.

use lernie::config::{LoadError, ProvidersConfig};
use lernie::harness_root;
use std::path::PathBuf;
use tempfile::TempDir;

const GLOBAL_PROVIDERS: &str = r#"
providers:
  anthropic:
    endpoint: https://api.anthropic.com
    auth:
      type: api_key
      env: ANTHROPIC_API_KEY
models:
  claude-sonnet-4-7:
    provider: anthropic
    model_id: claude-sonnet-4-7
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
    model: claude-sonnet-4-7
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
    let global_path = harness_root::providers_path(root.path());
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
    let s = scratch(GLOBAL_PROVIDERS, PER_REPO_ROLES);
    let (cfg, warnings) = ProvidersConfig::load(&s.global_path, &s.per_repo_path).unwrap();
    assert!(warnings.is_empty(), "unexpected warnings: {warnings:?}");
    assert_eq!(cfg.global.providers.len(), 1);
    assert_eq!(cfg.global.models.len(), 2);
    assert_eq!(cfg.per_repo.roles.len(), 2);
    assert_eq!(cfg.per_repo.roles["worker"].model, "claude-sonnet-4-7");
}

#[test]
fn legacy_blocks_in_per_repo_surface_as_warnings_not_errors() {
    // The v0.2 template still ships a per-repo file with providers/
    // models blocks; Phase 1 must keep loading such a file end-to-end
    // and surface a warning, since Phase 2 is what removes the legacy
    // blocks from the template.
    let per_repo_with_legacy = format!("{GLOBAL_PROVIDERS}\n{PER_REPO_ROLES}",);
    let s = scratch(GLOBAL_PROVIDERS, &per_repo_with_legacy);
    let (cfg, warnings) = ProvidersConfig::load(&s.global_path, &s.per_repo_path).unwrap();
    assert_eq!(cfg.per_repo.roles.len(), 2);
    assert_eq!(warnings.len(), 2, "expected providers + models warnings");
    let keys: Vec<&str> = warnings.iter().map(|w| w.key.as_str()).collect();
    assert!(keys.contains(&"providers"));
    assert!(keys.contains(&"models"));
}

#[test]
fn cross_validation_failure_surfaces_at_load() {
    let bad_per_repo = r#"
roles:
  worker:
    provider: anthropic
    model: claude-sonnet-9000
"#;
    let s = scratch(GLOBAL_PROVIDERS, bad_per_repo);
    let err = ProvidersConfig::load(&s.global_path, &s.per_repo_path).unwrap_err();
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
    let s = scratch(GLOBAL_PROVIDERS, PER_REPO_ROLES);
    std::fs::remove_file(&s.global_path).unwrap();
    let err = ProvidersConfig::load(&s.global_path, &s.per_repo_path).unwrap_err();
    assert!(matches!(err, LoadError::Io { .. }));
}
