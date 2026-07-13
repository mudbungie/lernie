//! Production wiring for `lernie advance <workspace> <agent>` (§6).
//!
//! Mirrors the `lernie prompt` deps wiring — the same real components,
//! the same discipline of keeping the bin a thin shim: [`cli_run`] does
//! everything up to the `exec` itself, returning the fully prepared
//! successor [`Command`] (args, `LERNIE_LOCK_FD`, close-on-exec cleared
//! — [`baton::successor_command`]) for the bin to `exec`. The exec
//! stays in the bin because a successful `execve` never returns — the
//! library boundary is the last observable point of this process.

use super::{AdvanceOutcome, run};
use crate::harness_root;
use crate::prompt::inbox::{self, AdvanceLauncher, baton};
use crate::prompt::resolve::resolve_worker;
use crate::prompt::{Deps, Error, RealSleeper, SpawnAdapter, SpawnDispatcher, SystemClock};
use crate::prompt::{NanoIdGen, tool::SpawnTool};
use std::ffi::OsStr;
use std::path::Path;
use std::process::Command;

/// What the bin does after one hop: nothing, or exec the successor.
#[derive(Debug)]
pub enum AdvanceHandoff {
    /// The hop completed in this process — a no-op, an already-driven
    /// exit, or a terminal event whose exit protocol already ran.
    Done(AdvanceOutcome),
    /// The step emitted `tool_use`: exec this prepared successor
    /// command (§6 exec baton). Only `exec` remains — the lease fd is
    /// already inheritable and published in the command's environment.
    Exec(Command),
}

/// Run one production hop: take the lease (adopting a predecessor's
/// [`baton::LOCK_FD_ENV`] fd from the live environment, else
/// acquiring), drive [`run`] with the real components, and prepare the
/// §6 handoff.
pub fn cli_run(workspace: &Path, agent_id: &str) -> Result<AdvanceHandoff, Error> {
    cli_run_with(
        workspace,
        agent_id,
        std::env::var_os(baton::LOCK_FD_ENV).as_deref(),
    )
}

/// [`cli_run`] with the lease env value injected — the same
/// env-as-parameter discipline as `inbox::resolve_cli_sender`, so the
/// adopt arm is exercisable without mutating the test process's
/// environment.
fn cli_run_with(
    workspace: &Path,
    agent_id: &str,
    lease_env: Option<&OsStr>,
) -> Result<AdvanceHandoff, Error> {
    let inbox_dir = inbox::inbox_dir(workspace, agent_id);
    let lease = match baton::take_lease(lease_env, &inbox_dir) {
        Ok(Some(lease)) => lease,
        Ok(None) => return Ok(AdvanceHandoff::Done(AdvanceOutcome::AlreadyDriven)),
        Err(baton::LeaseError::Acquire(source)) => {
            return Err(Error::ExecutorLock {
                path: inbox_dir,
                source,
            });
        }
        Err(baton::LeaseError::Adopt(e)) => {
            return Err(Error::LeaseAdopt {
                agent: agent_id.to_string(),
                detail: e.to_string(),
            });
        }
    };

    let exe = std::env::current_exe()?;
    let dispatcher = SpawnDispatcher::new()?;
    let roots = harness_root::resolve()?;
    let tool_executor = SpawnTool::new(&roots.data, &SystemClock);
    let launcher = AdvanceLauncher::with_exe(exe.clone());
    let deps = Deps {
        adapter: &SpawnAdapter,
        sleeper: &RealSleeper,
        git: &crate::template::RealGit::new(),
        clock: &SystemClock,
        id_gen: &NanoIdGen,
        dispatcher: &dispatcher,
        tool_executor: &tool_executor,
        config_root: &roots.config,
        stop: crate::prompt::stop_flag(),
        launcher: &launcher,
    };

    let outcome = run(workspace, agent_id, Some(lease), &deps, &mut || {
        resolve_worker(workspace, &deps)
    })?;
    handoff(&exe, workspace, agent_id, outcome)
}

