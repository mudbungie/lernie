//! The multi-tool: one model round trip fanned into N tool invocations
//! (ARCH §3.3 *The multi-tool*).
//!
//! `multi_tool` is the one tool whose "binary" is the step loop itself.
//! Its input is a list of **inner invocations** — the same `{name,
//! input}` shapes the individual tools declare — plus execution
//! metadata (`on_failure`). [`super::run_tool_calls`] intercepts the
//! name before the executor is entered, because everything the
//! multi-tool means lives on this side of the subprocess boundary: the
//! grant gate ([`super::refusal`]), the per-invocation diagnostic
//! record, and the projection policy all belong to the loop, and a
//! subprocess implementation would have to bypass every one of them.
//!
//! **No bypass:** each inner invocation passes through the same
//! controls as a top-level one — the [`super::refusal`] grant gate
//! first, then [`crate::prompt::tool::ToolExecutor::execute`] with the
//! same stop flag and the same `tool_output:` bounded-projection policy
//! (§3.3, bl-d5fa). Its diagnostic record lands under
//! `steps/<agent-id>/<NNN>/tools/<outer-id>-<k>/` like any other — the
//! inner id is derived from the envelope's wire id plus the 1-based
//! position, since the wire minted no id for it.
//!
//! **Serial by default; the envelope may assert otherwise.** Inner
//! invocations run strictly in list order, one at a time, unless
//! `execution` says `parallel`. The harness classifies nothing: only
//! the agent knows whether *these* invocations collide, so the
//! assertion is the caller's and is taken at face value (§3.3).
//!
//! Under `parallel` the envelope's one commit is unchanged — all N
//! invocations' writes land in the same `git add -A` sweep that always
//! swept them (§3.3 commit-per-side-effect), so there is no
//! per-invocation attribution to lose. Two hazards are the agent's to
//! avoid, deliberately unpoliced: concurrent writers to one path are
//! last-write-wins, and an inner `cd` races every sibling's cwd
//! resolution ([`crate::prompt::tool::spawn::Caller::resolve`] reads
//! the mark at spawn). Both are legal — unwise, not forbidden.
//!
//! `on_failure` is a *sequencing* policy and is not consulted under
//! `parallel`: every invocation has already started, so none can be
//! skipped. The tally reports each outcome.
//!
//! **Block-on-all, structurally.** All inner results return together as
//! this one envelope's single `tool_result`: the transcript commits one
//! entry per `tool_use` id (§2.3, §3.3) and every wire protocol brazen
//! encodes accepts exactly one result per id, so incremental delivery
//! has no wire home. The envelope's worktree side effects likewise ride
//! the one tool commit, together.
//!
//! **Depth 1.** An inner invocation naming `multi_tool` is declined —
//! nesting adds no expressive power to a flat list and would compound
//! the attribution scheme for nothing.

mod inner;
mod parallel;

use inner::{Ctx, Inner, run_inner};
use parallel::Fan;

use super::Resolved;
use crate::prompt::Deps;
use crate::prompt::Error;
use crate::prompt::tool::ToolOutcome;
use serde::Deserialize;
use serde_json::Value;
use std::path::Path;

/// The multi-tool's declared name — what the model spells in the
/// `tool_use` block and what `schemas/tools/multi_tool.json` titles.
pub(super) const NAME: &str = "multi_tool";

/// The envelope input, as `schemas/tools/multi_tool.json` declares it.
/// A shape this does not parse is declined in-band ([`malformed`]) —
/// the model reads the expected shape and re-emits.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Envelope {
    invocations: Vec<Invocation>,
    #[serde(default)]
    on_failure: OnFailure,
    #[serde(default)]
    execution: Execution,
}

/// Whether the inner invocations run one at a time or all at once —
/// the agent's own assertion about whether they collide. Serial is the
/// default because it is the safe reading of a list the model wrote in
/// an order.
#[derive(Deserialize, Default, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
enum Execution {
    /// List order, one at a time. A later entry sees every earlier
    /// entry's side effects.
    #[default]
    Serial,
    /// All at once. The agent asserts the entries do not collide; the
    /// harness verifies nothing and `on_failure` does not apply.
    Parallel,
}

/// One inner invocation: exactly the `{name, input}` a top-level
/// `tool_use` block carries. An omitted `input` is the empty object —
/// the general path with the field absent, matching a tool whose
/// schema requires nothing.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Invocation {
    name: String,
    #[serde(default = "empty_input")]
    input: Value,
}

fn empty_input() -> Value {
    Value::Object(serde_json::Map::new())
}

/// What a failed inner invocation (an `is_error` outcome or a decline)
/// does to the entries after it (the ball's execution metadata).
#[derive(Deserialize, Default, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
enum OnFailure {
    /// Skip every later entry, reporting each as skipped. The default:
    /// a serial list is usually a dependent sequence, and running step
    /// 3 after step 2 failed compounds the damage.
    #[default]
    Abort,
    /// Run every entry regardless and report each outcome — for
    /// independent fans (N reads, N messages).
    RunAll,
}

