//! The launched driver's own-branch entry (ARCH §2.11 exit protocol).
//!
//! Every launch — a writer's post-deposit probe, the `lernie scan`
//! flush, an exiting executor's self-directed launch — spawns a driver
//! that runs this entry against its target agent. Warrant is decided
//! here, under the lock, never by the launcher (§2.11): [`drive`]
//! acquires-or-exits, and what it finds decides what happens.
//!
//! **The no-op driver path (§2.11 pin 1).** A driver that acquires and
//! finds nothing to deliver exits silently — no step, no epitaph, no
//! further launch. Structurally: this function takes no launcher and
//! deposits nothing, so a found-nothing drive *cannot* relaunch or
//! deposit; the exit-launch recursion terminates here. Found mail is
//! delivered through the ordinary step-boundary drain ([`super::drain`]
//! — delivery commits, work-product transfers included), after
//! rematerializing the worktree if quiescence tore it down (§2.3
//! step 6).
//!
//! **Scope note.** The step that *reacts* to delivered mail — "found-mail
//! → step to a new terminal → exit-launch again" (§2.11) — is `lernie
//! advance`'s (§6, [`super::advance`]). This module is the own-branch
//! delivery entry that verb runs on arrival: `advance` holds its own
//! lease (adopted or acquired) and calls [`deliver`]; [`drive`] is the
//! acquire-and-deliver composition, the §2.11 contract in one call.

use super::drain;
use crate::prompt::Error;
use crate::prompt::inbox;
use crate::template::{GitRunner, ROOT_WORKTREE};
use std::path::Path;

/// What one [`drive`] found and did — derived on the fly, nothing stored.
#[derive(Debug, PartialEq, Eq)]
pub enum DriveOutcome {
    /// Another executor holds the lock: the branch is already driven, so
    /// this driver exits as a clean no-op (§2.11 Writer/driver totality).
    AlreadyDriven,
    /// Acquired and found an empty inbox: the silent exit of §2.11
    /// pin 1 — no step, no epitaph, no further launch.
    NothingToDeliver,
    /// Acquired and delivered this many pending messages as delivery
    /// commits (§2.11 *Delivery*).
    Delivered(usize),
}

/// Drive `agent_id`'s branch: acquire-or-exit, then deliver whatever is
/// pending — or exit silently when nothing is (§2.11 pin 1). The lock is
/// held for the whole delivery and kernel-released on return.
pub fn drive(workspace: &Path, agent_id: &str, git: &dyn GitRunner) -> Result<DriveOutcome, Error> {
    let inbox_dir = inbox::inbox_dir(workspace, agent_id);
    let Some(_lock) = inbox::try_acquire(&inbox_dir).map_err(|source| Error::ExecutorLock {
        path: inbox_dir.clone(),
        source,
    })?
    else {
        return Ok(DriveOutcome::AlreadyDriven);
    };
    match deliver(workspace, agent_id, git)? {
        0 => Ok(DriveOutcome::NothingToDeliver),
        n => Ok(DriveOutcome::Delivered(n)),
    }
}

/// Deliver `agent_id`'s pending mail under a lease the *caller* already
/// holds (§2.11 *Delivery* — only a lock-holding executor delivers):
/// rematerialize the worktree if quiescence tore it down, then run the
/// real drain (stray recovery + delivery commits). Returns how many
/// messages moved. An empty inbox over a torn-down worktree touches
/// nothing; an empty inbox over a live worktree still runs the drain's
/// stray recovery, closing the §2.11 rename-without-commit crash window
/// before the caller reads the tree.
pub(super) fn deliver(
    workspace: &Path,
    agent_id: &str,
    git: &dyn GitRunner,
) -> Result<usize, Error> {
    let inbox_dir = inbox::inbox_dir(workspace, agent_id);
    let pending = drain::pending(&inbox_dir)?.len();
    let worktree = workspace.join(agent_id);
    if !worktree.exists() {
        if pending == 0 {
            return Ok(0);
        }
        rematerialize(workspace, agent_id, &worktree, git)?;
    }
    drain::drain(&worktree, &inbox_dir, agent_id, git)?;
    Ok(pending)
}

/// Rematerialize a torn-down quiescent worktree off the persistent
/// branch ref (§2.3 step 6 — the worktree is disposable materialization,
/// never state): `git worktree add <path> <branch>`, run inside the
/// primary worktree where `.git` lives (§2.2).
fn rematerialize(
    workspace: &Path,
    agent_id: &str,
    worktree: &Path,
    git: &dyn GitRunner,
) -> Result<(), Error> {
    let wt_str = worktree.to_string_lossy().to_string();
    git.run(
        workspace.join(ROOT_WORKTREE).as_path(),
        &["worktree", "add", wt_str.as_str(), agent_id],
    )
    .map_err(|source| Error::Git {
        op: "worktree add (rematerialize)",
        source,
    })
}

#[cfg(test)]
mod tests;
