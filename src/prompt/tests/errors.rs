//! Pre-branch and adapter error paths for [`crate::prompt::run`].
//!
//! Covers failures the harness surfaces before the branch is spawned
//! (config, role, version guard, soul, workflow) and after the model
//! call returns (in-band `Error` events). Disk/git failures during
//! branch work live in [`super::errors_disk`].

use super::fixtures::*;
use crate::prompt::Error;
use brazen::ErrorKind;
use serde_json::Value;
use std::io;
use std::path::{Path, PathBuf};
use tempfile::TempDir;

/// Drive `run` with default stubs against an explicit config root.
fn run_with_harness(
    repo: &Path,
    msg: &str,
    adapter: &StubAdapter,
    git: &StubGit,
    config_root: &Path,
) -> Result<String, Error> {
    let clock = FixedClock::default();
    let (id, dispatcher) = (FixedIdGen, StubDispatcher::ok());
    let (sleeper, tool_executor) = (StubSleeper::default(), StubToolExecutor::ok());
    crate::prompt::run(
        repo,
        msg,
        &valid_deps(
            adapter,
            &sleeper,
            git,
            &clock,
            &id,
            &dispatcher,
            &tool_executor,
            config_root,
        ),
    )
}

#[test]
fn run_surfaces_global_models_yaml_load_error() {
    let repo = scaffold_repo(VALID_PER_REPO_PROVIDERS_YAML, Some("body"));
    let empty_harness = TempDir::new().unwrap();
    let err = run_with_harness(
        repo.path(),
        "hi",
        &unreachable_adapter(),
        &StubGit::ok(),
        empty_harness.path(),
    )
    .unwrap_err();
    assert!(matches!(err, Error::Config(_)), "got {err:?}");
}

#[test]
fn run_surfaces_per_repo_providers_yaml_load_error() {
    let tmp = TempDir::new().unwrap();
    let err = run_with_stubs(tmp.path(), "hi", &unreachable_adapter(), &StubGit::ok()).unwrap_err();
    assert!(matches!(err, Error::Config(_)), "got {err:?}");
}

