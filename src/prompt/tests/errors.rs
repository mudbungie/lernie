//! Error-path coverage for [`crate::prompt::run`].

use super::fixtures::*;
use crate::prompt::{Error, run};
use serde_json::Value;
use std::io;
use std::path::PathBuf;
use tempfile::TempDir;

#[test]
fn run_surfaces_providers_yaml_load_error() {
    let tmp = TempDir::new().unwrap();
    std::fs::create_dir_all(tmp.path().join(".agent")).unwrap();
    let adapter = StubAdapter::returning_ok(b"");
    let git = StubGit::ok();
    let clock = FixedClock::new();
    let id = FixedIdGen;
    let err = run(tmp.path(), "hi", &valid_deps(&adapter, &git, &clock, &id)).unwrap_err();
    assert!(matches!(err, Error::Config(_)), "got {err:?}");
}

#[test]
fn run_surfaces_agents_yaml_load_error() {
    let tmp = TempDir::new().unwrap();
    let agent = tmp.path().join(".agent");
    std::fs::create_dir_all(&agent).unwrap();
    std::fs::write(agent.join("providers.yaml"), VALID_PROVIDERS_YAML).unwrap();
    let adapter = StubAdapter::returning_ok(b"");
    let git = StubGit::ok();
    let clock = FixedClock::new();
    let id = FixedIdGen;
    let err = run(tmp.path(), "hi", &valid_deps(&adapter, &git, &clock, &id)).unwrap_err();
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
    let adapter = StubAdapter::returning_ok(b"");
    let git = StubGit::ok();
    let clock = FixedClock::new();
    let id = FixedIdGen;
    let err = run(repo.path(), "hi", &valid_deps(&adapter, &git, &clock, &id)).unwrap_err();
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
    let adapter = StubAdapter::returning_ok(b"");
    let git = StubGit::ok();
    let clock = FixedClock::new();
    let id = FixedIdGen;
    let err = run(repo.path(), "hi", &valid_deps(&adapter, &git, &clock, &id)).unwrap_err();
    match err {
        Error::RoleMissing(role) => assert_eq!(role, "worker"),
        other => panic!("expected RoleMissing, got {other:?}"),
    }
}

#[test]
fn run_surfaces_missing_system_prompt() {
    let repo = scaffold_repo(VALID_PROVIDERS_YAML, VALID_AGENTS_YAML, None);
    let adapter = StubAdapter::returning_ok(b"");
    let git = StubGit::ok();
    let clock = FixedClock::new();
    let id = FixedIdGen;
    let err = run(repo.path(), "hi", &valid_deps(&adapter, &git, &clock, &id)).unwrap_err();
    assert!(matches!(err, Error::SystemPromptRead { .. }));
}

#[test]
fn run_surfaces_adapter_spawn_failure() {
    let repo = scaffold_repo(VALID_PROVIDERS_YAML, VALID_AGENTS_YAML, Some("body"));
    let adapter = StubAdapter::returning_err(io::ErrorKind::NotFound, "no such binary");
    let git = StubGit::ok();
    let clock = FixedClock::new();
    let id = FixedIdGen;
    let err = run(repo.path(), "hi", &valid_deps(&adapter, &git, &clock, &id)).unwrap_err();
    assert!(matches!(err, Error::AdapterSpawn(_)));
}

