//! `lernie advance <workspace> <agent>` — the §6 driver verb: one hop
//! of the workflow chain.
//!
//! Every §2.11 launch seam spawns this verb (detached, via the injected
//! launcher), the exec baton hands one hop to the next (§6), and an
//! operator runs the same verb by hand — launch after a crash, launch
//! after a deposit, and hand-run are indistinguishable, which is the
//! §6 collapse of "advance" and "resume". A hop:
//!
//! 1. **takes the lease** — an adopted predecessor fd or a fresh
//!    acquire ([`crate::prompt::inbox::baton`]); losing the acquire is
//!    the clean no-op of Writer/driver totality (§2.11).
//! 2. **delivers** — [`super::driver::deliver`]: rematerialize, stray
//!    recovery, delivery commits (§2.11).
//! 3. **derives warrant from the tree** ([`warrant`]) — no launcher
//!    decides warrant; the driver decides under the lock (§2.11). The
//!    derivation is the wire alternation itself (§2.3): a transcript
//!    tail ending user-side means a model call is due.
//! 4. **runs one step** ([`hop`]) — the same step machinery
//!    [`super::run_exchange`] drives.
//! 5. **hands off** — tools ran → [`AdvanceOutcome::ToolsPending`]
//!    carries the live lease out for the caller to exec the successor
//!    (§6 exec baton, [`cli`]); a terminal event ends the chain through
//!    the §2.11 exit protocol (deposit at the terminal event → release →
//!    epitaph-valued launch → return).
//!
//! Config resolution is **lazy** (the `resolve` closure): a no-op hop
//! exits before any config file is read or any `bz --version` guard
//! runs, so the pin-1 recursion terminator costs nothing but the probe.

pub mod cli;
mod hop;

#[cfg(test)]
mod tests;

use super::{assembler, driver, terminal};
use crate::prompt::inbox::{self, Epitaph, ExecutorLock};
use crate::prompt::resolve::WorkerConfig;
use crate::prompt::{Deps, Error};
use brazen::{Content, Message, Role};
use std::path::Path;

/// What one hop found and did — derived on the fly, nothing stored.
#[derive(Debug)]
pub enum AdvanceOutcome {
    /// Another executor holds the lock: the branch is already driven —
    /// the clean no-op of Writer/driver totality (§2.11).
    AlreadyDriven,
    /// Nothing is due: empty inbox and a transcript tail with no model
    /// call pending — the §2.11 pin-1 silent exit, terminating the
    /// exit-launch recursion.
    NothingToDo,
    /// The hop stepped to a terminal event and ran the §2.11 exit
    /// protocol; the chain ends here.
    Terminal(Epitaph),
    /// The step emitted `tool_use` and its tools ran: the successor hop
    /// must run. Carries the live lease for the §6 exec baton — the
    /// caller preps the successor command ([`cli`], `baton`) and execs.
    ToolsPending(ExecutorLock),
}

/// What the transcript tail warrants (§6 hop step 3).
#[derive(Debug, PartialEq, Eq)]
enum Warrant {
    /// Tail ends user-side (delivered mail, committed tool results): a
    /// model call is due.
    ModelCallDue,
    /// Tail ends assistant-side without `tool_use`, or is empty: nothing
    /// is due (§2.11 pin 1).
    NothingDue,
    /// Tail ends assistant-side with `tool_use` unmatched by committed
    /// tool results: the one non-replayable state (§6), declined loudly.
    Unpaired,
}

/// Derive warrant from the assembled wire history (§6): the alternation
/// grouping (§2.3) makes the tail's role the whole answer — committed
/// tool results compose user-side, so "all tools resolved" and "mail
/// delivered" are the same observation.
fn warrant(messages: &[Message]) -> Warrant {
    match messages.last() {
        None => Warrant::NothingDue,
        Some(m) if m.role == Role::User => Warrant::ModelCallDue,
        Some(m)
            if m.content
                .iter()
                .any(|b| matches!(b, Content::ToolUse { .. })) =>
        {
            Warrant::Unpaired
        }
        Some(_) => Warrant::NothingDue,
    }
}

/// Run one hop against `agent_id`'s branch. `lease` is a lease the
/// caller already took (the adopted §6 baton fd); `None` acquires here
/// — losing the acquire is [`AdvanceOutcome::AlreadyDriven`]. `resolve`
/// loads the role config lazily, only once a step is warranted (`&mut
/// dyn` rather than `impl FnOnce` so the function has one instantiation
/// and one coverage record).
pub(in crate::prompt) fn run(
    workspace: &Path,
    agent_id: &str,
    lease: Option<ExecutorLock>,
    deps: &Deps<'_>,
    resolve: &mut dyn FnMut() -> Result<WorkerConfig, Error>,
) -> Result<AdvanceOutcome, Error> {
    let lock = match lease {
        Some(lock) => lock,
        None => {
            let inbox_dir = inbox::inbox_dir(workspace, agent_id);
            match inbox::try_acquire(&inbox_dir).map_err(|source| Error::ExecutorLock {
                path: inbox_dir.clone(),
                source,
            })? {
                Some(lock) => lock,
                None => return Ok(AdvanceOutcome::AlreadyDriven),
            }
        }
    };

    driver::deliver(workspace, agent_id, deps.git)?;
    let worktree = crate::workspace::agent_worktree(workspace, agent_id);
    if !worktree.exists() {
        // Torn down and no mail: quiescent, nothing due (§2.3 step 6).
        return Ok(AdvanceOutcome::NothingToDo);
    }

    match warrant(&assembler::assemble(&worktree)?) {
        Warrant::NothingDue => Ok(AdvanceOutcome::NothingToDo),
        Warrant::Unpaired => Err(Error::UnpairedToolUse {
            branch: agent_id.to_string(),
        }),
        Warrant::ModelCallDue => {
            let cfg = resolve()?;
            match hop::step(workspace, agent_id, &worktree, &cfg, deps)? {
                hop::StepOutcome::ToolsRan => Ok(AdvanceOutcome::ToolsPending(lock)),
                hop::StepOutcome::Terminal(epitaph) => {
                    // Terminal handling + exit protocol, exactly as
                    // `run_exchange`'s tail (§2.11): finish by epitaph
                    // value, release own lock, then the self-directed
                    // launch — after release this process has no
                    // authority; spawn and return are its only acts.
                    terminal::finish(workspace, agent_id, agent_id, &worktree, epitaph, deps)?;
                    drop(lock);
                    terminal::exit_launch(workspace, agent_id, epitaph, deps);
                    Ok(AdvanceOutcome::Terminal(epitaph))
                }
            }
        }
    }
}
