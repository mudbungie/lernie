//! Child dispatch — fork plus front door, never a spawn (ARCH §2.5).
//!
//! A tool-call **dispatch** (§2.5) starts a child agent with a goal. The
//! primitive is writes and one deposit, with no process supervision
//! anywhere in it. [`run`] does three things, inline and synchronously:
//!
//! 1. **fork** the child branch off the parent's tip and land the
//!    dispatch commit (§2.3 step 2) — `goal.md` + `soul.md` pinned, and
//!    the tree trimmed to the child's context: the config's control
//!    files leave (§2.2) and so do the `descriptions/**` descriptors the
//!    child's role does not grant (§3.3, §5.1). This is
//!    [`super::subagent::spawn_subagent_branch`], shared with the
//!    compactor (§2.7).
//! 2. **deposit** the dispatch message into the new agent's inbox
//!    through the front door (§2.11): `deposit` then `probe_and_launch`,
//!    exactly what `lernie message` does. The probe finds the fresh
//!    child quiescent and launches its driver — `lernie advance` (§6),
//!    the ordinary driver every agent runs under. There is no
//!    child-specific loop and no worker path; the step loop never
//!    branches on parent/child.
//! 3. **return** the child's id — its address (§2.11) — to the caller,
//!    which the `dispatch` built-in re-emits as the `tool_result`.
//!
//! The child reports back with no executor logic keyed on being a child:
//! at its terminal event `advance` deposits its result message into the
//! **return address**, which for the shipped default is derived from the
//! child's id (`inbox::parent_of` — the dispatcher's address, §2.6). A
//! root records no dispatch and has no such address, so it sends nothing
//! and answers the user instead (§2.4). Return totality (§2.3 step 5) is
//! thereby a property of the dispatch primitive — a dispatch cannot fork
//! without an inbox to deposit into — rather than of loop code.
//!
//! **One budget gate, every dispatch** (§6). `budgets:` is a ceiling on
//! the conversation tree, and a dispatch is the only act that can deepen
//! it — so the check belongs at the fork, not at the child's first model
//! call, and there is exactly one fork in the system: [`run`]. Every
//! caller reaches it — the `dispatch` built-in and `lernie dispatch`
//! (model-initiated, §3.4) and the §6 workflow bindings
//! `worker_flush → dispatch(compactor)` and the verifier gate
//! (harness-initiated) — so `max_depth` cannot be enforced against one
//! kind of dispatch and not the other. The two kinds differ only in what
//! a refusal *means*, which is [`run_procedure`]'s single job.
//!
//! Without this, a `max_depth` breach was caught only when the child
//! finally tried to step (`budget::check` at the model-call boundary),
//! by which time the branch, its worktree and its inbox already existed —
//! the shape of the runaway compaction cascade in bl-a9eb (yog bl-ebbd),
//! where hundreds of branches were minted below a declared `max_depth: 4`.
//!
//! The goal is one input with two projections, both written at dispatch:
//! `goal.md` (pinned standing context, §2.8) and the deposited dispatch
//! message (the on-ramp the child's step-1 drain delivers). They carry
//! the same text by construction and neither is ever rewritten, so no
//! second fact can drift (`docs/PRINCIPLES.md` Single source of truth) —
//! the same shape the root on-ramp uses (`dispatch::run_exchange`).

use super::clock::{Clock, IdGen};
use super::subagent::{SpawnRequest, spawn_subagent_branch};
use super::{Error, PER_REPO_PROVIDERS_FILE, SOULS_DIR, WORKFLOW_FILE};
use crate::config::{PerRepoProviders, Workflow};
use crate::prompt::budget;
use crate::prompt::inbox::{self, Launcher};
use crate::template::GitRunner;
use crate::workspace;
use std::path::{Path, PathBuf};

