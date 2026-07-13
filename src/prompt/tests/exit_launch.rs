//! The §2.11 exit protocol at the step loop's terminal seam: deposit →
//! release own lock → spawn a driver at own agent → exit. Pin 2 (launch
//! by epitaph value: final response launches, `stopped` and
//! `budget-exhausted` never), the post-release no-authority ordering
//! (the launcher observes a free lock and an already-landed deposit),
//! and the parentless case (deposit no-ops, launch still fires). The
//! fire-and-forget swallow, the helper negatives, and the real-git exit
//! race live in [`super::exit_race`] — split for the per-file line cap.

use super::fixtures::*;
use crate::prompt::inbox::{Launcher, inbox_dir, try_acquire};
use crate::prompt::{Clock, Deps, run};
use std::cell::RefCell;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::atomic::AtomicBool;

/// Records each launch with what the §2.11 ordering guarantees at that
/// instant: the executor lock already released (a probe succeeds) and
/// the terminal deposit already landed (the parent inbox holds it).
#[derive(Default)]
pub(super) struct ProbingLauncher {
    pub(super) calls: RefCell<Vec<(PathBuf, String, bool, bool)>>,
}

impl Launcher for ProbingLauncher {
    fn launch(&self, workspace: &Path, agent_id: &str) -> io::Result<()> {
        let lock_free = probe_until_free(workspace, agent_id)?;
        let deposited = deposit_files(workspace)
            .iter()
            .any(|n| n.contains("epitaph"));
        self.calls.borrow_mut().push((
            workspace.to_path_buf(),
            agent_id.to_string(),
            lock_free,
            deposited,
        ));
        Ok(())
    }
}

/// Bounded retries for the probes here and in [`super::exit_race`]. The
/// §2.11 ordering under test released the fd before launching, but a
/// concurrent test thread's `Command` spawn can fork while that fd was
/// still open and hold the inherited duplicate for the fork→exec window
/// (all fds are CLOEXEC, so exec drops it microseconds later). A genuine
/// ordering bug holds the lock forever and still fails; the fork window
/// clears in a retry or two.
pub(super) const PROBE_RETRIES: u32 = 60;

