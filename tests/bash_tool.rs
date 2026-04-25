//! End-to-end integration of the `bash` built-in (ARCH §3.3, §12 v0.3
//! toolset) through the tool executor.
//!
//! Mirrors `tests/read_file_tool.rs` but exercises the bash surface:
//!
//! 1. Stdout bytes from the spawned shell are surfaced verbatim — the
//!    §3.3 stdio contract.
//! 2. `is_error` is `false` on a zero-exit command.
//! 3. The per-call disk record lands at `<step>/tools/<tool-id>/`
//!    with `input.json` and `output.json` per §3.3 "Disk record".
//! 4. A failure mode (`false`) round-trips: `is_error: true`, stderr
//!    concatenated after stdout in `tool_result.content`,
//!    `output.json.exit_code != 0`.

use lernie::prompt::clock::SystemClock;
use lernie::prompt::tool::spawn::BinaryResolver;
use lernie::prompt::tool::{
    INPUT_FILE, OUTPUT_FILE, STEP_TOOLS_SUBDIR, SpawnTool, ToolCall, ToolExecutor, ToolInputRecord,
    ToolOutputRecord,
};
use serde_json::json;
use std::path::PathBuf;
use std::sync::atomic::AtomicBool;
use tempfile::TempDir;

fn lernie_bin() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_lernie"))
}

/// Resolver that pins the in-process route to the cargo-built lernie
/// binary and forces external lookups to miss. The harness root is
/// kept empty so resolution falls all the way through to the
/// in-process fallback.
struct InProcessLernie {
    lernie: PathBuf,
}

impl BinaryResolver for InProcessLernie {
    fn lernie_binary(&self) -> Option<PathBuf> {
        Some(self.lernie.clone())
    }
    fn which_on_path(&self, _prefixed_name: &str) -> Option<PathBuf> {
        None
    }
}

struct Fixture {
    _harness_root: TempDir,
    _step: TempDir,
    step_path: PathBuf,
}

impl Fixture {
    fn new() -> Self {
        let harness = TempDir::new().expect("harness root tempdir");
        std::fs::create_dir_all(harness.path().join("tools")).unwrap();
        let step = TempDir::new().expect("step tempdir");
        let step_path = step.path().join("steps").join("convid").join("001");
        std::fs::create_dir_all(&step_path).unwrap();
        Self {
            _harness_root: harness,
            _step: step,
            step_path,
        }
    }

    fn harness_path(&self) -> &std::path::Path {
        self._harness_root.path()
    }
}

fn executor<'a>(harness: &'a std::path::Path, clock: &'a SystemClock) -> SpawnTool<'a> {
    SpawnTool::new(harness, clock).with_resolver(Box::new(InProcessLernie {
        lernie: lernie_bin(),
    }))
}

#[test]
fn bash_through_executor_returns_stdout_and_lands_disk_record() {
    let fixture = Fixture::new();

    let clock = SystemClock;
    let exec = executor(fixture.harness_path(), &clock);
    let outcome = exec
        .execute(
            ToolCall {
                id: "toolu_bash_ok",
                name: "bash",
                input: &json!({ "command": "printf hello-from-bash" }),
            },
            &fixture.step_path,
            &AtomicBool::new(false),
        )
        .expect("execute succeeds");

    assert!(!outcome.is_error, "happy-path is_error should be false");
    assert_eq!(outcome.content, b"hello-from-bash");

    let dir = fixture
        .step_path
        .join(STEP_TOOLS_SUBDIR)
        .join("toolu_bash_ok");
    let input: ToolInputRecord =
        serde_json::from_slice(&std::fs::read(dir.join(INPUT_FILE)).unwrap()).unwrap();
    assert_eq!(input.id, "toolu_bash_ok");
    assert_eq!(input.name, "bash");
    assert_eq!(input.input["command"], json!("printf hello-from-bash"));

    let output: ToolOutputRecord =
        serde_json::from_slice(&std::fs::read(dir.join(OUTPUT_FILE)).unwrap()).unwrap();
    assert_eq!(output.exit_code, 0);
    assert_eq!(output.stdout, "hello-from-bash");
    assert_eq!(output.stderr, "");
    assert!(!output.started_at.is_empty(), "started_at present");
    assert!(!output.ended_at.is_empty(), "ended_at present");
}

#[test]
fn bash_failure_concats_stderr_and_marks_is_error() {
    let fixture = Fixture::new();

    let clock = SystemClock;
    let exec = executor(fixture.harness_path(), &clock);
    let outcome = exec
        .execute(
            ToolCall {
                id: "toolu_bash_err",
                name: "bash",
                input: &json!({
                    "command": "printf prelude; printf complaint 1>&2; exit 7"
                }),
            },
            &fixture.step_path,
            &AtomicBool::new(false),
        )
        .expect("execute returns Ok even when the shell exits non-zero");

    assert!(outcome.is_error, "failure path: is_error must be true");
    let content = String::from_utf8_lossy(&outcome.content);
    assert!(
        content.contains("prelude"),
        "stdout fragment missing: {content:?}",
    );
    assert!(
        content.contains("complaint"),
        "stderr fragment should be concatenated after stdout: {content:?}",
    );

    let dir = fixture
        .step_path
        .join(STEP_TOOLS_SUBDIR)
        .join("toolu_bash_err");
    let output: ToolOutputRecord =
        serde_json::from_slice(&std::fs::read(dir.join(OUTPUT_FILE)).unwrap()).unwrap();
    assert_eq!(output.exit_code, 7);
    assert_eq!(output.stdout, "prelude");
    assert_eq!(output.stderr, "complaint");
}
