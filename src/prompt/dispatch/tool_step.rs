//! Per-step tool-call orchestration (ARCH §2.5, §3.3).
//!
//! When a step's response carries `tool_use` blocks, the loop hands
//! each one to [`crate::prompt::ToolExecutor`] in emission order. The
//! executor lands `input.json` and `output.json` under
//! `<conv-repo>/steps/<conv-id>/<NNN>/tools/<tool-id>/` — outside
//! every worktree (§2.2 / §2.3), not git-tracked. The per-tool
//! diagnostic record is *not* a commit (§3.3 amended); only worktree
//! side effects from worktree-modifying tools (e.g. `bash` editing
//! files) commit, and that happens in the tool's own subprocess
//! contract, not here.
//!
//! Living in a sibling module keeps `super`'s `run_exchange` body
//! under the repo's 300-line code-file cap.

use crate::prompt::Deps;
use crate::prompt::Error;
use crate::prompt::tool::{ToolCall as ToolUse, ToolOutcome};
use crate::provider::wire::ContentBlock;
use std::path::Path;
use std::sync::atomic::AtomicBool;

/// Drive every `tool_use` block in `response_content` through the
/// executor. The executor lands the per-call record under
/// `<conv-repo>/steps/<conv-id>/<NNN>/tools/<tool-id>/`. Returns the
/// `tool_result` blocks in emission order.
///
/// `stop_flag` is the harness's cancel sentinel (PRINCIPLES "Stops
/// are aggressive"); v0.3 wires an always-false local since no signal
/// handler is hooked up yet, so the executor's polling loop never
/// trips. v0.4 connects this to the real harness signal handler.
pub(super) fn run_tool_calls(
    conv_repo: &Path,
    step_dir_rel_str: &str,
    response_content: &[ContentBlock],
    deps: &Deps<'_>,
) -> Result<Vec<ContentBlock>, Error> {
    let step_dir_abs = conv_repo.join(step_dir_rel_str);
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
        tool_results.push(outcome_to_tool_result(id, &outcome));
    }
    Ok(tool_results)
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
