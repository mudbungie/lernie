//! The evaluation record (bl-36fa, ARCH §9.3): one JSON document per
//! evaluation — the reproducibility inputs plus every task's per-run
//! quality and efficiency observations. Written by `agent-eval run
//! --record`, consumed by `agent-eval compare` as the baseline or the
//! candidate side. Observed model/provider sets are derived from the
//! runs at read time, never stored beside them (PRINCIPLES "Single
//! source of truth").

use crate::metrics::RunMetrics;
use crate::stats::TaskResult;
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use std::io;
use std::path::{Path, PathBuf};

/// The reproducibility inputs of one evaluation (ARCH §9.3): what would
/// have to be held equal for another run — or another harness — to be
/// comparable. Every probed field is `Option`: unknown is reported as
/// unknown, never guessed.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Provenance {
    /// Experiment name (`experiments/<name>/workflow.yaml`).
    pub experiment: String,
    /// The resolved `workflow.yaml` path the experiment named.
    pub workflow: String,
    /// The suite directory as given.
    pub suite: String,
    /// Git revision of the suite directory (`+dirty` when its tree has
    /// uncommitted changes); `None` when it is not a git checkout.
    pub suite_revision: Option<String>,
    /// Starting fixture identity: sha256 over the suite's task files —
    /// the `setup`/`check` scripts *are* the starting fixture, so this
    /// digest identifies it even outside git. `None` when unreadable.
    pub fixture_digest: Option<String>,
    /// The driver command (`--agent`) verbatim.
    pub driver: String,
    /// The driver's `--version` line; `None` when it reported none.
    pub driver_version: Option<String>,
    /// Runs per task (N).
    pub runs_per_task: usize,
}

/// One task's runs.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TaskRecord {
    pub id: String,
    pub categories: Vec<String>,
    pub runs: Vec<RunRecord>,
}

/// One (task, run) observation. `wall_ms` is the **outer wall time** —
/// the runner's own measurement around the driver invocation, as
/// opposed to the workspace's inner per-step spans (ARCH §8); a run
/// whose setup failed never invoked the driver and carries 0. `metrics`
/// is `None` when the driver disclosed no workspace — missing, not
/// zero.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct RunRecord {
    pub pass: bool,
    pub wall_ms: u64,
    pub metrics: Option<RunMetrics>,
}

/// The whole document: provenance plus per-task observations.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Record {
    pub provenance: Provenance,
    pub tasks: Vec<TaskRecord>,
}

/// Every way [`Record::load`] can fail.
#[derive(Debug, thiserror::Error)]
pub enum RecordError {
    #[error("read record {path}: {source}")]
    Read {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
    #[error("parse record {path}: {source}")]
    Parse {
        path: PathBuf,
        #[source]
        source: serde_json::Error,
    },
}

impl Record {
    /// Write the record as pretty JSON.
    pub fn save(&self, path: &Path) -> io::Result<()> {
        let mut text = serde_json::to_string_pretty(self).map_err(io::Error::other)?;
        text.push('\n');
        std::fs::write(path, text)
    }

    /// Read a record back.
    pub fn load(path: &Path) -> Result<Record, RecordError> {
        let text = std::fs::read_to_string(path).map_err(|source| RecordError::Read {
            path: path.to_path_buf(),
            source,
        })?;
        serde_json::from_str(&text).map_err(|source| RecordError::Parse {
            path: path.to_path_buf(),
            source,
        })
    }

    /// Project the pass/fail outcomes for [`crate::stats::compute`] —
    /// quality stays exactly the §9.1 pass@1/pass@5 pipeline.
    pub fn task_results(&self) -> Vec<TaskResult> {
        task_results(&self.tasks)
    }

    /// The sorted union of model ids observed across all runs.
    pub fn observed_models(&self) -> Vec<String> {
        self.observed(|m| &m.models)
    }

    /// The sorted union of provider rows observed across all runs.
    pub fn observed_providers(&self) -> Vec<String> {
        self.observed(|m| &m.providers)
    }

    fn observed(&self, pick: fn(&RunMetrics) -> &Vec<String>) -> Vec<String> {
        let mut set = BTreeSet::new();
        for task in &self.tasks {
            for run in &task.runs {
                if let Some(m) = &run.metrics {
                    set.extend(pick(m).iter().cloned());
                }
            }
        }
        set.into_iter().collect()
    }
}

/// [`Record::task_results`] over any task slice (the comparison uses it
/// on matched subsets).
pub fn task_results(tasks: &[TaskRecord]) -> Vec<TaskResult> {
    tasks
        .iter()
        .map(|t| TaskResult {
            id: t.id.clone(),
            categories: t.categories.clone(),
            outcomes: t.runs.iter().map(|r| r.pass).collect(),
        })
        .collect()
}