/// Inputs to a child dispatch. Built the same way by the dispatch
/// built-in and the `lernie dispatch` CLI regardless of the target role.
pub struct ChildDispatchRequest<'a> {
    /// Workspace repository root. Used for the child's worktree path
    /// (sibling under `agents/`, §2.2), for soul resolution
    /// (`souls/worker.md` in the governing config commit), and as the
    /// deposit target's workspace.
    pub repo: &'a Path,
    /// Dispatching branch name — the parent's full hyphenated descent
    /// (§2.3). The child forks off this branch's tip, its id is
    /// `<parent>-<sub-id>`, and it is the deposit's sender (§2.11
    /// provenance) and the derived return address (§2.6).
    pub parent_branch: &'a str,
    /// The dispatching branch's worktree — where `git worktree add` runs
    /// (any access point onto the one workspace repository, §2.2).
    pub parent_worktree: &'a Path,
    /// The child's role (§2.5, §4.3): selects the pinned soul
    /// (`souls/<role>.md` in the governing config commit) and labels the
    /// dispatch commit. `worker` for an ordinary child, `compactor` for a
    /// compaction dispatch (§2.7) — parent/child is provenance, the role
    /// is what the child *is*.
    pub role: &'a str,
    /// The goal / dispatch message. Written verbatim to the child's
    /// `goal.md` and deposited as its first inbox message.
    pub goal: &'a str,
    /// The ref the child forks off (ARCH §2.3). `None` is the ordinary
    /// child dispatch off the parent's tip (§2.5); `Some(ref)` forks off
    /// another ref — a verifier off the worker's terminal ref (§6 gate) —
    /// while the child id stays `<parent>-<sub>` (return address
    /// unchanged).
    pub fork_point: Option<&'a str>,
}

/// Fork a child agent off `req.parent_branch`'s tip and start it through
/// the front door. Returns the child's id (`<parent>-<sub-id>`) — its
/// branch name and its address (§2.3, §2.11). `launcher` is injected so
/// the post-deposit driver launch is testable without spawning a real
/// `lernie advance`; production passes [`inbox::AdvanceLauncher`].
///
/// **The §6 budget gate lives here**, and only here (module docs): the
/// declared `budgets:` are evaluated against the child's own prospective
/// branch name *before* the fork, so a dispatch that would breach
/// `max_depth` — or start work under a tree that has already spent its
/// tokens or wall — is refused with [`Error::DispatchRefused`] and leaves
/// no branch, no worktree and no inbox behind. Depth is positional and
/// derives from the branch name alone (`budget::derive::depth`), so the
/// check is exact for a branch that does not yet exist.
pub fn run(
    req: &ChildDispatchRequest<'_>,
    git: &dyn GitRunner,
    clock: &dyn Clock,
    id_gen: &dyn IdGen,
    launcher: &dyn Launcher,
) -> Result<String, Error> {
    let sub_id = format!("{}-{}", clock.now_compact(), id_gen.short());
    // Hyphenated descent (§2.3): the child's id and worktree share the
    // `<parent>-<sub-id>` name; the `agents/` ref prefix is applied at
    // the git boundary by `spawn_subagent_branch`.
    let sub_branch = format!("{}-{sub_id}", req.parent_branch);
    let sub_worktree = workspace::agent_worktree(req.repo, &sub_branch);

    // Soul resolution (§2.2 / §4.3): read `souls/worker.md` from the
    // dispatching branch's governing config commit, verbatim — the
    // immutable commit is the load-bearing artifact, never a worktree
    // file. A child inherits the parent's config through ancestry.
    let commit =
        workspace::governing_config(req.repo, req.parent_branch, git).map_err(|source| {
            Error::Git {
                op: "governing config",
                source,
            }
        })?;
    // §6 budget gate — the one enforcement point for *every* dispatch
    // (see [`run`]'s docs). Evaluated against the child's own prospective
    // branch name, so `max_depth` refuses the fork that would breach it
    // rather than letting the branch exist and decline later.
    if let Some(ex) = budget::check(req.repo, &sub_branch, &budgets(req, &commit, git)?) {
        return Err(Error::DispatchRefused {
            child: sub_branch,
            parent: req.parent_branch.to_string(),
            exhausted: ex,
        });
    }

    let soul_rel = format!("{SOULS_DIR}/{}.md", req.role);
    let soul = workspace::show_control(req.repo, &commit, &soul_rel, git).map_err(|source| {
        Error::ControlRead {
            path: PathBuf::from(format!("{commit}:{soul_rel}")),
            source,
        }
    })?;

    // The child's `tools:` grant (§4.3), read from the same governing
    // config commit as the soul — one commit, one answer, no second copy
    // of the grant anywhere. It is what the dispatch commit prunes the
    // inherited `descriptions/**` snapshot to (§3.3, §5.1), so the
    // child's tree documents exactly what its requests will declare. A
    // role the config lists without a `tools:` list grants none, which is
    // §4.3's own reading of an omitted list and the compactor's shape.
    let providers_raw = workspace::show_control(req.repo, &commit, PER_REPO_PROVIDERS_FILE, git)
        .map_err(|source| Error::ControlRead {
            path: PathBuf::from(format!("{commit}:{PER_REPO_PROVIDERS_FILE}")),
            source,
        })?;
    let providers = PerRepoProviders::parse(
        &providers_raw,
        Path::new(&format!("{commit}:{PER_REPO_PROVIDERS_FILE}")),
    )?;
    let granted = providers
        .roles
        .get(req.role)
        .map(|assignment| assignment.tools.clone())
        .unwrap_or_default();

    let commit_subject = format!("dispatch: {} [{sub_branch}]", req.role);
    spawn_subagent_branch(
        &SpawnRequest {
            parent_worktree: req.parent_worktree,
            parent_branch: req.parent_branch,
            sub_branch: &sub_branch,
            sub_worktree: &sub_worktree,
            fork_point: req.fork_point,
            goal_text: req.goal,
            soul_text: Some(&soul),
            granted: &granted,
            commit_subject: &commit_subject,
        },
        git,
    )?;

    // Front door (§2.11): deposit the dispatch message from the parent,
    // then probe-and-launch. The fresh child is quiescent, so the probe
    // launches `lernie advance` — its ordinary driver. This is the whole
    // of "starting" a child: a deposit and the deposit's own launch.
    inbox::deposit(req.repo, &sub_branch, req.parent_branch, req.goal, clock)?;
    inbox::probe_and_launch(req.repo, &sub_branch, launcher).map_err(|source| {
        Error::ExecutorLock {
            path: inbox::inbox_dir(req.repo, &sub_branch),
            source,
        }
    })?;

    Ok(sub_branch)
}

