//! Shared fixtures for `prompt::tests::*`.
//!
//! Stubs (`StubAdapter`, `StubSleeper`, `StubGit`, `StubDispatcher`)
//! live in [`super::stubs`]; brazen `v=1` NDJSON synthesis helpers
//! (`stream_of`, `error_stream`, `happy_response_bytes`, `parse_jsonl`)
//! live in [`super::streams`]. Both are re-exported here so test files
//! keep the single `use super::fixtures::*;` import surface.

pub(super) use super::streams::{
    Block, error_stream, happy_response_bytes, parse_jsonl, stream_of,
};
pub(super) use super::stubs::{
    StubAdapter, StubDispatcher, StubGit, StubSleeper, unreachable_adapter, version_line,
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

/// Global `<harness-root>/models.yaml` (ARCH §4.2) — capabilities and
/// context windows only; endpoints and auth are brazen's (§4.1).
pub(super) const VALID_GLOBAL_MODELS_YAML: &str = r#"
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
    tools: [bash, read_file]
"#;

/// Minimal `workflow.yaml` (ARCH §6). The retry block is omitted so
/// [`crate::config::RetryConfig::default`] (3 attempts, exponential)
/// applies unless a test overrides it.
pub(super) const VALID_WORKFLOW_YAML: &str = "events: {}\n";

/// Lay out a v0.6 conv repo (§2.2): per-repo `providers.yaml`, a
/// `workflow.yaml` (read for the retry policy, §2.10), and optional
/// `souls/worker.md`.
pub(super) fn scaffold_repo(per_repo_yaml: &str, worker_soul: Option<&str>) -> TempDir {
    scaffold_repo_with_workflow(per_repo_yaml, VALID_WORKFLOW_YAML, worker_soul)
}

/// Like [`scaffold_repo`] but with an explicit `workflow.yaml` body so a
/// test can pin a `retry:` block.
pub(super) fn scaffold_repo_with_workflow(
    per_repo_yaml: &str,
    workflow_yaml: &str,
    worker_soul: Option<&str>,
) -> TempDir {
    let tmp = TempDir::new().unwrap();
    std::fs::write(tmp.path().join("providers.yaml"), per_repo_yaml).unwrap();
    std::fs::write(tmp.path().join("workflow.yaml"), workflow_yaml).unwrap();
    if let Some(body) = worker_soul {
        let souls = tmp.path().join("souls");
        std::fs::create_dir_all(&souls).unwrap();
        std::fs::write(souls.join("worker.md"), body).unwrap();
    }
    tmp
}

/// Lay out a temp harness root (ARCH §2.2) with a global `models.yaml`.
pub(super) fn scaffold_harness_root() -> TempDir {
    let tmp = TempDir::new().unwrap();
    std::fs::write(tmp.path().join("models.yaml"), VALID_GLOBAL_MODELS_YAML).unwrap();
    tmp
}

/// Harness root whose `models.yaml` names an `adapter:` override (§4.2)
/// — the version guard is skipped and the MessageStart.v handshake
/// governs (§4.4). The override path need not exist: the stub adapter
/// ignores the binary.
pub(super) fn scaffold_harness_root_with_adapter(adapter: &str) -> TempDir {
    let tmp = TempDir::new().unwrap();
    let yaml = format!("adapter: {adapter}\n{VALID_GLOBAL_MODELS_YAML}");
    std::fs::write(tmp.path().join("models.yaml"), yaml).unwrap();
    tmp
}

#[allow(clippy::too_many_arguments)]
pub(super) fn valid_deps<'a>(
    adapter: &'a StubAdapter,
    sleeper: &'a StubSleeper,
    git: &'a StubGit,
    clock: &'a FixedClock,
    id: &'a FixedIdGen,
    dispatcher: &'a StubDispatcher,
    tool_executor: &'a StubToolExecutor,
    config_root: &'a Path,
) -> Deps<'a> {
    Deps {
        adapter,
        sleeper,
        git,
        clock,
        id_gen: id,
        dispatcher,
        tool_executor,
        config_root,
        stop: never_stopped(),
        launcher: no_launch(),
    }
}

/// The default exit-protocol launcher for tests off the §2.11 launch
/// path: the production [`AdvanceLauncher`] no-op, handed out as a
/// static so `valid_deps` needs no extra parameter. Launch-path tests
/// override `deps.launcher` with a recording stub instead — the same
/// pattern as [`never_stopped`].
///
/// [`AdvanceLauncher`]: crate::prompt::inbox::AdvanceLauncher
pub(super) fn no_launch() -> &'static crate::prompt::inbox::AdvanceLauncher {
    static NO_LAUNCH: crate::prompt::inbox::AdvanceLauncher = crate::prompt::inbox::AdvanceLauncher;
    &NO_LAUNCH
}

/// A stop flag that is never set — the default for tests off the §2.9
/// stop path. Returned from a function (not a bare `&static`, which const-
/// promotes and reads as an uncovered line) so `valid_deps` can hand out a
/// borrow without every caller threading one. Stop-path tests construct
/// their own live [`AtomicBool`] instead.
pub(super) fn never_stopped() -> &'static std::sync::atomic::AtomicBool {
    static NEVER: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
    &NEVER
}

/// Drive [`crate::prompt::run`] with default stubs (no-override path:
/// the adapter script leads with the version-guard reply).
pub(super) fn run_with_stubs(
    repo: &Path,
    msg: &str,
    adapter: &StubAdapter,
    git: &StubGit,
) -> Result<String, crate::prompt::Error> {
    let harness = scaffold_harness_root();
    let (clock, id, dispatcher) = (FixedClock::default(), FixedIdGen, StubDispatcher::ok());
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
            harness.path(),
        ),
    )
}

/// Conv worktree path for the standard fixtures (FixedClock=ct-1,
/// FixedIdGen=deadbeef → `<repo>/ct-1-deadbeef/`, §2.2).
pub(super) fn worktree_path(repo: &Path) -> PathBuf {
    repo.join("ct-1-deadbeef")
}
