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
use crate::prompt::resolve::{ConfigSource, resolve_worker};
use crate::prompt::{Deps, Error, RealSleeper, SpawnAdapter, SystemClock};
use crate::prompt::{NanoIdGen, tool::SpawnTool};
use std::ffi::OsStr;
use std::path::Path;
use std::process::Command;
use std::sync::atomic::AtomicBool;

/// What the bin does after one hop: nothing, or exec the successor.
#[derive(Debug)]
pub enum AdvanceHandoff {
    /// The hop completed in this process — a no-op, an already-driven
    /// exit, or a terminal event whose exit protocol already ran. The
    /// hop's outcome carries no product here: `cmd::advance::outcome_of`
    /// maps every completed hop to a product-less `Outcome::Quiet`
    /// (§3.4), so nothing downstream reads it.
    Done,
    /// The step emitted `tool_use`: exec this prepared successor
    /// command (§6 exec baton). Only `exec` remains — the lease fd is
    /// already inheritable and published in the command's environment.
    Exec(Command),
}

/// Run one production hop: take the lease (adopting a predecessor's
/// [`baton::LOCK_FD_ENV`] fd from the live environment, else
/// acquiring), drive [`run`] with the real components, and prepare the
/// §6 handoff. `driver_target` is the running-binary path the exec
/// binding injects (`cmd::Fx::driver_target`, §3.4) — it is the
/// successor `execve` target, the launcher's detached-spawn target,
/// *and* the §3.3 tool resolver's third hop, so the library resolves no
/// `current_exe` of its own; `stop` is the
/// executor's injected SIGTERM flag (`cmd::Fx::stop`, §2.9).
pub fn cli_run(
    workspace: &Path,
    agent_id: &str,
    driver_target: &Path,
    stop: &AtomicBool,
) -> Result<AdvanceHandoff, Error> {
    cli_run_with(
        workspace,
        agent_id,
        std::env::var_os(baton::LOCK_FD_ENV).as_deref(),
        driver_target,
        stop,
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
    driver_target: &Path,
    stop: &AtomicBool,
) -> Result<AdvanceHandoff, Error> {
    crate::workspace::require(workspace)?;
    let inbox_dir = inbox::inbox_dir(workspace, agent_id);
    let lease = match baton::take_lease(lease_env, &inbox_dir) {
        Ok(Some(lease)) => lease,
        Ok(None) => return Ok(AdvanceHandoff::Done),
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

    let roots = harness_root::resolve()?;
    let tool_executor = SpawnTool::new(&roots.data, &SystemClock, driver_target);
    let launcher = AdvanceLauncher::with_exe(driver_target.to_path_buf());
    let deps = Deps {
        adapter: &SpawnAdapter,
        sleeper: &RealSleeper,
        git: &crate::template::RealGit::new(),
        clock: &SystemClock,
        id_gen: &NanoIdGen,
        tool_executor: &tool_executor,
        config_root: &roots.config,
        stop,
        launcher: &launcher,
    };

    let outcome = run(workspace, agent_id, Some(lease), &deps, &mut || {
        resolve_worker(workspace, ConfigSource::Agent(agent_id), &deps)
    })?;
    handoff(driver_target, workspace, agent_id, outcome)
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
        _ => Ok(AdvanceHandoff::Done),
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

    /// The injected driver target for tests — a bare name; these hops all
    /// error before any spawn/exec would consult it.
    fn td() -> &'static Path {
        Path::new("lernie")
    }

    #[test]
    fn a_non_workspace_is_refused_by_the_layout_guard() {
        // Pre-v1 clean break (§2.2, §10): the guard fires before any
        // lease or inbox work.
        let ws = TempDir::new().unwrap();
        let err = cli_run(ws.path(), "20260101-a1", td(), &AtomicBool::new(false)).unwrap_err();
        assert!(matches!(err, Error::Layout(_)), "{err}");
    }

    #[test]
    fn empty_workspace_is_nothing_to_do_via_production_wiring() {
        let (_h, ws) = crate::workspace::fixture::workspace();
        let out = cli_run(&ws, "20260101-a1", td(), &AtomicBool::new(false)).unwrap();
        assert!(matches!(out, AdvanceHandoff::Done));
    }

    #[test]
    fn held_lock_is_already_driven() {
        let (_h, ws) = crate::workspace::fixture::workspace();
        let _held = test_lease(&inbox_dir(&ws, "20260101-a1"));
        let out = cli_run(&ws, "20260101-a1", td(), &AtomicBool::new(false)).unwrap();
        assert!(matches!(out, AdvanceHandoff::Done));
    }

    #[test]
    fn broken_inbox_surfaces_as_executor_lock_error() {
        let (_h, ws) = crate::workspace::fixture::workspace();
        std::fs::create_dir_all(ws.join("inbox")).unwrap();
        std::fs::write(inbox_dir(&ws, "20260101-a1"), b"not a dir").unwrap();
        let err = cli_run(&ws, "20260101-a1", td(), &AtomicBool::new(false)).unwrap_err();
        assert!(matches!(err, Error::ExecutorLock { .. }), "{err}");
    }

    #[test]
    fn bad_lease_env_is_declined_loudly_as_lease_adopt() {
        let (_h, ws) = crate::workspace::fixture::workspace();
        std::fs::create_dir_all(inbox_dir(&ws, "20260101-a1")).unwrap();
        let err = cli_run_with(
            &ws,
            "20260101-a1",
            Some(OsStr::new("not-an-fd")),
            td(),
            &AtomicBool::new(false),
        )
        .unwrap_err();
        assert!(matches!(err, Error::LeaseAdopt { .. }), "{err}");
    }

    #[test]
    fn adopted_lease_env_drives_the_hop() {
        // Simulate the predecessor: acquire, publish the fd number, and
        // leak the guard (exactly what `successor_command` does before
        // exec). The adopting hop finds nothing due on the empty branch.
        let (_h, ws) = crate::workspace::fixture::workspace();
        let dir = inbox_dir(&ws, "20260101-a1");
        let lease = test_lease(&dir);
        let fd = lease.as_raw_fd().to_string();
        std::mem::forget(lease);
        let out = cli_run_with(
            &ws,
            "20260101-a1",
            Some(OsStr::new(&fd)),
            td(),
            &AtomicBool::new(false),
        )
        .unwrap();
        assert!(matches!(out, AdvanceHandoff::Done));
    }

    #[test]
    fn a_warranted_hop_delivers_then_consults_the_resolver() {
        // A real branch with pending mail: the hop delivers (real git),
        // finds the tail user-side, and consults the production
        // resolver — loud against the test-machine harness root, whose
        // global models.yaml does not carry the template's models. That
        // the delivery landed before the resolution failed is the §6
        // lazy-resolution ordering, observed on disk.
        let (_h, ws) = crate::workspace::fixture::workspace();
        let agent = "20260101-a1";
        let wt = crate::workspace::fixture::spawn_root(&ws, agent);
        inbox::deposit(&ws, agent, "user", "hi", &SystemClock).unwrap();
        let err = cli_run(&ws, agent, td(), &AtomicBool::new(false)).unwrap_err();
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
        assert!(matches!(out, AdvanceHandoff::Done));
    }
}
