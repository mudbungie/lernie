//! Per-step tool-call orchestration (ARCH §2.5, §3.3).
//!
//! When a step's response carries `tool_use` blocks, the loop hands
//! each one to [`crate::prompt::ToolExecutor`] in emission order. The
//! executor lands `input.json` and `output.json` under
//! `steps/<conv-id>/<NNN>/tools/<tool-id>/` and returns a
//! [`ToolOutcome`]. This module then commits those records on the
//! branch — one commit per call, per §3.3 "Commit-per-tool-call" —
//! and converts the outcome into the wire `tool_result` block the
//! loop feeds into the next step's request (§2.5).
//!
//! Living in a sibling module keeps `super`'s `run_exchange` body
//! under the repo's 300-line code-file cap.

use crate::prompt::Deps;
use crate::prompt::Error;
use crate::prompt::tool::{STEP_TOOLS_SUBDIR, ToolCall as ToolUse, ToolOutcome};
use crate::provider::wire::ContentBlock;
use std::path::Path;
use std::sync::atomic::AtomicBool;

/// Drive every `tool_use` block in `response_content` through the
/// executor and commit each per-call record. Returns the
/// `tool_result` blocks in emission order.
///
/// `stop_flag` is the harness's cancel sentinel (PRINCIPLES "Stops
/// are aggressive"); v0.3 wires an always-false local since no signal
/// handler is hooked up yet, so the executor's polling loop never
/// trips. v0.4 connects this to the real harness signal handler.
pub(super) fn run_tool_calls(
    worktree_path: &Path,
    step_dir_rel_str: &str,
    conv_id: &str,
    response_content: &[ContentBlock],
    deps: &Deps<'_>,
) -> Result<Vec<ContentBlock>, Error> {
    let step_dir_abs = worktree_path.join(step_dir_rel_str);
    let stop_flag = AtomicBool::new(false);
    let mut tool_results: Vec<ContentBlock> = Vec::new();
    for block in response_content {
        let ContentBlock::ToolUse { id, name, input } = block else {
            continue;
        };
        let outcome = deps
            .tool_executor
            .execute(ToolUse { id, name, input }, &step_dir_abs, &stop_flag)
            .map_err(|source| Error::ToolExec {
                tool: name.clone(),
                source,
            })?;
        commit_tool_call(worktree_path, step_dir_rel_str, conv_id, name, id, deps)?;
        tool_results.push(outcome_to_tool_result(id, &outcome));
    }
    Ok(tool_results)
}

/// `git add` the `input.json` + `output.json` the executor just landed
/// for `tool_use_id`, then commit the pair as a single per-call
/// commit per ARCH §3.3 "Commit-per-tool-call".
fn commit_tool_call(
    worktree_path: &Path,
    step_dir_rel_str: &str,
    conv_id: &str,
    tool_name: &str,
    tool_use_id: &str,
    deps: &Deps<'_>,
) -> Result<(), Error> {
    let call_dir_rel = format!("{step_dir_rel_str}/{STEP_TOOLS_SUBDIR}/{tool_use_id}");
    deps.git
        .run(worktree_path, &["add", call_dir_rel.as_str()])
        .map_err(|source| Error::Git {
            op: "tool add",
            source,
        })?;
    let msg = format!("tool {tool_name} [{conv_id}/{tool_use_id}]");
    deps.git
        .run(worktree_path, &["commit", "-m", msg.as_str()])
        .map_err(|source| Error::Git {
            op: "tool commit",
            source,
        })
}

/// Turn the executor's [`ToolOutcome`] into the wire-shape
/// `tool_result` block the next step's user message carries (ARCH
/// §3.3 "Wire `tool_result` framing is application-layer"). Stdout
/// bytes round-trip through lossy UTF-8 because the v0.3
/// `tool_result.content` is a string per [`ContentBlock::ToolResult`];
/// non-UTF-8 tool output is not common enough in v0.3 to justify
/// holding a separate bytes path.
fn outcome_to_tool_result(tool_use_id: &str, outcome: &ToolOutcome) -> ContentBlock {
    ContentBlock::ToolResult {
        tool_use_id: tool_use_id.to_string(),
        content: String::from_utf8_lossy(&outcome.content).into_owned(),
        is_error: outcome.is_error,
    }
}
