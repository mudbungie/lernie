//! Pre-branch and adapter error paths for [`crate::prompt::run`].
//!
//! Covers failures the harness surfaces before the branch is spawned
//! (config, role, system prompt, describe) and after `complete`
//! returns (adapter-reported errors and response-parsing failures).
//! Disk/git failures during branch work live in
//! [`super::errors_disk`].

use super::fixtures::*;
use crate::prompt::Error;
use serde_json::Value;
use std::io;
use std::path::PathBuf;
use tempfile::TempDir;

#[test]
fn run_surfaces_providers_yaml_load_error() {
    let tmp = TempDir::new().unwrap();
    std::fs::create_dir_all(tmp.path().join(".agent")).unwrap();
    let err = run_with_stubs(tmp.path(), "hi", &unreachable_adapter(), &StubGit::ok()).unwrap_err();
    assert!(matches!(err, Error::Config(_)), "got {err:?}");
}

#[test]
fn run_surfaces_agents_yaml_load_error() {
    let tmp = TempDir::new().unwrap();
    let agent = tmp.path().join(".agent");
    std::fs::create_dir_all(&agent).unwrap();
    std::fs::write(agent.join("providers.yaml"), VALID_PROVIDERS_YAML).unwrap();
    let err = run_with_stubs(tmp.path(), "hi", &unreachable_adapter(), &StubGit::ok()).unwrap_err();
    assert!(matches!(err, Error::Config(_)));
}

#[test]
fn run_surfaces_cross_check_failure() {
    let bad_agents = r#"
agents:
  worker:
    model: nonexistent-model
    system_prompt: prompts/worker.md
"#;
    let repo = scaffold_repo(VALID_PROVIDERS_YAML, bad_agents, Some("body"));
    let err =
        run_with_stubs(repo.path(), "hi", &unreachable_adapter(), &StubGit::ok()).unwrap_err();
    assert!(matches!(err, Error::Config(_)));
}

#[test]
fn run_rejects_when_worker_role_missing() {
    let no_worker = r#"
agents:
  compactor:
    model: claude-sonnet-4-7
    system_prompt: prompts/compactor.md
"#;
    let repo = scaffold_repo(VALID_PROVIDERS_YAML, no_worker, Some("body"));
    let err =
        run_with_stubs(repo.path(), "hi", &unreachable_adapter(), &StubGit::ok()).unwrap_err();
    match err {
        Error::RoleMissing(role) => assert_eq!(role, "worker"),
        other => panic!("expected RoleMissing, got {other:?}"),
    }
}

#[test]
fn run_surfaces_missing_system_prompt() {
    let repo = scaffold_repo(VALID_PROVIDERS_YAML, VALID_AGENTS_YAML, None);
    let err =
        run_with_stubs(repo.path(), "hi", &unreachable_adapter(), &StubGit::ok()).unwrap_err();
    assert!(matches!(err, Error::SystemPromptRead { .. }));
}

#[test]
fn run_surfaces_describe_spawn_failure() {
    // First adapter call is `describe`; failure here surfaces as
    // AdapterSpawn and no branch is created (describe precedes
    // worktree add, so an adapter fault leaves no stray ref).
    let repo = scaffold_repo(VALID_PROVIDERS_YAML, VALID_AGENTS_YAML, Some("body"));
    let adapter = StubAdapter::failing(io::ErrorKind::NotFound, "no such binary");
    let git = StubGit::ok();
    let err = run_with_stubs(repo.path(), "hi", &adapter, &git).unwrap_err();
    assert!(matches!(err, Error::AdapterSpawn(_)));
    assert!(
        git.runs.borrow().is_empty(),
        "git must not run if describe failed"
    );
}

#[test]
fn run_surfaces_malformed_describe_json() {
    let repo = scaffold_repo(VALID_PROVIDERS_YAML, VALID_AGENTS_YAML, Some("body"));
    let adapter = StubAdapter::scripted([StubAdapter::reply_ok(b"{ not json")]);
    let err = run_with_stubs(repo.path(), "hi", &adapter, &StubGit::ok()).unwrap_err();
    assert!(matches!(err, Error::AdapterJson(_)), "got {err:?}");
}

#[test]
fn run_surfaces_describe_endpoint_env_wrong_type() {
    // `endpoint_env` must be an array of strings. Anything else
    // surfaces as a parse error — accepting it would silently drop
    // the adapter's declared config.
    let repo = scaffold_repo(VALID_PROVIDERS_YAML, VALID_AGENTS_YAML, Some("body"));
    let bad = br#"{"name":"x","schema_version":1,"capabilities":[],"models":[],
                   "auth_env":[],"endpoint_env":"NOT_AN_ARRAY"}"#;
    let adapter = StubAdapter::scripted([StubAdapter::reply_ok(bad)]);
    let err = run_with_stubs(repo.path(), "hi", &adapter, &StubGit::ok()).unwrap_err();
    assert!(matches!(err, Error::AdapterJson(_)), "got {err:?}");
}

