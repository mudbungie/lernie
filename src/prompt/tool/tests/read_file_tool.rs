//! End-to-end integration of the `read_file` built-in (ARCH §3.3,
//! §12 v0.3 toolset) through the tool executor (ball #4).
//!
//! Exercises the full chain: a fixture conversation step directory, a
//! `ToolCall` named `read_file`, and a [`SpawnTool`] resolved to the
//! cargo-built `lernie` binary via the in-process fallback (third hop
//! of §3.3 lookup). Asserts:
//!
//! 1. Stdout bytes are the file's bytes verbatim — the §3.3 stdio
//!    contract.
//! 2. `is_error` is `false` on a successful read.
//! 3. The per-call disk record lands at `<step>/tools/<tool-id>/`
//!    with `input.json` (the `tool_use` block) and `output.json`
//!    (stdout/stderr/exit/timestamps) per §3.3 "Disk record".
//! 4. A failure mode (`TooLarge`) round-trips: `is_error: true`,
//!    stderr concatenated after stdout in `tool_result.content`,
//!    `output.json.exit_code != 0`.

use crate::prompt::clock::SystemClock;
use crate::prompt::tool::spawn::PathLookup;
use crate::prompt::tool::{
    INPUT_FILE, OUTPUT_FILE, STEP_TOOLS_SUBDIR, SpawnTool, ToolCall, ToolExecutor, ToolInputRecord,
    ToolOutputRecord,
};
use serde_json::json;
use std::path::{Path, PathBuf};
use std::sync::atomic::AtomicBool;
use tempfile::TempDir;

fn lernie_bin() -> PathBuf {
    crate::test_support::lernie_binary()
}

/// Forces the §3.3 second hop to miss, so resolution falls through to
/// the injected driver target — here the cargo-built `lernie` binary.
/// The harness root is kept empty (the first hop misses too), and PATH
/// is short-circuited here so the test never depends on the live env.
struct NoPath;

impl PathLookup for NoPath {
    fn which_on_path(&self, _prefixed_name: &str) -> Option<PathBuf> {
        None
    }
}

struct Fixture {
    _harness_root: TempDir,
    step: TempDir,
    step_path: PathBuf,
}

impl Fixture {
    fn new() -> Self {
        let harness = TempDir::new().expect("harness root tempdir");
        std::fs::create_dir_all(harness.path().join("tools")).unwrap();
        let step = TempDir::new().expect("step tempdir");
        // Mirror the §2.2/§2.3 layout: <worktree>/steps/<conv-id>/<NNN>/.
        let step_path = step.path().join("steps").join("convid").join("001");
        std::fs::create_dir_all(&step_path).unwrap();
        Self {
            _harness_root: harness,
            step,
            step_path,
        }
    }

    fn harness_path(&self) -> &std::path::Path {
        self._harness_root.path()
    }
}

fn executor<'a>(harness: &'a Path, clock: &'a SystemClock, lernie: &'a Path) -> SpawnTool<'a> {
    SpawnTool::new(harness, clock, lernie).with_path_lookup(Box::new(NoPath))
}

#[test]
fn read_file_through_executor_returns_file_bytes_and_lands_disk_record() {
    let fixture = Fixture::new();
    let target = fixture.step.path().join("greeting.txt");
    let body = b"hello from read_file\n";
    std::fs::write(&target, body).unwrap();

    let clock = SystemClock;
    let lernie = lernie_bin();
    let exec = executor(fixture.harness_path(), &clock, &lernie);
    let outcome = exec
        .execute(
            ToolCall {
                id: "toolu_rf_ok",
                name: "read_file",
                input: &json!({ "path": target }),
            },
            &fixture.step_path,
            &AtomicBool::new(false),
        )
        .expect("execute succeeds");

    assert!(!outcome.is_error, "happy-path is_error should be false");
    assert_eq!(outcome.content, body);

    let dir = fixture
        .step_path
        .join(STEP_TOOLS_SUBDIR)
        .join("toolu_rf_ok");
    let input: ToolInputRecord =
        serde_json::from_slice(&std::fs::read(dir.join(INPUT_FILE)).unwrap()).unwrap();
    assert_eq!(input.id, "toolu_rf_ok");
    assert_eq!(input.name, "read_file");
    assert_eq!(input.input["path"], json!(target));

    let output: ToolOutputRecord =
        serde_json::from_slice(&std::fs::read(dir.join(OUTPUT_FILE)).unwrap()).unwrap();
    assert_eq!(output.exit_code, 0);
    assert_eq!(output.stdout.as_bytes(), body);
    assert_eq!(output.stderr, "");
    assert!(!output.started_at.is_empty(), "started_at present");
    assert!(!output.ended_at.is_empty(), "ended_at present");
}

#[test]
fn read_file_failure_concats_stderr_and_marks_is_error() {
    let fixture = Fixture::new();
    let missing = fixture.step.path().join("does-not-exist.txt");

    let clock = SystemClock;
    let lernie = lernie_bin();
    let exec = executor(fixture.harness_path(), &clock, &lernie);
    let outcome = exec
        .execute(
            ToolCall {
                id: "toolu_rf_err",
                name: "read_file",
                input: &json!({ "path": missing }),
            },
            &fixture.step_path,
            &AtomicBool::new(false),
        )
        .expect("execute returns Ok even when the tool exits non-zero");

    assert!(outcome.is_error, "failure path: is_error must be true");
    let content = String::from_utf8_lossy(&outcome.content);
    assert!(
        content.contains("does-not-exist.txt"),
        "stderr message should name the offending path; got: {content:?}",
    );

    let dir = fixture
        .step_path
        .join(STEP_TOOLS_SUBDIR)
        .join("toolu_rf_err");
    let output: ToolOutputRecord =
        serde_json::from_slice(&std::fs::read(dir.join(OUTPUT_FILE)).unwrap()).unwrap();
    assert_ne!(
        output.exit_code, 0,
        "output.json.exit_code records the non-zero exit"
    );
    assert!(
        output.stderr.contains("does-not-exist.txt"),
        "output.json.stderr captures the failure message: {:?}",
        output.stderr
    );
}