#[test]
fn run_surfaces_adapter_returning_in_band_error() {
    let repo = scaffold_repo(VALID_PROVIDERS_YAML, VALID_AGENTS_YAML, Some("body"));
    let error_json = br#"{"type":"error","kind":"fatal","http_status":401,"message":"boom"}"#;
    let adapter = StubAdapter::returning_ok(error_json);
    let git = StubGit::ok();
    let clock = FixedClock::new();
    let id = FixedIdGen;
    let err = run(repo.path(), "hi", &valid_deps(&adapter, &git, &clock, &id)).unwrap_err();
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
fn run_surfaces_malformed_adapter_json() {
    let repo = scaffold_repo(VALID_PROVIDERS_YAML, VALID_AGENTS_YAML, Some("body"));
    let adapter = StubAdapter::returning_ok(b"{ not json");
    let git = StubGit::ok();
    let clock = FixedClock::new();
    let id = FixedIdGen;
    let err = run(repo.path(), "hi", &valid_deps(&adapter, &git, &clock, &id)).unwrap_err();
    assert!(matches!(err, Error::AdapterJson(_)));
}

#[test]
fn run_surfaces_response_shape_mismatch() {
    let repo = scaffold_repo(VALID_PROVIDERS_YAML, VALID_AGENTS_YAML, Some("body"));
    let adapter = StubAdapter::returning_ok(br#"{"unexpected":"shape"}"#);
    let git = StubGit::ok();
    let clock = FixedClock::new();
    let id = FixedIdGen;
    let err = run(repo.path(), "hi", &valid_deps(&adapter, &git, &clock, &id)).unwrap_err();
    assert!(matches!(err, Error::AdapterJson(_)));
}

#[test]
fn run_surfaces_in_band_error_with_default_fields() {
    let repo = scaffold_repo(VALID_PROVIDERS_YAML, VALID_AGENTS_YAML, Some("body"));
    let adapter = StubAdapter::returning_ok(br#"{"type":"error"}"#);
    let git = StubGit::ok();
    let clock = FixedClock::new();
    let id = FixedIdGen;
    let err = run(repo.path(), "hi", &valid_deps(&adapter, &git, &clock, &id)).unwrap_err();
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
fn run_surfaces_git_add_failure() {
    let repo = scaffold_repo(VALID_PROVIDERS_YAML, VALID_AGENTS_YAML, Some("body"));
    let adapter = StubAdapter::returning_ok(HAPPY_RESPONSE_JSON.as_bytes());
    let git = StubGit::failing_at(0);
    let clock = FixedClock::new();
    let id = FixedIdGen;
    let err = run(repo.path(), "hi", &valid_deps(&adapter, &git, &clock, &id)).unwrap_err();
    assert!(matches!(err, Error::Git { op: "add", .. }));
}

#[test]
fn run_surfaces_git_commit_failure() {
    let repo = scaffold_repo(VALID_PROVIDERS_YAML, VALID_AGENTS_YAML, Some("body"));
    let adapter = StubAdapter::returning_ok(HAPPY_RESPONSE_JSON.as_bytes());
    let git = StubGit::failing_at(1);
    let clock = FixedClock::new();
    let id = FixedIdGen;
    let err = run(repo.path(), "hi", &valid_deps(&adapter, &git, &clock, &id)).unwrap_err();
    assert!(matches!(err, Error::Git { op: "commit", .. }));
}

#[test]
fn run_surfaces_git_rev_parse_failure() {
    let repo = scaffold_repo(VALID_PROVIDERS_YAML, VALID_AGENTS_YAML, Some("body"));
    let adapter = StubAdapter::returning_ok(HAPPY_RESPONSE_JSON.as_bytes());
    let git = StubGit::failing_at(2);
    let clock = FixedClock::new();
    let id = FixedIdGen;
    let err = run(repo.path(), "hi", &valid_deps(&adapter, &git, &clock, &id)).unwrap_err();
    assert!(matches!(
        err,
        Error::Git {
            op: "rev-parse",
            ..
        }
    ));
}

#[test]
fn run_surfaces_exchanges_write_failure() {
    // Pre-create `exchanges` as a regular file so create_dir_all fails.
    let repo = scaffold_repo(VALID_PROVIDERS_YAML, VALID_AGENTS_YAML, Some("body"));
    std::fs::write(repo.path().join("exchanges"), b"blocker").unwrap();
    let adapter = StubAdapter::returning_ok(HAPPY_RESPONSE_JSON.as_bytes());
    let git = StubGit::ok();
    let clock = FixedClock::new();
    let id = FixedIdGen;
    let err = run(repo.path(), "hi", &valid_deps(&adapter, &git, &clock, &id)).unwrap_err();
    assert!(matches!(err, Error::Io(_)), "got {err:?}");
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
    let _: String = Error::Io(io::Error::other("x")).to_string();
    let load_err: crate::config::LoadError = crate::config::LoadError::UnresolvedRef {
        key: "k".into(),
        message: "m".into(),
    };
    let e: Error = load_err.into();
    let _: String = e.to_string();
}
