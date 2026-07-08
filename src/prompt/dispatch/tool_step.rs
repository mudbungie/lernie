//! Per-step tool-call orchestration (ARCH §2.5, §3.3).
//!
//! When a step's completion carries `tool_use` blocks, the loop hands
//! each one to [`crate::prompt::ToolExecutor`] in emission order. The
//! executor lands `input.json` and `output.json` under
//! `<conv-repo>/steps/<conv-id>/<NNN>/tools/<tool-id>/` — outside every
//! worktree (§2.2 / §2.3). The per-tool diagnostic record is *not* a
//! commit (§3.3); only worktree-modifying tools commit, inside their own
//! subprocess contract, not here.
//!
//! Living in a sibling module keeps `super`'s `run_exchange` body under
//! the repo's 300-line code-file cap.

use crate::prompt::Deps;
use crate::prompt::Error;
use crate::prompt::tool::{ToolCall as ToolUse, ToolOutcome};
use brazen::Content;
use std::path::Path;
use std::sync::atomic::AtomicBool;

/// Drive every `tool_use` block in `assistant_content` through the
/// executor. Returns the `tool_result` blocks in emission order (§4.4
/// `Content::ToolResult`).
pub(super) fn run_tool_calls(
    conv_repo: &Path,
    step_dir_rel_str: &str,
    assistant_content: &[Content],
    deps: &Deps<'_>,
) -> Result<Vec<Content>, Error> {
    let step_dir_abs = conv_repo.join(step_dir_rel_str);
    let stop_flag = AtomicBool::new(false);
    let mut tool_results: Vec<Content> = Vec::new();
    for block in assistant_content {
        let Content::ToolUse { id, name, input } = block else {
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

/// Turn the executor's [`ToolOutcome`] into the canonical `ToolResult`
/// block the next step's user message carries (ARCH §3.3). Stdout bytes
/// round-trip through lossy UTF-8 — the harness wraps a tool's stdout as
/// a single `Content::Text` per §3.3.
fn outcome_to_tool_result(tool_use_id: &str, outcome: &ToolOutcome) -> Content {
    Content::ToolResult {
        tool_use_id: tool_use_id.to_string(),
        content: vec![Content::Text(
            String::from_utf8_lossy(&outcome.content).into_owned(),
        )],
        is_error: outcome.is_error,
    }
}