/// Probe the executor lock with the bounded retry above.
pub(super) fn probe_until_free(workspace: &Path, agent_id: &str) -> io::Result<bool> {
    for _ in 0..PROBE_RETRIES {
        if try_acquire(&inbox_dir(workspace, agent_id))?.is_some() {
            return Ok(true);
        }
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
    Ok(false)
}

/// Every deposited file body under `<workspace>/inbox/**` (flat walk).
pub(super) fn deposit_files(workspace: &Path) -> Vec<String> {
    let mut out = Vec::new();
    let Ok(agents) = std::fs::read_dir(workspace.join("inbox")) else {
        return out;
    };
    for agent in agents.flatten() {
        let Ok(rd) = std::fs::read_dir(agent.path()) else {
            continue;
        };
        for f in rd.flatten() {
            out.push(std::fs::read_to_string(f.path()).unwrap_or_default());
        }
    }
    out
}

#[test]
fn final_response_exit_launches_own_agent_after_release_and_deposit() {
    let repo = scaffold_repo(VALID_PER_REPO_PROVIDERS_YAML, Some("body"));
    let harness = scaffold_harness_root();
    let adapter = StubAdapter::happy(&stream_of(brazen::FinishReason::Stop, &[Block::Text("hi")]));
    let git = StubGit::ok();
    let (clock, id, dispatcher) = (FixedClock::default(), FixedIdGen, StubDispatcher::ok());
    let (sleeper, tool_executor) = (StubSleeper::default(), StubToolExecutor::ok());
    let launcher = ProbingLauncher::default();

    let mut deps = valid_deps(
        &adapter,
        &sleeper,
        &git,
        &clock,
        &id,
        &dispatcher,
        &tool_executor,
        harness.path(),
    );
    deps.launcher = &launcher;

    run(repo.path(), "go", &deps).unwrap();
    let calls = launcher.calls.borrow();
    // One launch, at the exiting agent itself.
    assert_eq!(calls.len(), 1);
    let (ws, agent, lock_free, deposited) = &calls[0];
    assert_eq!(ws, repo.path());
    assert_eq!(agent, "ct-1-deadbeef");
    // §2.11 ordering: deposit → release → launch. The launcher sees both.
    assert!(*lock_free, "the lock must be released before the launch");
    assert!(*deposited, "the result deposit must land before the launch");
}

/// A hyphen-free compact stamp makes the conv-id a two-token *root*
/// (`parent_of` = None): the parentless arm of the terminal sequence.
struct RootClock;
impl Clock for RootClock {
    fn now_iso8601(&self) -> String {
        "iso".into()
    }
    fn now_compact(&self) -> String {
        "ct1".into()
    }
}

#[test]
fn parentless_agent_deposit_noops_but_exit_launch_still_fires() {
    let repo = scaffold_repo(VALID_PER_REPO_PROVIDERS_YAML, Some("body"));
    let harness = scaffold_harness_root();
    let adapter = StubAdapter::happy(&stream_of(brazen::FinishReason::Stop, &[Block::Text("hi")]));
    let git = StubGit::ok();
    let (clock, id, dispatcher) = (RootClock, FixedIdGen, StubDispatcher::ok());
    let (sleeper, tool_executor) = (StubSleeper::default(), StubToolExecutor::ok());
    let launcher = ProbingLauncher::default();
    let deps = Deps {
        adapter: &adapter,
        sleeper: &sleeper,
        git: &git,
        clock: &clock,
        id_gen: &id,
        dispatcher: &dispatcher,
        tool_executor: &tool_executor,
        config_root: harness.path(),
        stop: never_stopped(),
        launcher: &launcher,
    };

    let branch = run(repo.path(), "go", &deps).unwrap();
    assert_eq!(branch, "ct1-deadbeef", "two tokens: a parentless root");
    // The deposit is a structural no-op — no result message anywhere…
    assert!(
        !deposit_files(repo.path())
            .iter()
            .any(|b| b.contains("epitaph")),
        "a root deposits no result"
    );
    // …and the one unconditional sequence still launches (no agent kinds).
    let calls = launcher.calls.borrow();
    assert_eq!(calls.len(), 1);
    assert_eq!(calls[0].1, "ct1-deadbeef");
}

#[test]
fn stopped_exit_never_launches() {
    // §2.11 pin 2: `stopped` → never (a relaunch would resurrect the
    // branch the operator just killed).
    let repo = scaffold_repo(VALID_PER_REPO_PROVIDERS_YAML, Some("body"));
    let harness = scaffold_harness_root();
    let adapter = StubAdapter::scripted([StubAdapter::reply_ok(&version_line())]);
    let git = StubGit::ok();
    let (clock, id, dispatcher) = (FixedClock::default(), FixedIdGen, StubDispatcher::ok());
    let (sleeper, tool_executor) = (StubSleeper::default(), StubToolExecutor::ok());
    let stop = AtomicBool::new(true);
    let launcher = ProbingLauncher::default();

    let mut deps = valid_deps(
        &adapter,
        &sleeper,
        &git,
        &clock,
        &id,
        &dispatcher,
        &tool_executor,
        harness.path(),
    );
    deps.stop = &stop;
    deps.launcher = &launcher;

    run(repo.path(), "go", &deps).unwrap();
    assert!(
        launcher.calls.borrow().is_empty(),
        "stopped must not launch"
    );
}

#[test]
fn budget_exhausted_exit_never_launches() {
    // §2.11 pin 2: `budget-exhausted` → never (epitaph-spam cycle).
    const EXHAUSTING: &str = "events: {}\nbudgets:\n  max_total_tokens: 8\n";
    let repo = scaffold_repo_with_workflow(VALID_PER_REPO_PROVIDERS_YAML, EXHAUSTING, Some("body"));
    let harness = scaffold_harness_root();
    let adapter = StubAdapter::scripted([
        StubAdapter::reply_ok(&version_line()),
        StubAdapter::reply_ok(&stream_of(
            brazen::FinishReason::ToolUse,
            &[Block::ToolUse {
                id: "toolu_01",
                name: "bash",
                input: serde_json::json!({"cmd": "ls"}),
            }],
        )),
    ]);
    let git = StubGit::ok();
    let (clock, id, dispatcher) = (FixedClock::default(), FixedIdGen, StubDispatcher::ok());
    let (sleeper, tool_executor) = (StubSleeper::default(), StubToolExecutor::ok());
    let launcher = ProbingLauncher::default();

    let mut deps = valid_deps(
        &adapter,
        &sleeper,
        &git,
        &clock,
        &id,
        &dispatcher,
        &tool_executor,
        harness.path(),
    );
    deps.launcher = &launcher;

    run(repo.path(), "go", &deps).unwrap();
    assert!(
        deposit_files(repo.path())
            .iter()
            .any(|b| b.contains("epitaph: budget-exhausted")),
        "the exhaustion deposit landed"
    );
    assert!(
        launcher.calls.borrow().is_empty(),
        "exhausted must not launch"
    );
}

#[test]
fn an_errored_executor_never_launches() {
    // An executor error is not a terminal event: it deposits nothing and
    // launches nothing — the accepted crash class (§2.11); the next
    // touch heals.
    let repo = scaffold_repo(VALID_PER_REPO_PROVIDERS_YAML, Some("body"));
    let harness = scaffold_harness_root();
    let adapter = StubAdapter::scripted([
        StubAdapter::reply_ok(&version_line()),
        StubAdapter::reply_err(io::ErrorKind::ConnectionRefused, "no provider"),
    ]);
    let git = StubGit::ok();
    let (clock, id, dispatcher) = (FixedClock::default(), FixedIdGen, StubDispatcher::ok());
    let (sleeper, tool_executor) = (StubSleeper::default(), StubToolExecutor::ok());
    let launcher = ProbingLauncher::default();

    let mut deps = valid_deps(
        &adapter,
        &sleeper,
        &git,
        &clock,
        &id,
        &dispatcher,
        &tool_executor,
        harness.path(),
    );
    deps.launcher = &launcher;

    run(repo.path(), "go", &deps).unwrap_err();
    assert!(
        launcher.calls.borrow().is_empty(),
        "an error must not launch"
    );
}