/// Map a hop's outcome to the bin's next act (§6 step 5): tools ran →
/// the prepared successor exec; anything else completed here.
fn handoff(
    exe: &Path,
    workspace: &Path,
    agent_id: &str,
    outcome: AdvanceOutcome,
) -> Result<AdvanceHandoff, Error> {
    match outcome {
        AdvanceOutcome::ToolsPending(lease) => Ok(AdvanceHandoff::Exec(baton::successor_command(
            exe, workspace, agent_id, lease,
        )?)),
        done => Ok(AdvanceHandoff::Done(done)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::prompt::inbox::{ExecutorLock, inbox_dir};
    use tempfile::TempDir;

    /// Take a lease for tests: acquire on a scratch inbox, or die trying.
    fn test_lease(dir: &Path) -> ExecutorLock {
        inbox::try_acquire(dir).unwrap().expect("free lock")
    }

    #[test]
    fn empty_workspace_is_nothing_to_do_via_production_wiring() {
        let ws = TempDir::new().unwrap();
        let out = cli_run(ws.path(), "20260101-a1").unwrap();
        assert!(matches!(
            out,
            AdvanceHandoff::Done(AdvanceOutcome::NothingToDo)
        ));
    }

    #[test]
    fn held_lock_is_already_driven() {
        let ws = TempDir::new().unwrap();
        let _held = test_lease(&inbox_dir(ws.path(), "20260101-a1"));
        let out = cli_run(ws.path(), "20260101-a1").unwrap();
        assert!(matches!(
            out,
            AdvanceHandoff::Done(AdvanceOutcome::AlreadyDriven)
        ));
    }

    #[test]
    fn broken_inbox_surfaces_as_executor_lock_error() {
        let ws = TempDir::new().unwrap();
        std::fs::create_dir_all(ws.path().join("inbox")).unwrap();
        std::fs::write(inbox_dir(ws.path(), "20260101-a1"), b"not a dir").unwrap();
        let err = cli_run(ws.path(), "20260101-a1").unwrap_err();
        assert!(matches!(err, Error::ExecutorLock { .. }), "{err}");
    }

    #[test]
    fn bad_lease_env_is_declined_loudly_as_lease_adopt() {
        let ws = TempDir::new().unwrap();
        std::fs::create_dir_all(inbox_dir(ws.path(), "20260101-a1")).unwrap();
        let err =
            cli_run_with(ws.path(), "20260101-a1", Some(OsStr::new("not-an-fd"))).unwrap_err();
        assert!(matches!(err, Error::LeaseAdopt { .. }), "{err}");
    }

    #[test]
    fn adopted_lease_env_drives_the_hop() {
        // Simulate the predecessor: acquire, publish the fd number, and
        // leak the guard (exactly what `successor_command` does before
        // exec). The adopting hop finds nothing due on the empty branch.
        let ws = TempDir::new().unwrap();
        let dir = inbox_dir(ws.path(), "20260101-a1");
        let lease = test_lease(&dir);
        let fd = lease.as_raw_fd().to_string();
        std::mem::forget(lease);
        let out = cli_run_with(ws.path(), "20260101-a1", Some(OsStr::new(&fd))).unwrap();
        assert!(matches!(
            out,
            AdvanceHandoff::Done(AdvanceOutcome::NothingToDo)
        ));
    }

    #[test]
    fn a_warranted_hop_delivers_then_consults_the_resolver() {
        // A real branch with pending mail: the hop delivers (real git),
        // finds the tail user-side, and consults the production
        // resolver — loud on a workspace with no role config. That the
        // delivery landed before the resolution failed is the §6 lazy-
        // resolution ordering, observed on disk.
        use crate::template::GitRunner;
        let ws = TempDir::new().unwrap();
        let root = ws.path().join(crate::template::ROOT_WORKTREE);
        std::fs::create_dir_all(&root).unwrap();
        let g = crate::template::RealGit::new();
        g.run(&root, &["init", "-b", "main"]).unwrap();
        g.run(&root, &["config", "user.email", "t@test.invalid"])
            .unwrap();
        g.run(&root, &["config", "user.name", "t"]).unwrap();
        g.run(&root, &["commit", "--allow-empty", "-m", "init"])
            .unwrap();
        let agent = "20260101-a1";
        let wt = ws.path().join(agent);
        g.run(
            &root,
            &[
                "worktree",
                "add",
                "-b",
                agent,
                wt.to_string_lossy().as_ref(),
                "main",
            ],
        )
        .unwrap();
        inbox::deposit(ws.path(), agent, "user", "hi", &SystemClock).unwrap();
        let err = cli_run(ws.path(), agent).unwrap_err();
        assert!(!err.to_string().is_empty());
        // The delivery commit landed ahead of the failed resolution.
        assert!(wt.join("messages/001-user.md").exists());
    }

    #[test]
    fn tools_pending_hands_off_as_a_prepared_exec() {
        let ws = TempDir::new().unwrap();
        let lease = test_lease(&inbox_dir(ws.path(), "20260101-a1"));
        let out = handoff(
            Path::new("/usr/bin/lernie"),
            ws.path(),
            "20260101-a1",
            AdvanceOutcome::ToolsPending(lease),
        )
        .unwrap();
        let AdvanceHandoff::Exec(cmd) = out else {
            panic!("expected Exec");
        };
        let args: Vec<_> = cmd.get_args().map(|a| a.to_os_string()).collect();
        assert_eq!(args[0], "advance");
        assert!(
            cmd.get_envs()
                .any(|(k, v)| k == baton::LOCK_FD_ENV && v.is_some())
        );
    }

    #[test]
    fn non_tools_outcomes_hand_off_as_done() {
        let ws = TempDir::new().unwrap();
        let out = handoff(
            Path::new("lernie"),
            ws.path(),
            "20260101-a1",
            AdvanceOutcome::NothingToDo,
        )
        .unwrap();
        assert!(matches!(
            out,
            AdvanceHandoff::Done(AdvanceOutcome::NothingToDo)
        ));
    }
}
