//! The evaluation orchestrator (ARCH §9.3): experiment × suite × N.
//!
//! For each task, for each of N runs, the runner seeds a fresh isolated
//! workspace (its own `LERNIE_HOME` and working directory under `base`),
//! runs the task `setup` (shell), invokes the agent through the [`Agent`]
//! seam, then runs the task `check` (shell) — **exit 0 is the sole pass
//! signal** (§9.1), so success is observable state, never the agent's own
//! claim. Setup, agent, and check share one working directory, as the
//! suite format specifies (`tests/suite/README.md`).
//!
//! A failing run is optionally archived for triage (§9.2): when a bundle
//! directory is configured and the agent disclosed a [`BundleTarget`],
//! the run's subtree is bundled through the [`Bundler`] seam.
//!
//! Setup, agent invocation, and check are the only impure edges; the
//! aggregation is [`stats::compute`]. Injecting the agent (and bundler)
//! is what lets the whole path run in tests without live model traffic.

use crate::agent::{Agent, Bundler, Dispatch};
use crate::experiment::Experiment;
use crate::stats::{self, Metrics, TaskResult};
use crate::suite::Task;
use std::io;
use std::path::{Path, PathBuf};
use std::process::Command;

/// Knobs for one evaluation.
pub struct EvalConfig {
    /// Runs per task (N ≥ 5 per §9.1).
    pub runs: usize,
    /// When set, failing runs are bundled here for triage (§9.2).
    pub bundle_dir: Option<PathBuf>,
}

/// Run the whole evaluation and aggregate the metrics.
pub fn evaluate(
    tasks: &[Task],
    experiment: &Experiment,
    base: &Path,
    agent: &dyn Agent,
    bundler: Option<&dyn Bundler>,
    cfg: &EvalConfig,
) -> io::Result<Metrics> {
    let mut results = Vec::with_capacity(tasks.len());
    for task in tasks {
        let mut outcomes = Vec::with_capacity(cfg.runs);
        for run in 0..cfg.runs {
            outcomes.push(run_once(task, experiment, base, agent, bundler, cfg, run)?);
        }
        results.push(TaskResult {
            id: task.id.clone(),
            categories: task.categories.clone(),
            outcomes,
        });
    }
    Ok(stats::compute(&results))
}

/// One (task, run): seed, setup, agent, check → pass/fail.
fn run_once(
    task: &Task,
    experiment: &Experiment,
    base: &Path,
    agent: &dyn Agent,
    bundler: Option<&dyn Bundler>,
    cfg: &EvalConfig,
    run: usize,
) -> io::Result<bool> {
    let dir = base.join(&task.id).join(run.to_string());
    let home = dir.join("home");
    let work = dir.join("work");
    std::fs::create_dir_all(&home)?;
    std::fs::create_dir_all(&work)?;

    // A failed `setup` means the run never got a fair start: count it a
    // fail without invoking the agent or the check.
    if let Some(setup) = &task.setup
        && !run_shell(setup, &work)?
    {
        return Ok(false);
    }

    let outcome = agent.dispatch(&Dispatch {
        prompt: &task.prompt,
        workdir: &work,
        lernie_home: &home,
        experiment: &experiment.workflow,
    })?;

    let pass = run_shell(&task.check, &work)?;
    if !pass
        && let (Some(dest_root), Some(b), Some(target)) =
            (&cfg.bundle_dir, bundler, &outcome.target)
    {
        let dest = dest_root.join(format!("{}-{run}", task.id));
        b.bundle(target, &dest)?;
    }
    Ok(pass)
}

/// Run a shell script in `cwd`; `true` iff it exits 0.
fn run_shell(script: &str, cwd: &Path) -> io::Result<bool> {
    Ok(Command::new("sh")
        .arg("-c")
        .arg(script)
        .current_dir(cwd)
        .status()?
        .success())
}