/// How the fan-out ended, mirroring [`super::run_tool_calls`]'s own two
/// non-error exits: an outcome to commit, or the §2.9 stop observed
/// mid-envelope (the loop ceases; nothing is committed).
#[derive(Debug)]
pub(super) enum Fanout {
    Outcome(ToolOutcome),
    Stopped,
}

/// One inner invocation's reported state, attributed in the aggregate
/// rendering. `ok`/`failed` restate the inner outcome's `is_error`;
/// `declined` never reached the executor (grant or depth); `skipped`
/// never ran because an earlier entry failed under `abort`.
const OK: &str = "ok";
const FAILED: &str = "failed";
const DECLINED: &str = "declined";
const SKIPPED: &str = "skipped";

/// One rendered line item of the aggregate result.
struct Entry {
    name: String,
    status: &'static str,
    text: String,
}

/// Drive the envelope: parse, gate and execute each inner invocation in
/// order, and aggregate one [`ToolOutcome`] for the outer `tool_use` id.
/// Harness-level faults propagate exactly as from a top-level
/// invocation ([`Error::ToolExec`], §2.10).
pub(super) fn fan_out(
    outer_id: &str,
    input: &Value,
    step_dir_abs: &Path,
    resolved: &Resolved<'_>,
    conv_repo: &Path,
    conv_id: &str,
    deps: &Deps<'_>,
) -> Result<Fanout, Error> {
    let envelope = match Envelope::deserialize(input) {
        Ok(envelope) => envelope,
        Err(err) => return Ok(Fanout::Outcome(malformed(&err))),
    };
    let ctx = Ctx {
        executor: deps.tool_executor,
        stop: deps.stop,
    };
    if envelope.execution == Execution::Parallel {
        let fan = Fan {
            outer_id,
            invocations: &envelope.invocations,
            step_dir_abs,
            conv_repo,
            conv_id,
        };
        return Ok(match parallel::run(&fan, resolved, ctx)? {
            Some(entries) => Fanout::Outcome(render(&entries)),
            None => Fanout::Stopped,
        });
    }
    let total = envelope.invocations.len();
    let mut entries: Vec<Entry> = Vec::with_capacity(total);
    let mut failed_at: Option<usize> = None;
    for (idx, inv) in envelope.invocations.iter().enumerate() {
        if envelope.on_failure == OnFailure::Abort
            && let Some(failed) = failed_at
        {
            entries.push(skipped(inv, failed, total));
            continue;
        }
        let inner = Inner {
            outer_id,
            k: idx + 1,
            inv,
            step_dir_abs,
            conv_repo,
            conv_id,
        };
        let entry = match run_inner(&inner, resolved, ctx)? {
            Some(entry) => entry,
            None => return Ok(Fanout::Stopped),
        };
        if entry.status != OK {
            failed_at = failed_at.or(Some(idx + 1));
        }
        entries.push(entry);
    }
    Ok(Fanout::Outcome(render(&entries)))
}

/// The entry for an invocation `abort` skipped: it never ran, so it has
/// no diagnostic record and no outcome — only the reason it was passed
/// over, naming the entry that failed.
fn skipped(inv: &Invocation, failed_at: usize, total: usize) -> Entry {
    Entry {
        name: inv.name.clone(),
        status: SKIPPED,
        text: format!(
            "not run: on_failure \"abort\" ended the envelope after \
             [{failed_at}/{total}] failed."
        ),
    }
}

/// The aggregate rendering: a first-line tally, then one attributed
/// section per inner invocation in list order. Plain text like the
/// result envelope (§3.3) — each section's body *is* that invocation's
/// envelope (or decline / skip reason), already bounded per-stream by
/// the executor, so nothing is double-encoded and nothing is re-cut.
/// `is_error` is true when any entry failed or was declined; skipped
/// entries follow only from such a failure and add no signal of their
/// own.
fn render(entries: &[Entry]) -> ToolOutcome {
    let total = entries.len();
    let ok = entries.iter().filter(|e| e.status == OK).count();
    let skip = entries.iter().filter(|e| e.status == SKIPPED).count();
    let failed = total - ok - skip;
    let mut out = format!("{total} invocations: {ok} ok, {failed} failed, {skip} skipped\n");
    for (idx, entry) in entries.iter().enumerate() {
        let (k, name, status, text) = (idx + 1, &entry.name, entry.status, &entry.text);
        out.push_str(&format!(
            "\n=== [{k}/{total}] {name}: {status} ===\n{text}\n"
        ));
    }
    ToolOutcome {
        content: out.into_bytes(),
        is_error: failed > 0,
    }
}

/// The in-band decline for an envelope [`Envelope::deserialize`] does
/// not parse: the model is told the expected shape and re-emits — the
/// same idiom as an unknown-skill or unknown-path decline (§3.3).
fn malformed(err: &serde_json::Error) -> ToolOutcome {
    ToolOutcome {
        content: format!(
            "{NAME}: malformed envelope: {err}. Expected \
             {{\"invocations\": [{{\"name\": \"<tool>\", \"input\": {{...}}}}, ...], \
             \"on_failure\": \"abort\"|\"run_all\"}}."
        )
        .into_bytes(),
        is_error: true,
    }
}
