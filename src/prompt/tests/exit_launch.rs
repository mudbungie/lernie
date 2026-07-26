//! The §2.11 exit protocol at the step loop's terminal seam: deposit →
//! release own lock → spawn a driver at own agent *and* at the parent
//! the deposit revived → exit. Pin 2 (launch by epitaph value: final
//! response launches, `stopped` and `budget-exhausted` never — at the
//! parent as much as at the exiting agent), the post-release
//! no-authority ordering (the launcher observes a free lock and an
//! already-landed deposit), and the parentless case (deposit no-ops,
//! self-launch still fires, nothing is revived). The child-path
//! revival — a real `lernie advance` child terminal waking a real
//! parent — is [`super::parent_revival`]. The
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
    pub(super) invocations: RefCell<Vec<(PathBuf, String, bool, bool)>>,
}

impl Launcher for ProbingLauncher {
    fn launch(&self, workspace: &Path, agent_id: &str) -> io::Result<()> {
        let lock_free = probe_until_free(workspace, agent_id)?;
        let deposited = deposit_files(workspace)
            .iter()
            .any(|n| n.contains("epitaph"));
        self.invocations.borrow_mut().push((
            workspace.to_path_buf(),
            agent_id.to_string(),
            lock_free,
            deposited,
        ));
        Ok(())
    }
}

/// Bounded retries for every executor-lock probe in these tests — here,
/// in [`super::exit_race`], and in [`super::advance`]. The §2.11 ordering
/// under test released the fd before launching, but a
/// concurrent test thread's `Command` spawn can fork while that fd was
/// still open and hold the inherited duplicate for the fork→exec window
/// (all fds are CLOEXEC, so exec drops it microseconds later). A genuine
/// ordering bug holds the lock forever and still fails; the fork window
/// clears in a retry or two.
///
/// A count, not a duration: the budget must not shrink because the
/// machine is busy, and the give-up arm must be reached by the same
/// number of iterations on every run — a wall-clock deadline makes the
/// retry sleep's coverage load-dependent (see [`super::advance::free_within`]).
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
fn final_response_exit_launches_own_agent_and_revives_the_parent() {
    let repo = scaffold_repo(VALID_PER_REPO_PROVIDERS_YAML, Some("body"));
    let harness = scaffold_harness_root();
    let adapter = StubAdapter::happy(&stream_of(brazen::FinishReason::Stop, &[Block::Text("hi")]));
    let git = StubGit::ok();
    let (clock, id) = (FixedClock::default(), FixedIdGen);
    let (sleeper, tool_executor) = (StubSleeper::default(), StubToolExecutor::ok());
    let launcher = ProbingLauncher::default();

    let mut deps = valid_deps(
        &adapter,
        &sleeper,
        &git,
        &clock,
        &id,
        &tool_executor,
        harness.path(),
    );
    deps.launcher = &launcher;

    run(repo.path(), "go", &deps).unwrap();
    let invocations = launcher.invocations.borrow();
    // Two launches: the self-directed one, then the parent whose inbox
    // the terminal deposit just landed in (§2.11 revival-on-deposit).
    assert_eq!(invocations.len(), 2);
    let (ws, agent, lock_free, deposited) = &invocations[0];
    assert_eq!(ws, repo.path());
    assert_eq!(agent, "ct-1-deadbeef");
    // §2.11 ordering: deposit → release → launch. The launcher sees both.
    assert!(*lock_free, "the lock must be released before the launch");
    assert!(*deposited, "the result deposit must land before the launch");
    // The parent is addressed by derivation alone — this agent's id
    // minus its last descent segment (§2.11) — and its lease is free,
    // so the probe launched rather than leaving the result undelivered.
    let (pws, parent, parent_free, _) = &invocations[1];
    assert_eq!(pws, repo.path());
    assert_eq!(parent, "ct");
    assert!(*parent_free, "the parent was quiescent, so it is launched");
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
    let (clock, id) = (RootClock, FixedIdGen);
    let (sleeper, tool_executor) = (StubSleeper::default(), StubToolExecutor::ok());
    let launcher = ProbingLauncher::default();
    let deps = Deps {
        adapter: &adapter,
        sleeper: &sleeper,
        git: &git,
        clock: &clock,
        id_gen: &id,
        tool_executor: &tool_executor,
        config_root: harness.path(),
        adapter_target: None,
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
    let invocations = launcher.invocations.borrow();
    assert_eq!(invocations.len(), 1);
    assert_eq!(invocations[0].1, "ct1-deadbeef");
}

#[test]
fn stopped_exit_never_launches() {
    // §2.11 pin 2: `stopped` → never (a relaunch would resurrect the
    // branch the operator just killed) — and never at the parent
    // either: waking it would hand it a stop to undo one level up.
    // The conv-id here is child-shaped (parent `ct`), so a parent-side
    // launch would show up in the recorder.
    let repo = scaffold_repo(VALID_PER_REPO_PROVIDERS_YAML, Some("body"));
    let harness = scaffold_harness_root();
    let adapter = StubAdapter::scripted([StubAdapter::reply_ok(&version_line())]);
    let git = StubGit::ok();
    let (clock, id) = (FixedClock::default(), FixedIdGen);
    let (sleeper, tool_executor) = (StubSleeper::default(), StubToolExecutor::ok());
    let stop = AtomicBool::new(true);
    let launcher = ProbingLauncher::default();

    let mut deps = valid_deps(
        &adapter,
        &sleeper,
        &git,
        &clock,
        &id,
        &tool_executor,
        harness.path(),
    );
    deps.stop = &stop;
    deps.launcher = &launcher;

    run(repo.path(), "go", &deps).unwrap();
    assert!(
        launcher.invocations.borrow().is_empty(),
        "stopped must not launch"
    );
}

#[test]
fn budget_exhausted_exit_never_launches() {
    // §2.11 pin 2: `budget-exhausted` → never (epitaph-spam cycle) —
    // at the parent too, since the ceiling is derived over the whole
    // tree (§6), so a revived parent would exhaust on its own next
    // check and deposit again. The conv-id is child-shaped (parent
    // `ct`): a parent-side launch would be recorded.
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
    let (clock, id) = (FixedClock::default(), FixedIdGen);
    let (sleeper, tool_executor) = (StubSleeper::default(), StubToolExecutor::ok());
    let launcher = ProbingLauncher::default();

    let mut deps = valid_deps(
        &adapter,
        &sleeper,
        &git,
        &clock,
        &id,
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
        launcher.invocations.borrow().is_empty(),
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
    let (clock, id) = (FixedClock::default(), FixedIdGen);
    let (sleeper, tool_executor) = (StubSleeper::default(), StubToolExecutor::ok());
    let launcher = ProbingLauncher::default();

    let mut deps = valid_deps(
        &adapter,
        &sleeper,
        &git,
        &clock,
        &id,
        &tool_executor,
        harness.path(),
    );
    deps.launcher = &launcher;

    run(repo.path(), "go", &deps).unwrap_err();
    assert!(
        launcher.invocations.borrow().is_empty(),
        "an error must not launch"
    );
}
