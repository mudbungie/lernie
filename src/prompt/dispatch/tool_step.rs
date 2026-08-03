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

use super::Resolved;
use super::stop_signal;
use super::transcript;
use crate::prompt::Deps;
use crate::prompt::Error;
use crate::prompt::compactor;
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
///
/// `resolved` carries the calling agent's role and its `providers.yaml`
/// `tools:` grant (§4.3) — the pair travels from the one resolution that
/// reads both ([`crate::prompt::resolve`]), so a role and a grant that
/// do not belong together cannot reach here. Its workflow also supplies
/// the `tool_output:` bounded-projection policy (§3.3, §6), handed to
/// every `execute` so the executor caps the streams it renders. They gate what may be
/// *called*, which the request's declaration does not imply: a request
/// declares every tool its history names so the wire holds
/// ([`super::tools::close_over_history`]), and an inherited transcript
/// names whatever tools the dispatching branch used. A role reaching for
/// a tool outside its effective toolset ([`refusal`]) is declined in-band
/// — an `is_error` `tool_result` committed like any other, so the model
/// reads the decline and steps on — and the executor is never entered.
pub(super) fn run_tool_calls(
    conv_repo: &Path,
    worktree: &Path,
    conv_id: &str,
    resolved: &Resolved<'_>,
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
        let outcome = match refusal(resolved.grant.role, resolved.grant.tools, name) {
            Some(decline) => ToolOutcome {
                content: decline.into_bytes(),
                is_error: true,
            },
            None => match deps.tool_executor.execute(
                ToolUse { id, name, input },
                &step_dir_abs,
                deps.stop,
                resolved.workflow.tool_output,
            ) {
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
            },
        };
        let tool_result = outcome_to_tool_result(id, &outcome);
        transcript::commit_tool(worktree, conv_id, &tool_result, deps.git)?;
    }
    Ok(false)
}

/// Why `role` may not call `tool`, or `None` when it may (ARCH §3.3
/// *declaring is not permitting*, §4.3 *Toolset*).
///
/// A role's **effective toolset** is its `providers.yaml` `tools:` grant
/// plus whatever its procedure injects ([`compactor::injected`] — the
/// compactor's pair, empty for every other role). Its request declares
/// more than that and must: the array is closed over the history it
/// ships (§3.3), and a branch inherits its dispatcher's transcript by
/// fork (§2.3), so the tools that dispatcher used are named in the
/// history whether or not this role was granted them.
///
/// Permitting does not follow from declaring. If it did, a grant would
/// widen itself the moment a dispatcher used a tool the child was
/// denied — voiding exactly the boundaries a grant exists to draw: a
/// read-only observer on an outward surface (§4.3) forked from a
/// dispatcher that speaks there, or the compactor's deletion-only
/// guarantee (§2.7). So the decline is in-band: an `is_error`
/// `tool_result` naming the role's own toolset, which the model reads
/// and steps on from, and the executor is never entered.
fn refusal(role: &str, grant: &[String], tool: &str) -> Option<String> {
    let injected = compactor::injected(role);
    if grant.iter().any(|granted| granted == tool) || injected.contains(&tool) {
        return None;
    }
    let mut effective: Vec<&str> = grant.iter().map(String::as_str).collect();
    effective.extend(injected);
    let toolset = if effective.is_empty() {
        "empty".to_string()
    } else {
        effective.join(", ")
    };
    Some(format!(
        "{tool:?} is not callable by a {role}: it is declared only because \
         the inherited transcript references it. The {role} toolset is \
         {toolset} (ARCH §3.3, declaring is not permitting)."
    ))
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
