//! Shared fixtures for `prompt::tests::*`.
//!
//! Stubs (`StubAdapter`, `StubGit`, `StubDispatcher`) live in
//! [`super::stubs`]; §4.4 JSONL stream synthesis helpers
//! (`streaming_response`, `happy_response_bytes`, `parse_jsonl`) live
//! in [`super::streams`]. Both are re-exported here so test files keep
//! the single `use super::fixtures::*;` import surface.

pub(super) use super::streams::{happy_response_bytes, parse_jsonl, streaming_response};
pub(super) use super::stubs::{
    STUB_DESCRIBE_JSON, StubAdapter, StubDispatcher, StubGit, unreachable_adapter,
};
pub(super) use super::tool_stub::StubToolExecutor;

use crate::prompt::{Clock, Deps, IdGen};
use std::cell::RefCell;
use std::path::{Path, PathBuf};
use tempfile::TempDir;

/// Monotonic `iso-N`/`ct-N` [`Clock`].
#[derive(Default)]
pub(super) struct FixedClock {
    iso_calls: RefCell<u32>,
    compact_calls: RefCell<u32>,
}
impl Clock for FixedClock {
    fn now_iso8601(&self) -> String {
        *self.iso_calls.borrow_mut() += 1;
        format!("iso-{}", self.iso_calls.borrow())
    }
    fn now_compact(&self) -> String {
        *self.compact_calls.borrow_mut() += 1;
        format!("ct-{}", self.compact_calls.borrow())
    }
}

pub(super) struct FixedIdGen;
impl IdGen for FixedIdGen {
    fn short(&self) -> String {
        "deadbeef".into()
    }
}

/// Global `<harness-root>/providers.yaml` (ARCH §4.1).
pub(super) const VALID_GLOBAL_PROVIDERS_YAML: &str = r#"
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
    capabilities: [tool_use_native]
    context_window: 200000
"#;

/// Per-repo `<conv-repo>/providers.yaml` (ARCH §4.3).
pub(super) const VALID_PER_REPO_PROVIDERS_YAML: &str = r#"
roles:
  worker:
    provider: anthropic
    model: claude-sonnet-4-7
"#;

/// Lay out a v0.3 conv repo (§2.2): per-repo `providers.yaml` and
/// optional `souls/worker.md`.
pub(super) fn scaffold_repo(per_repo_yaml: &str, worker_soul: Option<&str>) -> TempDir {
    let tmp = TempDir::new().unwrap();
    std::fs::write(tmp.path().join("providers.yaml"), per_repo_yaml).unwrap();
    if let Some(body) = worker_soul {
        let souls = tmp.path().join("souls");
        std::fs::create_dir_all(&souls).unwrap();
        std::fs::write(souls.join("worker.md"), body).unwrap();
    }
    tmp
}

/// Lay out a temp harness root (ARCH §2.2) with a global
/// `providers.yaml`.
pub(super) fn scaffold_harness_root() -> TempDir {
    let tmp = TempDir::new().unwrap();
    std::fs::write(
        tmp.path().join("providers.yaml"),
        VALID_GLOBAL_PROVIDERS_YAML,
    )
    .unwrap();
    tmp
}

pub(super) fn valid_deps<'a>(
    adapter: &'a StubAdapter,
    git: &'a StubGit,
    clock: &'a FixedClock,
    id: &'a FixedIdGen,
    dispatcher: &'a StubDispatcher,
    tool_executor: &'a StubToolExecutor,
    harness_root: &'a Path,
) -> Deps<'a> {
    Deps {
        adapter,
        git,
        clock,
        id_gen: id,
        dispatcher,
        tool_executor,
        harness_root,
    }
}

/// Drive [`crate::prompt::run`] with default stubs.
pub(super) fn run_with_stubs(
    repo: &Path,
    msg: &str,
    adapter: &StubAdapter,
    git: &StubGit,
) -> Result<String, crate::prompt::Error> {
    let harness = scaffold_harness_root();
    let (clock, id, dispatcher) = (FixedClock::default(), FixedIdGen, StubDispatcher::ok());
    let tool_executor = StubToolExecutor::ok();
    crate::prompt::run(
        repo,
        msg,
        &valid_deps(
            adapter,
            git,
            &clock,
            &id,
            &dispatcher,
            &tool_executor,
            harness.path(),
        ),
    )
}

/// Conv worktree path for the standard fixtures (FixedClock=ct-1,
/// FixedIdGen=deadbeef → `<repo>/ct-1-deadbeef/`, §2.2).
pub(super) fn worktree_path(repo: &Path) -> PathBuf {
    repo.join("ct-1-deadbeef")
}