/// A **harness-initiated** dispatch: the §6 workflow bindings' own
/// procedure dispatches (`worker_flush → dispatch(compactor)`, the
/// verifier gate and its reject re-dispatch). It runs [`run`] — the same
/// fork, the same front door, the same budget gate — and differs in one
/// thing only: whose failure a refusal is. A model that asked for a child
/// it cannot have must be told (the `dispatch` built-in re-emits the
/// error as its `tool_result`); a *procedure* that cannot have one has
/// simply reached the ceiling the operator declared, which is not the
/// dispatching branch's failure. So the refusal is reported and the
/// branch steps on, uncompacted or ungated — the §2.7 outcome for any
/// compaction that does not land.
///
/// Every other error still propagates: only the budget refusal is a
/// non-event here.
pub fn run_procedure(
    req: &ChildDispatchRequest<'_>,
    git: &dyn GitRunner,
    clock: &dyn Clock,
    id_gen: &dyn IdGen,
    launcher: &dyn Launcher,
) -> Result<(), Error> {
    match run(req, git, clock, id_gen, launcher) {
        Err(refused @ Error::DispatchRefused { .. }) => {
            eprintln!("lernie: {refused}");
            Ok(())
        }
        other => other.map(drop),
    }
}

/// The `budgets:` block governing this dispatch, read from the same
/// frozen config commit the soul comes from (§2.2, §6). One home for the
/// limits: the child inherits the parent's config through ancestry, so
/// parent and child evaluate the identical declaration.
fn budgets(
    req: &ChildDispatchRequest<'_>,
    commit: &str,
    git: &dyn GitRunner,
) -> Result<crate::config::Budgets, Error> {
    let raw = workspace::show_control(req.repo, commit, WORKFLOW_FILE, git).map_err(|source| {
        Error::ControlRead {
            path: PathBuf::from(format!("{commit}:{WORKFLOW_FILE}")),
            source,
        }
    })?;
    Ok(Workflow::parse(&raw, &PathBuf::from(format!("{commit}:{WORKFLOW_FILE}")))?.budgets)
}

#[cfg(test)]
mod tests;
#[cfg(test)]
mod tests_grant;
