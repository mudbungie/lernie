//! Per-step tool-call orchestration (ARCH §2.5, §3.3).
//!
//! When a step's completion carries `tool_use` blocks, the loop hands
//! each one to [`crate::prompt::ToolExecutor`] in emission order. The
//! executor lands `input.json` and `output.json` under
//! `<conv-repo>/steps/<conv-id>/<NNN>/tools/<tool-id>/` — outside every
//! worktree (§2.2 / §2.3), a diagnostic record that is *not* a commit.
//!
//! As each tool resolves, its canonical `tool_result` block *is*
//! committed — `messages/NNN-tool.json`, the transcript entry the next
//! step's request composes from (§2.3, §3.3 "Wire `tool_result` framing
//! is transcript-backed"). Nothing is returned to the loop: the next
//! step re-assembles its whole history from the tree (§5), so a
//! `tool_result` has exactly one home, the committed entry. The per-call
//! `output.json` stays the raw audit capture, written but never read at
//! runtime (§2.3 Diagnostic-only contract) — two facts, not two copies.
//! The sequential loop *is* the sibling-tool serialization §3.3
//! requires, and the counter read (`next_seq`) rides inside it.
//!
//! Living in a sibling module keeps `super`'s `run_exchange` body under
//! the repo's 300-line code-file cap.

#[cfg(test)]
mod tests;

use super::stop_signal;
use super::transcript;
use crate::prompt::Deps;
use crate::prompt::Error;
use crate::prompt::tool::{ExecError, ToolCall as ToolUse, ToolOutcome};
use brazen::Content;
use std::path::Path;

/// Drive every `tool_use` block in `assistant_content` through the
/// executor in emission order, committing each result as a transcript
/// entry (§2.3, §4.4 `Content::ToolResult`). The next step's request is
/// re-assembled from the tree (§5), so nothing flows back through the
/// loop.
///
/// The executor's SIGTERM flag ([`Deps::stop`], §2.9 step 3) is the stop
/// signal handed to each tool, so a `lernie stop` landing in a
/// tool-execution window is the *same* terminal sequence as one landing in
/// a model-call window: the tool subprocesses are the executor's limbs and
/// take the group SIGTERM (§2.9 steps 1-2). A tool cut down that way
/// returns [`ExecError::KilledBySignal`]; with the stop flag set that is
/// the stop, not a harness fault — this returns `Ok(true)` so
/// [`super::run_exchange`] ceases the loop for the clean stopped-deposit
/// exit ([`super::terminal::finish`]), never an error propagation. A
/// `KilledBySignal` with *no* stop pending is a genuine crash (SIGSEGV, …)
/// and still surfaces as [`Error::ToolExec`] (§2.10). `Ok(false)` means
/// every tool resolved and the loop continues.
pub(super) fn run_tool_calls(
    conv_repo: &Path,
    worktree: &Path,
    conv_id: &str,
    step_dir_rel_str: &str,
    assistant_content: &[Content],
    deps: &Deps<'_>,
) -> Result<bool, Error> {
    let step_dir_abs = conv_repo.join(step_dir_rel_str);
    for block in assistant_content {
        let Content::ToolUse {
            id, name, input, ..
        } = block
        else {
            continue;
        };
        let outcome =
            match deps
                .tool_executor
                .execute(ToolUse { id, name, input }, &step_dir_abs, deps.stop)
            {
                Ok(outcome) => outcome,
                // §2.9 step 3: a tool group-killed by the executor's own
                // SIGTERM, with the stop flag set, is the stop — cease the loop
                // for the stopped-deposit exit, not an error.
                Err(ExecError::KilledBySignal { .. }) if stop_signal::stopped(deps.stop) => {
                    return Ok(true);
                }
                Err(source) => {
                    return Err(Error::ToolExec {
                        tool: name.clone(),
                        source,
                    });
                }
            };
        let tool_result = outcome_to_tool_result(id, &outcome);
        transcript::commit_tool(worktree, conv_id, &tool_result, deps.git)?;
    }
    Ok(false)
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
