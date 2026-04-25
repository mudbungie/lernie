//! §3.3 resolution order: harness-root, then PATH, then in-process
//! fallback via [`super::super::BinaryResolver`]. Each branch lands in
//! its own test so a regression points at the offending hop.

use super::super::spawn::{BinaryResolver, CurrentExeResolver, which_in_path_env};
use super::super::{SpawnTool, ToolCall, ToolExecutor};
use super::fixtures::{FixedClock, HarnessRoot, StepDir, write_script};
use serde_json::json;
use std::ffi::OsString;
use std::path::PathBuf;
use std::sync::atomic::AtomicBool;
use tempfile::TempDir;

struct StaticResolver(Option<PathBuf>);
impl BinaryResolver for StaticResolver {
    fn lernie_binary(&self) -> Option<PathBuf> {
        self.0.clone()
    }
}

/// Test resolver that returns `path_hit` for the PATH lookup and
/// `lernie` for the in-process fallback. Lets the resolve flow be
/// driven without mutating the process PATH.
struct ScriptedResolver {
    path_hit: Option<PathBuf>,
    lernie: Option<PathBuf>,
}
impl BinaryResolver for ScriptedResolver {
    fn lernie_binary(&self) -> Option<PathBuf> {
        self.lernie.clone()
    }
    fn which_on_path(&self, _prefixed_name: &str) -> Option<PathBuf> {
        self.path_hit.clone()
    }
}

#[test]
fn resolves_external_from_harness_root_first() {
    let root = HarnessRoot::new();
    let installed = root.install("greet", "echo from-harness-root");
    let clock = FixedClock::default();
    let step = StepDir::new();
    let exec = SpawnTool::new(root.path(), &clock);
    let outcome = exec
        .execute(
            ToolCall {
                id: "tu_1",
                name: "greet",
                input: &json!({}),
            },
            &step.path,
            &AtomicBool::new(false),
        )
        .unwrap();
    assert!(installed.is_file(), "installed script vanished");
    assert!(!outcome.is_error);
    assert_eq!(outcome.content, b"from-harness-root\n");
}

#[test]
fn falls_through_to_path_when_harness_root_missing() {
    // PATH lookup is exercised through `which_in_path_env` rather than
    // mutating the live PATH (which races with parallel tests under
    // edition 2024's unsafe set_var).
    let pathdir = TempDir::new().unwrap();
    let bin = pathdir.path().join("lernie-tool-from-path");
    write_script(&bin, "echo found");
    let hit = which_in_path_env("lernie-tool-from-path", Some(pathdir.path().as_os_str()));
    assert_eq!(hit, Some(bin));
}

#[test]
fn path_lookup_skips_dirs_without_the_binary() {
    let a = TempDir::new().unwrap();
    let b = TempDir::new().unwrap();
    let bin = b.path().join("lernie-tool-second");
    write_script(&bin, "echo b");
    // Concatenate two dirs into a single PATH so split_paths walks both.
    let combined: OsString = std::env::join_paths([a.path(), b.path()]).expect("joinable paths");
    let hit = which_in_path_env("lernie-tool-second", Some(&combined));
    assert_eq!(hit, Some(bin));
}

#[test]
fn path_lookup_returns_none_when_unset() {
    assert_eq!(which_in_path_env("lernie-tool-x", None), None);
}

#[test]
fn current_exe_resolver_returns_some_path_for_the_test_binary() {
    // Cargo runs every test inside a real binary, so current_exe()
    // never returns None here. The path itself is opaque — we only
    // assert the trait wires through to a value.
    let r = CurrentExeResolver;
    let p = r.lernie_binary().expect("test binary must have a path");
    assert!(p.is_file(), "current_exe() points at a real file: {:?}", p);
}

#[test]
fn resolves_external_via_path_when_harness_root_misses() {
    // Drop a real script in a tempdir and tell the resolver to return
    // it from PATH lookup. Drives the second hop in `resolve`
    // (line 92→93) without mutating the live PATH env.
    let root = HarnessRoot::new();
    let path_dir = TempDir::new().unwrap();
    let bin = path_dir.path().join("lernie-tool-from-path");
    write_script(&bin, "echo hit-via-path");
    let clock = FixedClock::default();
    let step = StepDir::new();
    let exec = SpawnTool::new(root.path(), &clock).with_resolver(Box::new(ScriptedResolver {
        path_hit: Some(bin),
        lernie: None,
    }));
    let outcome = exec
        .execute(
            ToolCall {
                id: "tu_p",
                name: "from-path",
                input: &json!({}),
            },
            &step.path,
            &AtomicBool::new(false),
        )
        .unwrap();
    assert!(!outcome.is_error);
    assert_eq!(outcome.content, b"hit-via-path\n");
}

#[test]
fn falls_back_to_in_process_when_external_missing() {
    let root = HarnessRoot::new();
    let scripts = TempDir::new().unwrap();
    // Pretend `scripts/fake-lernie` is the lernie binary; when invoked
    // with `tool greet …`, write the args to stdout so the test can
    // confirm the in-process argv shape. Use [`ScriptedResolver`]
    // rather than [`StaticResolver`] so this exercise also covers the
    // resolver's own `lernie_binary` clone path.
    let fake_lernie = scripts.path().join("fake-lernie");
    write_script(&fake_lernie, r#"echo "$@""#);
    let clock = FixedClock::default();
    let step = StepDir::new();
    let exec = SpawnTool::new(root.path(), &clock).with_resolver(Box::new(ScriptedResolver {
        path_hit: None,
        lernie: Some(fake_lernie),
    }));
    let outcome = exec
        .execute(
            ToolCall {
                id: "tu_1",
                name: "greet",
                input: &json!({"k": "v"}),
            },
            &step.path,
            &AtomicBool::new(false),
        )
        .unwrap();
    assert!(!outcome.is_error);
    // The fake lernie echoed `tool greet`, confirming the in-process
    // argv is built per §3.3 ("addressed as `lernie tool <name>`").
    assert_eq!(outcome.content, b"tool greet\n");
}

#[test]
fn not_found_when_external_missing_and_resolver_returns_none() {
    let root = HarnessRoot::new();
    let clock = FixedClock::default();
    let step = StepDir::new();
    let exec = SpawnTool::new(root.path(), &clock).with_resolver(Box::new(StaticResolver(None)));
    let err = exec
        .execute(
            ToolCall {
                id: "tu_1",
                name: "missing-tool",
                input: &json!({}),
            },
            &step.path,
            &AtomicBool::new(false),
        )
        .unwrap_err();
    let msg = err.to_string();
    assert!(msg.contains("missing-tool"), "got: {msg}");
    assert!(msg.contains("not found"), "got: {msg}");
}
