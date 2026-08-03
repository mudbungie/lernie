//! One inner invocation of a multi-tool envelope, run through the
//! top-level gates (ARCH §3.3 *The multi-tool*, No bypass). Split from
//! [`super`] to hold the 300-line code-file cap.

use super::super::{Resolved, refusal, seam, stop_signal};
use super::{DECLINED, Entry, FAILED, Invocation, NAME, OK};
use crate::prompt::Deps;
use crate::prompt::Error;
use crate::prompt::tool::{ExecError, ToolCall};
use std::path::Path;

/// One inner invocation's coordinates — bundled so `run_inner` keeps a
/// readable arity across the gates it threads them through.
pub(super) struct Inner<'a> {
    pub(super) outer_id: &'a str,
    pub(super) k: usize,
    pub(super) inv: &'a Invocation,
    pub(super) step_dir_abs: &'a Path,
    pub(super) conv_repo: &'a Path,
    pub(super) conv_id: &'a str,
}

/// Run one inner invocation through the top-level controls: the depth
/// refusal, the grant gate, the tool-control seam (§3.3 *Tool control*
/// — No bypass: an envelope must not dodge what a top-level `tool_use`
/// meets), then the executor with the derived id `<outer-id>-<k>` (its
/// diagnostic record's directory name). `None` means the §2.9 stop was
/// observed — same reading as the top-level window: the executor's own
/// group SIGTERM with the stop flag set is the stop, not a fault.
///
/// A **hold cannot park mid-envelope**: entries before it have already
/// executed, and a park would make their uncommitted side effects
/// re-run on resume — exactly what the hold mark exists to rule out. So
/// an inner hold degrades to a decline that keeps the hold's one
/// guarantee (the invocation does not run unreviewed) and tells the
/// model to re-issue it as a top-level `tool_use`, where a hold parks
/// properly.
pub(super) fn run_inner(
    inner: &Inner<'_>,
    resolved: &Resolved<'_>,
    deps: &Deps<'_>,
) -> Result<Option<Entry>, Error> {
    let inv = inner.inv;
    if inv.name == NAME {
        return Ok(Some(Entry {
            name: inv.name.clone(),
            status: DECLINED,
            text: format!(
                "{NAME:?} may not contain itself (depth 1): \
                 list the nested invocations in this envelope directly."
            ),
        }));
    }
    if let Some(decline) = refusal(resolved.grant.role, resolved.grant.tools, &inv.name) {
        return Ok(Some(Entry {
            name: inv.name.clone(),
            status: DECLINED,
            text: decline,
        }));
    }
    let inner_id = format!("{}-{}", inner.outer_id, inner.k);
    match seam::adjudicate(
        resolved.workflow.tool_control.as_ref(),
        resolved.grant.role,
        &inner_id,
        &inv.name,
        &inv.input,
        inner.conv_repo,
        inner.conv_id,
        deps.stop,
    )? {
        seam::Gate::Stopped => return Ok(None),
        seam::Gate::Refuse(reason) => {
            return Ok(Some(Entry {
                name: inv.name.clone(),
                status: DECLINED,
                text: seam::refusal_text(&inv.name, &reason),
            }));
        }
        seam::Gate::Hold(reason) => {
            return Ok(Some(Entry {
                name: inv.name.clone(),
                status: DECLINED,
                text: format!(
                    "the workflow's tool control held {:?} ({reason}), and a hold cannot \
                     park mid-envelope: re-issue this invocation as a top-level tool_use \
                     to have it reviewed (ARCH §3.3 Tool control).",
                    inv.name
                ),
            }));
        }
        seam::Gate::Proceed => {}
    }
    match deps.tool_executor.execute(
        ToolCall {
            id: &inner_id,
            name: &inv.name,
            input: &inv.input,
        },
        inner.step_dir_abs,
        deps.stop,
        resolved.workflow.tool_output,
    ) {
        Ok(outcome) => Ok(Some(Entry {
            name: inv.name.clone(),
            status: if outcome.is_error { FAILED } else { OK },
            text: String::from_utf8_lossy(&outcome.content).into_owned(),
        })),
        Err(ExecError::KilledBySignal { .. }) if stop_signal::stopped(deps.stop) => Ok(None),
        Err(source) => Err(Error::ToolExec {
            tool: inv.name.clone(),
            source,
        }),
    }
}
