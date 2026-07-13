//! User actions issued through `cli_outbound` (ARCH §3.4 / §3.5).
//!
//! Two buttons live on the action surface. **New prompt** invokes
//! `lernie prompt <repo> <message>`; **Stop** invokes
//! `lernie stop <repo> <agent-id>` against the selected live agent.
//! The CLI shapes are pinned by `src/bin/lernie.rs` (`Command::Prompt`
//! and `Command::Stop`, the latter introduced by bl-a144). Per the §2.9
//! amendment (bl-abf3) there is no user-facing "resume": continuing
//! a stopped agent is `lernie message` (the deposit starts a driver,
//! §2.11) or fork-from-history.
//!
//! The view-model is a pure function of inputs and carries no egui
//! dependency, so `lernie-ui-web` (or any other future frontend) can
//! reuse the derivation and dispatch helpers unchanged. Per §3.5 the UI
//! holds no persistent state — `ActionsState` is in-memory only and is
//! discarded on UI exit.

use std::path::Path;

use crate::cli_outbound::{Cli, CliError, Stream};
use crate::git_tree::{Agent, AgentState};

/// `lernie` subcommand for sending a user message on a fresh root
/// branch. Pinned to `src/bin/lernie.rs` `Command::Prompt`.
const SUBCOMMAND_PROMPT: &str = "prompt";

/// `lernie` subcommand for stopping a live agent (ARCH §2.9 cascading
/// SIGTERM, bl-a144). Pinned to `src/bin/lernie.rs` `Command::Stop`.
const SUBCOMMAND_STOP: &str = "stop";

/// Ephemeral action-surface state. Held in memory by the running
/// frontend and discarded on exit (ARCH §3.5: frontends hold no
/// persistent state).
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct ActionsState {
    /// In-progress text for the New-prompt input. The button is
    /// disabled while this is empty (or whitespace-only).
    pub new_prompt_input: String,
    /// User-selected agent id (§2.3 — the id is the address; `lernie
    /// stop` takes it, not the `agents/*` ref). The Stop button is
    /// disabled while this is `None` or while the selected agent is no
    /// longer live (e.g. it went quiescent or stopped between selection
    /// and click).
    pub selected_branch: Option<String>,
}

/// True iff the new-prompt input has at least one non-whitespace
/// character. Pure derivation per §3.5; no I/O.
pub fn new_prompt_enabled(input: &str) -> bool {
    !input.trim().is_empty()
}

/// True iff `selected_branch` names an agent (by id, §2.3) in `agents`
/// that is **live** — [`AgentState::Live`] or [`AgentState::InFlight`],
/// the two states where a driver holds the executor lock (§2.11). Stop
/// targets a live executor (§2.9), and it is wanted precisely during tool
/// execution (a `Live` agent between model calls), not only mid-model-call
/// — so both live states are stoppable. Returns `false` for `None`, for an
/// id not present, and for a `Quiescent` or `Stopped` agent (no executor
/// to signal).
pub fn stop_enabled(selected_branch: Option<&str>, agents: &[Agent]) -> bool {
    let Some(name) = selected_branch else {
        return false;
    };
    agents
        .iter()
        .any(|a| a.agent_id == name && matches!(a.state, AgentState::Live | AgentState::InFlight))
}

/// Spawn `lernie prompt <repo> <message>`. Caller owns the returned
/// [`Stream`]: dropping it before the harness exits sends SIGTERM
/// (ARCH §2.9 cascade), so the architectural intent is detach-and-drain
/// — spawn a thread that consumes the stream until natural exit so the
/// harness lives independently of the UI (§1 #4 Regenerability).
pub fn dispatch_new_prompt(cli: &Cli, repo: &Path, message: &str) -> Result<Stream, CliError> {
    let repo = repo.display().to_string();
    cli.run(&[SUBCOMMAND_PROMPT, &repo, message])
}

/// Spawn `lernie stop <workspace> <agent-id>`. The harness performs
/// the §2.9 SIGTERM cascade against the executor pgid; this is the UI
/// view of that same operation. `branch` is the agent id (§2.3).
pub fn dispatch_stop(cli: &Cli, repo: &Path, branch: &str) -> Result<Stream, CliError> {
    let repo = repo.display().to_string();
    cli.run(&[SUBCOMMAND_STOP, &repo, branch])
}

#[cfg(test)]
mod tests;
