//! Resolution / spawn / disk-record I/O failure modes — every
//! [`super::super::ExecError`] variant gets a constructive test.

use super::super::spawn::{BinaryResolver, which_in_path_env};
use super::super::{ExecError, SpawnTool, ToolCall, ToolExecutor};
use super::fixtures::{FixedClock, HarnessRoot, StepDir, write_script};
use serde_json::json;
use std::path::PathBuf;
use std::sync::atomic::AtomicBool;
use tempfile::TempDir;

struct StaticResolver(Option<PathBuf>);
impl BinaryResolver for StaticResolver {
    fn lernie_binary(&self) -> Option<PathBuf> {
        self.0.clone()
    }
}

#[test]
fn spawn_error_when_resolved_binary_is_not_executable() {
    // Resolution succeeds (we drop a *file* under `tools/`) but
    // `Command::spawn` rejects it because it is not chmod +x.
    let root = HarnessRoot::new();
    let bin = root.dir.path().join(super::super::TOOLS_DIR).join(format!(
        "{}{}",
        super::super::EXTERNAL_PREFIX,
        "not-exec"
    ));
    std::fs::write(&bin, b"not a real binary").unwrap();
    let clock = FixedClock::default();
    let step = StepDir::new();
    let exec = SpawnTool::new(root.path(), &clock);
    let err = exec
        .execute(
            ToolCall {
                id: "tu_e1",
                name: "not-exec",
                input: &json!({}),
            },
            &step.path,
            &AtomicBool::new(false),
        )
        .unwrap_err();
    match err {
        ExecError::Spawn { name, .. } => assert_eq!(name, "not-exec"),
        other => panic!("expected Spawn, got {other:?}"),
    }
}

#[test]
fn io_error_when_step_dir_is_a_file() {
    // `create_dir_all` refuses when the leaf is an existing file —
    // exercise the [`ExecError::Io`] branch.
    let root = HarnessRoot::new();
    root.install("anything", "true");
    let scratch = TempDir::new().unwrap();
    let bogus_step = scratch.path().join("not-a-dir");
    std::fs::write(&bogus_step, b"i am a file").unwrap();
    let clock = FixedClock::default();
    let exec = SpawnTool::new(root.path(), &clock);
    let err = exec
        .execute(
            ToolCall {
                id: "tu_e2",
                name: "anything",
                input: &json!({}),
            },
            &bogus_step,
            &AtomicBool::new(false),
        )
        .unwrap_err();
    match err {
        ExecError::Io { dir, .. } => {
            assert!(dir.ends_with("tu_e2"), "wrong dir in error: {:?}", dir);
        }
        other => panic!("expected Io, got {other:?}"),
    }
}

#[test]
fn not_found_error_path_includes_harness_root_lookup_path() {
    let root = HarnessRoot::new();
    let clock = FixedClock::default();
    let step = StepDir::new();
    let exec = SpawnTool::new(root.path(), &clock).with_resolver(Box::new(StaticResolver(None)));
    let err = exec
        .execute(
            ToolCall {
                id: "tu_e3",
                name: "ghost",
                input: &json!({}),
            },
            &step.path,
            &AtomicBool::new(false),
        )
        .unwrap_err();
    match err {
        ExecError::NotFound { name, harness_path } => {
            assert_eq!(name, "ghost");
            assert!(
                harness_path.ends_with("tools/lernie-tool-ghost"),
                "wrong harness path: {:?}",
                harness_path
            );
        }
        other => panic!("expected NotFound, got {other:?}"),
    }
}

#[test]
fn which_in_path_misses_when_no_dir_carries_the_binary() {
    let empty = TempDir::new().unwrap();
    assert_eq!(
        which_in_path_env("lernie-tool-nope", Some(empty.path().as_os_str())),
        None
    );
}

#[test]
fn which_in_path_live_env_returns_a_value_for_a_real_binary() {
    // Cover the live `which_in_path` (env-var-reading) wrapper. `sh`
    // is on PATH on every POSIX runner. We're not asserting where —
    // just that the env-read branch produces *something*.
    use super::super::spawn::which_in_path_env as wpe;
    let path = std::env::var_os("PATH");
    let hit = wpe("sh", path.as_deref());
    assert!(hit.is_some(), "expected /bin/sh or similar on PATH");
}

#[test]
fn live_which_in_path_reads_path_env_without_panicking() {
    // Covers the `var_os("PATH")` line in `which_in_path`. The
    // result is `Option` either way — under cargo test PATH is
    // typically set, but the wrapper must tolerate it being unset
    // (the `?` short-circuits) without us asserting a specific
    // outcome.
    let _ = super::super::spawn::which_in_path("lernie-tool-definitely-not-installed");
}

#[test]
fn write_script_helper_is_round_tripped_by_the_fixture() {
    // Sanity: `fixtures::write_script` produces a runnable script.
    // Without this the resolution / cascade tests would fail in a
    // confusing way.
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("hi.sh");
    write_script(&path, "echo hi");
    let out = std::process::Command::new(&path).output().unwrap();
    assert!(out.status.success());
    assert_eq!(out.stdout, b"hi\n");
}
