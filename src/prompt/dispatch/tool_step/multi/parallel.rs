//! The `parallel` execution mode of a multi-tool envelope (ARCH §3.3
//! *The multi-tool*): every inner invocation runs at once, on the
//! agent's own assertion that they do not collide.
//!
//! The harness classifies nothing here — no read/write table, no
//! per-tool effect fact. Only the caller knows whether *these*
//! invocations touch the same paths, so the claim is the caller's and
//! is taken at face value. What the harness still guarantees is
//! unchanged: every entry passes the same depth, grant, and
//! tool-control gates ([`super::inner::gate`]), and all N writes land
//! in the envelope's one `git add -A` commit exactly as they did
//! serially — the envelope never committed per-invocation, so
//! concurrency costs no attribution.
//!
//! **Gate-then-execute.** Every entry is gated first, on this thread,
//! and only the survivors are handed to
//! [`crate::prompt::tool::ToolExecutor::execute_all`] together. The
//! gates are cheap and the loop's own — the grant read, the depth
//! check, the tool-control seam — while the executor owns the part
//! worth overlapping: N blocking subprocess waits. Clearing every gate
//! before starting any invocation also means a refusal cannot land
//! after a sibling it was meant to precede.
//!
//! **Concurrency lives in the executor, not here.** The step loop
//! never shares its dependencies across threads; if it did, `Sync`
//! would have to spread to the clock, the git runner and the PATH
//! lookup — three traits with nothing to do with threading
//! (PRINCIPLES, severability). An executor that declines to overlap
//! (every in-process stub) inherits `execute_all`'s serial default and
//! is still correct: concurrency is an optimization, not a semantic.

use super::super::Resolved;
use super::inner::{Ctx, Gated, Inner, finish, gate};
use super::{Entry, Invocation};
use crate::prompt::Error;
use crate::prompt::tool::ToolCall;
use std::path::Path;

/// The envelope-wide coordinates every inner invocation shares.
pub(super) struct Fan<'a> {
    pub(super) outer_id: &'a str,
    pub(super) invocations: &'a [Invocation],
    pub(super) step_dir_abs: &'a Path,
    pub(super) conv_repo: &'a Path,
    pub(super) conv_id: &'a str,
}

/// Gate every entry, run the survivors together, and return the
/// entries in **list order** — never completion order, so the
/// rendering is deterministic whatever the scheduler did.
///
/// `Ok(None)` means the §2.9 stop was observed, the same reading as
/// the serial path: the loop ceases and nothing is committed.
pub(super) fn run(
    fan: &Fan<'_>,
    resolved: &Resolved<'_>,
    ctx: Ctx<'_>,
) -> Result<Option<Vec<Entry>>, Error> {
    let mut gated = Vec::with_capacity(fan.invocations.len());
    for (idx, inv) in fan.invocations.iter().enumerate() {
        let inner = Inner {
            outer_id: fan.outer_id,
            k: idx + 1,
            inv,
            step_dir_abs: fan.step_dir_abs,
            conv_repo: fan.conv_repo,
            conv_id: fan.conv_id,
        };
        match gate(&inner, resolved, ctx)? {
            Some(decision) => gated.push(decision),
            None => return Ok(None),
        }
    }

    // `calls` borrows the ids out of `gated`; the borrow ends with this
    // block, so the weave below can consume `gated` by value.
    let results = {
        let calls: Vec<ToolCall<'_>> = gated
            .iter()
            .zip(fan.invocations)
            .filter_map(|(decision, inv)| match decision {
                Gated::Ready(id) => Some(ToolCall {
                    id,
                    name: &inv.name,
                    input: &inv.input,
                }),
                Gated::Declined(_) => None,
            })
            .collect();
        ctx.executor.execute_all(
            &calls,
            fan.step_dir_abs,
            ctx.stop,
            resolved.workflow.tool_output,
        )
    };

    let mut results = results.into_iter();
    let mut entries = Vec::with_capacity(gated.len());
    for (decision, inv) in gated.into_iter().zip(fan.invocations) {
        let entry = match decision {
            Gated::Declined(entry) => entry,
            Gated::Ready(_) => {
                let result = results
                    .next()
                    .expect("one result per tool call handed over");
                match finish(&inv.name, result, ctx.stop)? {
                    Some(entry) => entry,
                    None => return Ok(None),
                }
            }
        };
        entries.push(entry);
    }
    Ok(Some(entries))
}
