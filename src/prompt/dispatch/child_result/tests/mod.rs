//! Tests for the §6 delivered-child-result and checkpoint-flush seams.
//! Real git + a real scaffolded workspace (the shapes production runs
//! against); the adapter/sleeper/tool-executor deps are never reached on
//! these paths, so they are `unreachable!` stubs.

use super::*;
use crate::config::Workflow;
use crate::prompt::adapter::AdapterRunner;
use crate::prompt::dispatch::Sleeper;
use crate::prompt::inbox::{Epitaph, Launcher, deposit_result};
use crate::prompt::tool::{ExecError, ToolCall, ToolExecutor, ToolOutcome};
use crate::prompt::{ChildDispatchRequest, NanoIdGen, SystemClock, child_dispatch};
use crate::template::{GitRunner, RealGit};
use crate::workspace::agent_worktree;
use std::cell::RefCell;
use std::ffi::OsString;
use std::io;
use std::path::Path;
use std::sync::atomic::AtomicBool;
use std::time::Duration;
use tempfile::TempDir;

struct NoAdapter;
impl AdapterRunner for NoAdapter {
    fn run(
        &self,
        _b: &OsString,
        _a: &[&str],
        _s: &[u8],
        _o: &mut dyn FnMut(&[u8]) -> io::Result<()>,
    ) -> io::Result<()> {
        unreachable!("adapter is never reached on the interpreter path")
    }
}
struct NoSleeper;
impl Sleeper for NoSleeper {
    fn sleep(&self, _d: Duration) {
        unreachable!("sleeper is never reached")
    }
}
struct NoTools;
impl ToolExecutor for NoTools {
    fn execute(
        &self,
        _c: ToolCall<'_>,
        _s: &Path,
        _st: &AtomicBool,
    ) -> Result<ToolOutcome, ExecError> {
        unreachable!("tool executor is never reached")
    }
}
/// A [`Launcher`] recording the agent ids it was asked to start (the
/// compactor dispatch's front-door launch), swallowing the spawn.
#[derive(Default)]
struct RecLauncher {
    launched: RefCell<Vec<String>>,
}
impl Launcher for RecLauncher {
    fn launch(&self, _ws: &Path, agent: &str) -> io::Result<()> {
        self.launched.borrow_mut().push(agent.to_string());
        Ok(())
    }
}

/// Owns the deps components so [`Fx::deps`] can borrow them into one
/// [`Deps`] with the unused traits stubbed.
struct Fx {
    git: RealGit,
    clock: SystemClock,
    id: NanoIdGen,
    adapter: NoAdapter,
    sleeper: NoSleeper,
    tools: NoTools,
    launcher: RecLauncher,
    stop: AtomicBool,
    cfg: TempDir,
}
impl Fx {
    fn new() -> Self {
        Self {
            git: RealGit::new(),
            clock: SystemClock,
            id: NanoIdGen,
            adapter: NoAdapter,
            sleeper: NoSleeper,
            tools: NoTools,
            launcher: RecLauncher::default(),
            stop: AtomicBool::new(false),
            cfg: TempDir::new().unwrap(),
        }
    }
    fn deps(&self) -> Deps<'_> {
        Deps {
            adapter: &self.adapter,
            sleeper: &self.sleeper,
            git: &self.git,
            clock: &self.clock,
            id_gen: &self.id,
            tool_executor: &self.tools,
            config_root: self.cfg.path(),
            stop: &self.stop,
            launcher: &self.launcher,
        }
    }
}

/// Fork a `role` child off `parent`, add a committed work file on the
/// child branch, and deposit its result message into the parent's inbox.
/// Returns the child id. `work` is `(path, contents)` committed on the
/// child so the transfer / merge has something to move.
fn returned_child(
    ws: &Path,
    parent: &str,
    role: &str,
    goal: &str,
    work: (&str, &str),
    fx: &Fx,
) -> String {
    let parent_wt = agent_worktree(ws, parent);
    let req = ChildDispatchRequest {
        repo: ws,
        parent_branch: parent,
        parent_worktree: &parent_wt,
        role,
        goal,
        fork_point: None,
    };
    let child = child_dispatch::run(&req, &fx.git, &fx.clock, &fx.id, &fx.launcher).unwrap();
    // Simulate the child doing its work and committing (§2.3).
    let child_wt = agent_worktree(ws, &child);
    let f = child_wt.join(work.0);
    std::fs::create_dir_all(f.parent().unwrap()).unwrap();
    std::fs::write(&f, work.1).unwrap();
    fx.git.run(&child_wt, &["add", "-A"]).unwrap();
    fx.git
        .run(&child_wt, &["commit", "-m", "child work"])
        .unwrap();
    let tip = fx
        .git
        .run_capture(&child_wt, &["rev-parse", "HEAD"])
        .unwrap();
    deposit_result(
        ws,
        parent,
        &child,
        Epitaph::FinalResponse,
        tip.trim(),
        Some("done"),
        &fx.clock,
    )
    .unwrap();
    child
}

fn workflow(yaml: &str) -> Workflow {
    Workflow::parse(yaml, Path::new("workflow.yaml")).unwrap()
}

mod cases;
mod gate;
