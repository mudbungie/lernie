//! Pre-branch and adapter error paths for [`crate::prompt::run`].
//!
//! Covers failures the harness surfaces before the branch is spawned
//! (config, role, soul, describe) and after `complete` returns
//! (adapter-reported errors and response-parsing failures). Disk/git
//! failures during branch work live in [`super::errors_disk`].

use super::fixtures::*;
use crate::prompt::Error;
use serde_json::Value;
use std::io;
use std::path::PathBuf;
use tempfile::TempDir;

#[test]
fn run_surfaces_global_providers_yaml_load_error() {
    // Repo has its per-repo providers.yaml + soul, but the harness
    // root is missing — global providers.yaml fails to load.
    let repo = scaffold_repo(VALID_PER_REPO_PROVIDERS_YAML, Some("body"));
    let empty_harness = TempDir::new().unwrap();
    let adapter = unreachable_adapter();
    let git = StubGit::ok();
    let clock = FixedClock::default();
    let id = FixedIdGen;
    let dispatcher = StubDispatcher::ok();
    let tool_executor = StubToolExecutor::ok();
    let err = crate::prompt::run(
        repo.path(),
        "hi",
        &valid_deps(
            &adapter,
            &git,
            &clock,
            &id,
            &dispatcher,
            &tool_executor,
            empty_harness.path(),
        ),
    )
    .unwrap_err();
    assert!(matches!(err, Error::Config(_)), "got {err:?}");
}

#[test]
fn run_surfaces_per_repo_providers_yaml_load_error() {
    // Empty conv-repo (no providers.yaml) — the per-repo loader
    // surfaces the missing file as Config.
    let tmp = TempDir::new().unwrap();
    let err = run_with_stubs(tmp.path(), "hi", &unreachable_adapter(), &StubGit::ok()).unwrap_err();
    assert!(matches!(err, Error::Config(_)), "got {err:?}");
}

#[test]
fn run_surfaces_cross_check_failure() {
    // role.model names a model not declared in the global file —
    // ProvidersConfig::load runs the §4.3 cross-check.
    let bad_per_repo = r#"
roles:
  worker:
    provider: anthropic
    model: nonexistent-model
"#;
    let repo = scaffold_repo(bad_per_repo, Some("body"));
    let err =
        run_with_stubs(repo.path(), "hi", &unreachable_adapter(), &StubGit::ok()).unwrap_err();
    assert!(matches!(err, Error::Config(_)));
}

#[test]
fn run_rejects_when_worker_role_missing() {
    // No `worker` role in the per-repo providers.yaml — the prompt
    // path surfaces this as RoleMissing rather than letting it slip
    // through as a downstream lookup failure.
    let no_worker = r#"
roles:
  compactor:
    provider: anthropic
    model: claude-sonnet-4-7
"#;
    let repo = scaffold_repo(no_worker, Some("body"));
    let err =
        run_with_stubs(repo.path(), "hi", &unreachable_adapter(), &StubGit::ok()).unwrap_err();
    match err {
        Error::RoleMissing(role) => assert_eq!(role, "worker"),
        other => panic!("expected RoleMissing, got {other:?}"),
    }
}

#[test]
fn run_surfaces_missing_soul() {
    let repo = scaffold_repo(VALID_PER_REPO_PROVIDERS_YAML, None);
    let err =
        run_with_stubs(repo.path(), "hi", &unreachable_adapter(), &StubGit::ok()).unwrap_err();
    assert!(matches!(err, Error::SoulRead { .. }));
}

#[test]
fn run_surfaces_describe_spawn_failure() {
    // First adapter call is `describe`; failure here surfaces as
    // AdapterSpawn and no branch is created (describe precedes
    // worktree add, so an adapter fault leaves no stray ref).
    let repo = scaffold_repo(VALID_PER_REPO_PROVIDERS_YAML, Some("body"));
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
    let repo = scaffold_repo(VALID_PER_REPO_PROVIDERS_YAML, Some("body"));
    let adapter = StubAdapter::scripted([StubAdapter::reply_ok(b"{ not json")]);
    let err = run_with_stubs(repo.path(), "hi", &adapter, &StubGit::ok()).unwrap_err();
    assert!(matches!(err, Error::AdapterJson(_)), "got {err:?}");
}

#[test]
fn run_surfaces_describe_endpoint_env_wrong_type() {
    // `endpoint_env` must be an array of strings. Anything else
    // surfaces as a parse error — accepting it would silently drop
    // the adapter's declared config.
    let repo = scaffold_repo(VALID_PER_REPO_PROVIDERS_YAML, Some("body"));
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
    let repo = scaffold_repo(VALID_PER_REPO_PROVIDERS_YAML, Some("body"));
    let adapter = StubAdapter::scripted([
        StubAdapter::reply_ok(STUB_DESCRIBE_JSON.as_bytes()),
        StubAdapter::reply_err(io::ErrorKind::BrokenPipe, "complete crashed"),
    ]);
    let err = run_with_stubs(repo.path(), "hi", &adapter, &StubGit::ok()).unwrap_err();
    assert!(matches!(err, Error::AdapterSpawn(_)));
}

#[test]
fn run_surfaces_adapter_returning_in_band_error() {
    let repo = scaffold_repo(VALID_PER_REPO_PROVIDERS_YAML, Some("body"));
    // §4.4 streaming error event: terminal `error` with kind/message
    // (and optional http_status / retry_after_seconds). One JSONL
    // line is enough — the assembler treats it as terminal.
    let error_jsonl = br#"{"type":"error","kind":"fatal","http_status":401,"message":"boom"}
"#;
    let adapter = StubAdapter::happy(error_jsonl);
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

// Streaming-shape error paths (malformed JSONL, half-stream,
// missing stop_reason, malformed tool_use input, post-terminal
// strays) live in [`super::errors_stream`].

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
    let _: String = Error::SoulRead {
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
