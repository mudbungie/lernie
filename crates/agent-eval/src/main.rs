//! The `agent-eval` binary (ARCH §9.3, v0.10).
//!
//! `agent-eval --config <experiment> --suite <suite> --runs N` executes
//! the experiment (`experiments/<config>/workflow.yaml`) against the
//! suite (a task directory) N times per task and prints per-task /
//! per-category pass@1 and pass@5 (ARCH §9.1).
//!
//! `--agent` names the external harness-driver the runner invokes per run
//! (the injectable agent seam, §9.3). It is **required with no default**:
//! which driver runs the agent under test is an experiment-defining
//! input, so it is named at every invocation. The shipped driver is
//! `lernie-eval-agent` (`crates/lernie-eval-agent`); any program
//! honouring the contract in the repo README ("Run the suite") works.
//! Clap rejects a missing `--agent` up front rather than letting the
//! runner die on a failed spawn per task.
//!
//! `--bundle-dir` archives failing runs for triage via `lernie bundle`
//! (§9.2). This file is thin wiring over the library (`lib.rs`); all
//! logic and its coverage live there.

use agent_eval::agent::{CommandAgent, CommandBundler};
use agent_eval::runner::{self, EvalConfig};
use agent_eval::{experiment, report, suite};
use clap::Parser;
use std::path::PathBuf;
use std::process::ExitCode;

#[derive(Parser)]
#[command(
    name = "agent-eval",
    about = "lernie evaluation runner (ARCH §9.3)",
    version
)]
struct Cli {
    /// Experiment name: `experiments/<config>/workflow.yaml` (§9.3).
    #[arg(long)]
    config: String,
    /// Path to the task-suite directory (e.g. `tests/suite`, §9.1).
    #[arg(long)]
    suite: PathBuf,
    /// Runs per task (N ≥ 5 per §9.1).
    #[arg(long)]
    runs: usize,
    /// Directory holding the experiments (default `experiments`).
    #[arg(long, default_value = "experiments")]
    experiments_dir: PathBuf,
    /// External harness-driver invoked per run (the agent seam, §9.3).
    /// Required, no default — the driver is an experiment-defining
    /// input. The shipped one is `lernie-eval-agent`; the contract any
    /// driver must honour is in the repo README, "Run the suite".
    #[arg(long)]
    agent: String,
    /// Archive failing runs here for triage via `lernie bundle` (§9.2).
    #[arg(long)]
    bundle_dir: Option<PathBuf>,
    /// The `lernie` binary used to bundle failing runs (§9.2).
    #[arg(long, default_value = "lernie")]
    lernie: String,
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    match run(cli) {
        Ok(text) => {
            print!("{text}");
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("agent-eval: {e}");
            ExitCode::FAILURE
        }
    }
}

fn run(cli: Cli) -> Result<String, Box<dyn std::error::Error>> {
    let experiment = experiment::resolve(&cli.config, &cli.experiments_dir)?;
    let tasks = suite::load(&cli.suite)?;
    let base = tempfile::tempdir()?;

    let agent = CommandAgent::new(cli.agent);
    let bundler = CommandBundler::new(cli.lernie);
    let cfg = EvalConfig {
        runs: cli.runs,
        bundle_dir: cli.bundle_dir,
    };
    let metrics = runner::evaluate(
        &tasks,
        &experiment,
        base.path(),
        &agent,
        Some(&bundler),
        &cfg,
    )?;
    Ok(report::render(&experiment.name, &metrics))
}
