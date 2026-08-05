//! Settling a tool window the §2.9 stop cascade felled.
//!
//! A stop landing in a **tool window** finds the step's model-output
//! entry already committed (§2.5 — the assistant entry lands before any
//! tool runs) while some of its `tool_use` blocks have no committed
//! `tool_result`. Left that way the branch tip is the §6 *one
//! non-replayable state*: `lernie advance` declines it loudly
//! (`Error::UnpairedToolUse`), so no deposit could ever revive the agent
//! — the stop would retire the branch instead of ending the work it
//! had in flight, contradicting §2.9 ("a stop is not a locked door … a message
//! into the stopped agent's inbox starts a driver and resumes the same
//! branch").
//!
//! So the stopped exit **settles its own window before it deposits**:
//! one in-band `is_error` `tool_result` per unanswered `tool_use` id —
//! the same shape a grant decline and a control refusal already commit
//! ([`super::refusal`], [`super::seam::refusal_text`]) — saying the
//! invocation was cut short. The tail is then settled, the warrant is
//! `ModelCallDue`, an ordinary deposit revives the agent, and the model
//! reads *in band* that it was interrupted, which is both the truthful
//! record and the useful one.
//!
//! Deleting the tail — what the dispatch commit does at a **fork**
//! ([`super::super::step_commit::unsettled`], §2.3 step 2) — is the
//! wrong repair here: that tail belongs to the agent's *own* branch,
//! where discarding it would throw away the assistant's reasoning and
//! leave the model with no evidence it was ever cut off.
//!
//! A **hold** is deliberately not settled (§3.3 *Tool control*): a
//! parked branch's unpaired tail is its state, and its mark asserts
//! nothing at or past the held block ran. Only the stop settles.

use super::super::transcript;
use super::ToolWindow;
use crate::prompt::{Deps, Error};
use brazen::Content;
use std::path::Path;

/// Commit an interrupted `tool_result` for every `tool_use` in
/// `assistant_content` still unanswered, and report the window stopped.
///
/// Idempotent by construction: the answered ids are read from the
/// transcript (the record, never a stored cursor — PRINCIPLES single
/// source of truth), so results committed before the stop keep the one
/// entry they already have.
pub(super) fn interrupted(
    worktree: &Path,
    conv_id: &str,
    assistant_content: &[Content],
    deps: &Deps<'_>,
) -> Result<ToolWindow, Error> {
    let committed = transcript::committed_result_ids(worktree)?;
    for block in assistant_content {
        let Content::ToolUse { id, name, .. } = block else {
            continue;
        };
        if committed.contains(id) {
            continue;
        }
        let tool_result = Content::ToolResult {
            tool_use_id: id.clone(),
            content: vec![Content::Text(text(name))],
            is_error: true,
        };
        transcript::commit_tool(worktree, conv_id, &tool_result, deps.git)?;
    }
    Ok(ToolWindow::Stopped)
}

/// The in-band text an unanswered invocation carries as its `is_error`
/// `tool_result` — why there is no output, in the terms §2.9 gives it.
/// No result envelope and no exit code: nothing returned, so none is
/// invented (§3.3, the [`super::seam::refusal_text`] discipline).
fn text(tool: &str) -> String {
    format!(
        "{tool:?} did not return: this agent was stopped while the invocation was in \
         flight (ARCH §2.9), so it was cut short and produced no result. Any side \
         effects it had already performed stand."
    )
}
