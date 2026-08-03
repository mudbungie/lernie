//! Recording [`ToolExecutor`] for the prompt-loop tests.
//!
//! Lives in its own file so [`super::fixtures`] stays under the
//! repo's 300-line code-file cap. The stub satisfies the §3.3 disk
//! contract minimally — it lands `input.json` and `output.json`
//! under `<step_dir>/tools/<id>/` so the loop's per-call commit step
//! has something to `git add` — without spawning a subprocess. Ball
//! #4's [`crate::prompt::SpawnTool`] is the production impl.

use crate::prompt::ExecError;
use crate::prompt::tool::{
    INPUT_FILE, OUTPUT_FILE, ToolCall, ToolExecutor, ToolInputRecord, ToolOutcome,
    ToolOutputRecord, atomic_write_json, tool_call_dir,
};
use std::cell::RefCell;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::atomic::AtomicBool;

/// One observed `execute` call: (step_dir, tool_use_id, tool_name,
/// tool_input). Step seq is folded into `step_dir`'s tail.
pub(super) type ObservedCall = (PathBuf, String, String, serde_json::Value);

/// Returns `(content=replies[name].unwrap_or("stub:<name>"),
/// is_error=false)` per tool call. `fail_on(name)` short-circuits the tool call
/// with [`ExecError::Spawn`] instead — exercises the loop's
/// tool-failure surface.
#[derive(Default)]
pub(super) struct StubToolExecutor {
    pub(super) invocations: RefCell<Vec<ObservedCall>>,
    replies: HashMap<String, String>,
    fail_on: Option<String>,
    /// When set, `execute` on this tool name returns
    /// [`ExecError::KilledBySignal`] — a tool cut down by a signal. If
    /// `kill_sets_stop`, it first flips the injected stop flag, the §2.9
    /// group-SIGTERM shape (the executor's handler ran and this limb died
    /// with it) so the loop classifies the kill as a stop rather than a
    /// fault.
    kill_on: Option<String>,
    kill_sets_stop: bool,
}

impl StubToolExecutor {
    pub(super) fn ok() -> Self {
        Self::default()
    }
    pub(super) fn with_reply(name: &str, content: &str) -> Self {
        let mut replies = HashMap::new();
        replies.insert(name.to_string(), content.to_string());
        Self {
            replies,
            ..Self::default()
        }
    }
    pub(super) fn failing_on(name: &str) -> Self {
        Self {
            fail_on: Some(name.to_string()),
            ..Self::default()
        }
    }
    /// A tool killed by a signal *without* a stop pending: a genuine
    /// crash (SIGSEGV, …) the loop must surface as a fault (§2.10).
    pub(super) fn killed_on(name: &str) -> Self {
        Self {
            kill_on: Some(name.to_string()),
            ..Self::default()
        }
    }
    /// A tool killed by the executor's own group SIGTERM mid-stop: sets
    /// the injected stop flag and returns `KilledBySignal`, the shape the
    /// loop must read as the clean stopped exit (§2.9 step 3).
    pub(super) fn stop_killed_on(name: &str) -> Self {
        Self {
            kill_on: Some(name.to_string()),
            kill_sets_stop: true,
            ..Self::default()
        }
    }
}

impl ToolExecutor for StubToolExecutor {
    fn execute(
        &self,
        call: ToolCall<'_>,
        step_dir: &Path,
        stop: &AtomicBool,
        _output_bound: Option<crate::config::ToolOutputBound>,
    ) -> Result<ToolOutcome, ExecError> {
        self.invocations.borrow_mut().push((
            step_dir.to_path_buf(),
            call.id.to_string(),
            call.name.to_string(),
            call.input.clone(),
        ));
        if self.fail_on.as_deref() == Some(call.name) {
            return Err(ExecError::Spawn {
                name: call.name.to_string(),
                source: std::io::Error::other(format!("stub fail on {}", call.name)),
            });
        }
        if self.kill_on.as_deref() == Some(call.name) {
            if self.kill_sets_stop {
                stop.store(true, std::sync::atomic::Ordering::SeqCst);
            }
            return Err(ExecError::KilledBySignal {
                name: call.name.to_string(),
                signal: 15,
            });
        }
        // Mirror the production §3.3 disk contract so the loop's
        // commit step has files to `git add`. Atomic-rename keeps
        // partial captures out of `git status`. The loop guarantees
        // `step_dir` already exists by the time the executor runs
        // (the snapshot commit landed it), so `create_dir_all`
        // failing here would be a test-fixture bug, not a runtime
        // error path.
        let dir = tool_call_dir(step_dir, call.id);
        std::fs::create_dir_all(&dir).expect("step_dir exists by §2.10 ordering");
        atomic_write_json(
            &dir,
            INPUT_FILE,
            &ToolInputRecord {
                id: call.id.to_string(),
                name: call.name.to_string(),
                input: call.input.clone(),
            },
        )?;
        let content = self
            .replies
            .get(call.name)
            .cloned()
            .unwrap_or_else(|| format!("stub:{}", call.name));
        atomic_write_json(
            &dir,
            OUTPUT_FILE,
            &ToolOutputRecord {
                stdout: content.clone(),
                stderr: String::new(),
                exit_code: 0,
                started_at: "stub-start".into(),
                ended_at: "stub-end".into(),
            },
        )?;
        Ok(ToolOutcome {
            content: content.into_bytes(),
            is_error: false,
        })
    }
}
