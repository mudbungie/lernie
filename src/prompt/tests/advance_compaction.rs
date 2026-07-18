//! §6 live-compaction / role-aware wiring on the `lernie advance` hop:
//! the delivered-child-result interpretation reached through `run`, and
//! the compactor-role built-in-toolset injection. Split out of
//! [`super::advance`] so that file stays under the per-file line cap; the
//! shared helpers (`worker_config`, `AGENT`, `RecLauncher`, …) live there.

use super::advance::{AGENT, RecLauncher, terminal_tail, worker_config, workspace_with_tail};
use super::fixtures::*;
use crate::prompt::dispatch::advance::{AdvanceOutcome, run};
use crate::prompt::inbox;
use crate::prompt::resolve::WorkerConfig;

/// A [`worker_config`] specialized to the compactor role — the shape a
/// dispatched compactor resolves (§6). Drives the step's built-in-toolset
/// injection (§2.7).
fn compactor_config() -> WorkerConfig {
    WorkerConfig {
        role: "compactor".into(),
        ..worker_config()
    }
}

#[test]
fn a_pending_worker_result_is_interpreted_then_the_branch_steps() {
    // §6 end-to-end wiring on the hop: a worker child's result message,
    // left in the inbox by the drain, is interpreted (deliver_result:
    // transfer + transcript delivery), which makes the tail user-side, and
    // the branch steps to react — reusing the config resolved for the
    // interpretation. Real git (transfer needs it); stub adapter/tools.
    use crate::prompt::child_dispatch::{ChildDispatchRequest, run as dispatch_child};
    use crate::prompt::inbox::deposit_result;
    use crate::template::{GitRunner, RealGit};
    use crate::workspace::{agent_worktree, fixture};

    let (_h, ws) = fixture::workspace();
    let parent = AGENT;
    let parent_wt = fixture::spawn_root(&ws, parent);
    let git = RealGit::new();
    let (clock, id) = (FixedClock::default(), FixedIdGen);
    let rec = RecLauncher::default();
    // Fork a worker child, commit a work product, deposit its result.
    let req = ChildDispatchRequest {
        repo: &ws,
        parent_branch: parent,
        parent_worktree: &parent_wt,
        role: "worker",
        goal: "do it",
        fork_point: None,
    };
    let child = dispatch_child(&req, &git, &clock, &id, &rec).unwrap();
    let child_wt = agent_worktree(&ws, &child);
    std::fs::write(child_wt.join("out.txt"), "result\n").unwrap();
    git.run(&child_wt, &["add", "-A"]).unwrap();
    git.run(&child_wt, &["commit", "-m", "work"]).unwrap();
    let tip = git.run_capture(&child_wt, &["rev-parse", "HEAD"]).unwrap();
    deposit_result(
        &ws,
        parent,
        &child,
        inbox::Epitaph::FinalResponse,
        tip.trim(),
        Some("done"),
        &clock,
    )
    .unwrap();

    let adapter = StubAdapter::scripted([StubAdapter::reply_ok(&happy_response_bytes())]);
    let (sleeper, tools, stub_git) = (
        StubSleeper::default(),
        StubToolExecutor::ok(),
        StubGit::ok(),
    );
    let mut deps = valid_deps(&adapter, &sleeper, &stub_git, &clock, &id, &tools, &ws);
    deps.git = &git;
    deps.launcher = &rec;
    let out = run(&ws, parent, None, &deps, &mut || Ok(worker_config())).unwrap();

    assert!(matches!(
        out,
        AdvanceOutcome::Terminal(inbox::Epitaph::FinalResponse)
    ));
    // The child's work product transferred into the parent tree, and its
    // result message delivered to the transcript, then a step answered.
    assert_eq!(
        std::fs::read_to_string(parent_wt.join("out.txt")).unwrap(),
        "result\n"
    );
    assert!(parent_wt.join(format!("messages/001-{child}.md")).exists());
}

#[test]
fn a_compactor_hop_injects_the_builtin_toolset_into_the_request() {
    // §2.7/§6 role-aware resolution: a compactor-role hop composes the
    // built-in write_summary / mark_for_deletion schemas into the request,
    // even though no `descriptions/**` or `providers.yaml` list carries
    // them. Asserted on the step's request.json (written before the call).
    let (ws, _wt) = workspace_with_tail(&terminal_tail());
    let (clock, id) = (FixedClock::default(), FixedIdGen);
    inbox::deposit(ws.path(), AGENT, "user", "compact", &clock).unwrap();
    let adapter = StubAdapter::scripted([StubAdapter::reply_ok(&happy_response_bytes())]);
    let (sleeper, git) = (StubSleeper::default(), StubGit::ok());
    let tools = StubToolExecutor::ok();
    let rec = RecLauncher::default();
    let mut deps = valid_deps(&adapter, &sleeper, &git, &clock, &id, &tools, ws.path());
    deps.launcher = &rec;
    run(
        ws.path(),
        AGENT,
        None,
        &deps,
        &mut || Ok(compactor_config()),
    )
    .unwrap();

    let req: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(ws.path().join(format!("steps/{AGENT}/001/request.json")))
            .unwrap(),
    )
    .unwrap();
    let names: Vec<&str> = req["tools"]
        .as_array()
        .unwrap()
        .iter()
        .map(|t| t["name"].as_str().unwrap())
        .collect();
    assert!(names.contains(&"write_summary"), "{names:?}");
    assert!(names.contains(&"mark_for_deletion"), "{names:?}");
}