#[test]
fn run_surfaces_complete_spawn_failure() {
    // describe succeeds, snapshot commit succeeds, complete fails at
    // spawn time.
    let repo = scaffold_repo(VALID_PROVIDERS_YAML, VALID_AGENTS_YAML, Some("body"));
    let adapter = StubAdapter::scripted([
        StubAdapter::reply_ok(STUB_DESCRIBE_JSON.as_bytes()),
        StubAdapter::reply_err(io::ErrorKind::BrokenPipe, "complete crashed"),
    ]);
    let err = run_with_stubs(repo.path(), "hi", &adapter, &StubGit::ok()).unwrap_err();
    assert!(matches!(err, Error::AdapterSpawn(_)));
}

#[test]
fn run_surfaces_adapter_returning_in_band_error() {
    let repo = scaffold_repo(VALID_PROVIDERS_YAML, VALID_AGENTS_YAML, Some("body"));
    let error_json = br#"{"type":"error","kind":"fatal","http_status":401,"message":"boom"}"#;
    let adapter = StubAdapter::happy(error_json);
    let err = run_with_stubs(repo.path(), "hi", &adapter, &StubGit::ok()).unwrap_err();
    match err {
        Error::AdapterError {
            kind,
            message,
            http_status,
        } => {
            assert_eq!(kind, "fatal");
            assert_eq!(message, "boom");
            assert_eq!(http_status, Some(401));
        }
        other => panic!("expected AdapterError, got {other:?}"),
    }
}

#[test]
fn run_surfaces_malformed_complete_json() {
    let repo = scaffold_repo(VALID_PROVIDERS_YAML, VALID_AGENTS_YAML, Some("body"));
    let adapter = StubAdapter::happy(b"{ not json");
    let err = run_with_stubs(repo.path(), "hi", &adapter, &StubGit::ok()).unwrap_err();
    assert!(matches!(err, Error::AdapterJson(_)));
}

#[test]
fn run_surfaces_response_shape_mismatch() {
    let repo = scaffold_repo(VALID_PROVIDERS_YAML, VALID_AGENTS_YAML, Some("body"));
    let adapter = StubAdapter::happy(br#"{"unexpected":"shape"}"#);
    let err = run_with_stubs(repo.path(), "hi", &adapter, &StubGit::ok()).unwrap_err();
    assert!(matches!(err, Error::AdapterJson(_)));
}

#[test]
fn run_surfaces_in_band_error_with_default_fields() {
    let repo = scaffold_repo(VALID_PROVIDERS_YAML, VALID_AGENTS_YAML, Some("body"));
    let adapter = StubAdapter::happy(br#"{"type":"error"}"#);
    let err = run_with_stubs(repo.path(), "hi", &adapter, &StubGit::ok()).unwrap_err();
    match err {
        Error::AdapterError {
            kind,
            message,
            http_status,
        } => {
            assert_eq!(kind, "unknown");
            assert_eq!(message, "");
            assert_eq!(http_status, None);
        }
        other => panic!("expected AdapterError, got {other:?}"),
    }
}

#[test]
fn error_display_includes_context() {
    // Exercises every `#[error("...")]` format line.
    let _: String = Error::RoleMissing("worker".into()).to_string();
    let _: String = Error::AdapterError {
        kind: "fatal".into(),
        message: "m".into(),
        http_status: Some(401),
    }
    .to_string();
    let _: String = Error::SystemPromptRead {
        path: PathBuf::from("/x"),
        source: io::Error::other("y"),
    }
    .to_string();
    let _: String = Error::AdapterSpawn(io::Error::other("x")).to_string();
    let _: String = Error::AdapterJson(serde_json::from_str::<Value>("{").unwrap_err()).to_string();
    let _: String = Error::Git {
        op: "add",
        source: io::Error::other("x"),
    }
    .to_string();
    let _: String = Error::DispatchFailed {
        role: "compactor",
        source: io::Error::other("x"),
    }
    .to_string();
    let _: String = Error::Io(io::Error::other("x")).to_string();
    let load_err: crate::config::LoadError = crate::config::LoadError::UnresolvedRef {
        key: "k".into(),
        message: "m".into(),
    };
    let e: Error = load_err.into();
    let _: String = e.to_string();
}
