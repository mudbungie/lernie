//! Resolving an experiment (ARCH §9.3).
//!
//! An experiment is a `workflow.yaml` variant under `experiments/<name>/`
//! — a config diff, no code changes. `--config <name>` names the
//! subdirectory; the runner resolves it to that directory's
//! `workflow.yaml`, which the agent invocation is handed as the config
//! to run under. A missing variant is a loud failure, never a silent
//! fallback (`docs/PRINCIPLES.md` Decline illegal operations).

use std::path::{Path, PathBuf};

/// A resolved experiment: its name and the `workflow.yaml` that defines
/// it.
#[derive(Clone, Debug, PartialEq)]
pub struct Experiment {
    pub name: String,
    pub workflow: PathBuf,
}

/// Every way [`resolve`] can fail.
#[derive(Debug, thiserror::Error)]
pub enum ExperimentError {
    #[error("experiment {name:?} has no workflow.yaml at {path}")]
    Missing { name: String, path: PathBuf },
}

/// Resolve `config` to `<experiments_root>/<config>/workflow.yaml`,
/// erroring if that file does not exist.
pub fn resolve(config: &str, experiments_root: &Path) -> Result<Experiment, ExperimentError> {
    let workflow = experiments_root.join(config).join("workflow.yaml");
    if workflow.is_file() {
        Ok(Experiment {
            name: config.to_string(),
            workflow,
        })
    } else {
        Err(ExperimentError::Missing {
            name: config.to_string(),
            path: workflow,
        })
    }
}