#[test]
fn run_surfaces_cross_check_failure() {
    // role.model names a model not declared in models.yaml.
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
fn run_surfaces_version_skew() {
    // The version guard runs before any branch work: a `bz` reporting a
    // version other than the linked crate pin is declined (§4.4).
    let repo = scaffold_repo(VALID_PER_REPO_PROVIDERS_YAML, Some("body"));
    let adapter = StubAdapter::scripted([StubAdapter::reply_ok(b"bz 9.9.9\n")]);
    let git = StubGit::ok();
    let err = run_with_stubs(repo.path(), "hi", &adapter, &git).unwrap_err();
    match err {
        Error::VersionSkew { found, .. } => assert_eq!(found, "9.9.9"),
        other => panic!("expected VersionSkew, got {other:?}"),
    }
    assert!(
        git.runs.borrow().is_empty(),
        "no git before the guard passes"
    );
}

#[test]
fn run_surfaces_version_guard_spawn_failure() {
    let repo = scaffold_repo(VALID_PER_REPO_PROVIDERS_YAML, Some("body"));
    let adapter = StubAdapter::scripted([StubAdapter::reply_err(io::ErrorKind::NotFound, "no bz")]);
    let git = StubGit::ok();
    let err = run_with_stubs(repo.path(), "hi", &adapter, &git).unwrap_err();
    assert!(matches!(err, Error::AdapterSpawn(_)));
    assert!(git.runs.borrow().is_empty());
}

#[test]
fn run_surfaces_workflow_load_error() {
    // Version guard passes, then the retry policy load fails because
    // workflow.yaml is absent.
    let repo = scaffold_repo(VALID_PER_REPO_PROVIDERS_YAML, Some("body"));
    std::fs::remove_file(repo.path().join("workflow.yaml")).unwrap();
    let adapter = StubAdapter::scripted([StubAdapter::reply_ok(&version_line())]);
    let err = run_with_stubs(repo.path(), "hi", &adapter, &StubGit::ok()).unwrap_err();
    assert!(matches!(err, Error::Config(_)), "got {err:?}");
}

#[test]
fn run_surfaces_missing_soul() {
    // Version guard passes, then the soul read fails.
    let repo = scaffold_repo(VALID_PER_REPO_PROVIDERS_YAML, None);
    let adapter = StubAdapter::scripted([StubAdapter::reply_ok(&version_line())]);
    let err = run_with_stubs(repo.path(), "hi", &adapter, &StubGit::ok()).unwrap_err();
    assert!(matches!(err, Error::SoulRead { .. }));
}

#[test]
fn run_surfaces_model_call_spawn_failure() {
    // Version ok, branch spawned, dispatch committed; the model-call
    // `bz` fails at spawn.
    let repo = scaffold_repo(VALID_PER_REPO_PROVIDERS_YAML, Some("body"));
    let adapter = StubAdapter::scripted([
        StubAdapter::reply_ok(&version_line()),
        StubAdapter::reply_err(io::ErrorKind::BrokenPipe, "model call crashed"),
    ]);
    let err = run_with_stubs(repo.path(), "hi", &adapter, &StubGit::ok()).unwrap_err();
    assert!(matches!(err, Error::AdapterSpawn(_)));
}

#[test]
fn run_retries_on_retryable_error_then_completes() {
    // End-to-end retry through `run` (§2.10): a retryable 529 on the
    // first attempt, a clean stream on the second. The harness sleeps
    // the backoff (StubSleeper records it) and `response.json` carries
    // two attempt segments; the branch completes and merges.
    let repo = scaffold_repo(VALID_PER_REPO_PROVIDERS_YAML, Some("body"));
    let harness = scaffold_harness_root();
    let adapter = StubAdapter::scripted([
        StubAdapter::reply_ok(&version_line()),
        StubAdapter::reply_ok(&error_stream(
            ErrorKind::Provider { status: 529 },
            "overloaded",
        )),
        StubAdapter::reply_ok(&happy_response_bytes()),
    ]);
    let git = StubGit::ok();
    let (clock, id, dispatcher) = (FixedClock::default(), FixedIdGen, StubDispatcher::ok());
    let (sleeper, tool_executor) = (StubSleeper::default(), StubToolExecutor::ok());
    crate::prompt::run(
        repo.path(),
        "hi",
        &valid_deps(
            &adapter,
            &sleeper,
            &git,
            &clock,
            &id,
            &dispatcher,
            &tool_executor,
            harness.path(),
        ),
    )
    .unwrap();
    // Exactly one backoff sleep drove the single retry.
    assert_eq!(sleeper.slept.borrow().len(), 1);
    // Two attempt segments on disk (two terminal `end` lines).
    let resp = std::fs::read(repo.path().join("steps/ct-1-deadbeef/001/response.json")).unwrap();
    let ends = resp
        .split(|b| *b == b'\n')
        .filter(|l| *l == br#"{"type":"end"}"#)
        .count();
    assert_eq!(ends, 2);
}

#[test]
fn run_surfaces_adapter_returning_in_band_error() {
    // A non-retryable in-band `Error` (auth) aborts the step (§2.10).
    let repo = scaffold_repo(VALID_PER_REPO_PROVIDERS_YAML, Some("body"));
    let adapter = StubAdapter::happy(&error_stream(ErrorKind::Auth, "unauthorized"));
    let err = run_with_stubs(repo.path(), "hi", &adapter, &StubGit::ok()).unwrap_err();
    match err {
        Error::AdapterError { kind, message } => {
            assert_eq!(kind, "Auth");
            assert_eq!(message, "unauthorized");
        }
        other => panic!("expected AdapterError, got {other:?}"),
    }
}

// Stream-shape error paths (half-stream, malformed events) live in
// [`super::errors_stream`]; parse-shape paths in [`super::errors_parse`].

#[test]
fn error_display_includes_context() {
    // Exercises every `#[error("...")]` format line.
    let _: String = Error::RoleMissing("worker".into()).to_string();
    let _: String = Error::AdapterError {
        kind: "Auth".into(),
        message: "m".into(),
    }
    .to_string();
    let _: String = Error::AdapterHalfStream.to_string();
    let _: String = Error::VersionSkew {
        found: "9.9.9".into(),
        expected: "0.0.2".into(),
    }
    .to_string();
    let _: String = Error::HandshakeMismatch {
        found: Some(2),
        expected: 1,
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
    let _: String = Error::ToolExec {
        tool: "bash".into(),
        source: crate::prompt::ExecError::NotFound {
            name: "bash".into(),
            harness_path: PathBuf::from("/x"),
        },
    }
    .to_string();
    let load_err: crate::config::LoadError = crate::config::LoadError::UnresolvedRef {
        key: "k".into(),
        message: "m".into(),
    };
    let e: Error = load_err.into();
    let _: String = e.to_string();
}
