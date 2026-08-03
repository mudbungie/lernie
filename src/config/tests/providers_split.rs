//! End-to-end integration: load both halves of the model config — a
//! scratch `<harness-root>/models.yaml` and a scratch per-repo
//! `<conv-repo>/providers.yaml` with a `roles:` section.
//!
//! The global file carries only the optional `adapter:` override
//! (bl-35e2 — no models table, no endpoints or auth, which are
//! brazen's); the role assignment is the single home of the (provider
//! row, model id) pointer, so there is no cross-file resolution to
//! check. The test exercises the loader independently of the dispatch
//! path so regressions in the split land visibly.

use crate::config::{LoadError, ModelsConfig};
use crate::harness_root;
use std::path::PathBuf;
use tempfile::TempDir;

/// The shipped-template shape: comments only, no override, no models.
const GLOBAL_MODELS: &str = "# adapter: /path/to/bz-compatible-binary\n";

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

/// The two halves as the runtime sees them: the global `models.yaml` on
/// disk at the harness root, and the per-repo `providers.yaml` as content
/// already read out of the config commit's tree (ARCH §2.2 — never a
/// worktree file), labelled with its `<commit>:<path>` origin.
struct Scratch {
    _root: TempDir,
    global_path: PathBuf,
    per_repo_raw: String,
    per_repo_origin: PathBuf,
}

impl Scratch {
    fn load(&self) -> Result<ModelsConfig, LoadError> {
        ModelsConfig::load_with_per_repo(
            &self.global_path,
            &self.per_repo_raw,
            &self.per_repo_origin,
        )
    }
}

fn scratch(global: &str, per_repo: &str) -> Scratch {
    let root = TempDir::new().unwrap();
    let global_path = harness_root::models_path(root.path());
    std::fs::write(&global_path, global).unwrap();
    Scratch {
        _root: root,
        global_path,
        per_repo_raw: per_repo.to_string(),
        per_repo_origin: PathBuf::from("<commit>:providers.yaml"),
    }
}

#[test]
fn loads_both_halves() {
    let s = scratch(GLOBAL_MODELS, PER_REPO_ROLES);
    let cfg = s.load().unwrap();
    assert!(cfg.global.adapter.is_none());
    assert_eq!(cfg.per_repo.roles.len(), 2);
    assert_eq!(cfg.per_repo.roles["worker"].model, "claude-sonnet-5");
    assert_eq!(
        cfg.per_repo.roles["worker"].tools,
        vec!["bash", "read_file"]
    );
}

#[test]
fn adapter_override_rides_the_global_half() {
    let s = scratch("adapter: /opt/bz\n", PER_REPO_ROLES);
    let cfg = s.load().unwrap();
    assert_eq!(
        cfg.global.adapter.as_deref(),
        Some(std::path::Path::new("/opt/bz"))
    );
}

#[test]
fn legacy_blocks_in_per_repo_are_a_load_error() {
    // A v0.2-shaped per-repo file carrying providers:/models: blocks
    // hard-errors at load — a conversation repo never declares either.
    let per_repo_with_legacy = format!("models: {{}}\n{PER_REPO_ROLES}");
    let s = scratch(GLOBAL_MODELS, &per_repo_with_legacy);
    let err = s.load().unwrap_err();
    match err {
        LoadError::Invalid { key, .. } => assert_eq!(key, "models"),
        other => panic!("expected Invalid, got {other:?}"),
    }
}

#[test]
fn missing_global_is_a_load_error() {
    let s = scratch(GLOBAL_MODELS, PER_REPO_ROLES);
    std::fs::remove_file(&s.global_path).unwrap();
    let err = s.load().unwrap_err();
    assert!(matches!(err, LoadError::Io { .. }));
}
