//! Loading the evaluation task suite (ARCH §9.1).
//!
//! The suite is data under a directory (`tests/suite/`): one YAML file
//! per failure category, each a list under `tasks:`. This loader reads
//! every `*.yaml` in the directory (sorted for determinism) and returns
//! the flattened task list. The schema mirrors `tests/suite/README.md`
//! and the well-formedness gate in `tests/suite.rs`; this crate is the
//! runner that consumes it (§9.3 / v0.10).

use std::path::{Path, PathBuf};

/// One suite task: the goal handed to the agent, optional shell `setup`
/// seeding the workspace, and a machine-checkable `check` (exit 0 = pass).
#[derive(Clone, Debug, serde::Deserialize)]
pub struct Task {
    pub id: String,
    pub categories: Vec<String>,
    pub prompt: String,
    #[serde(default)]
    pub setup: Option<String>,
    pub check: String,
}

#[derive(serde::Deserialize)]
struct SuiteFile {
    tasks: Vec<Task>,
}

/// Every way [`load`] can fail.
#[derive(Debug, thiserror::Error)]
pub enum SuiteError {
    #[error("read suite directory {path}: {source}")]
    Dir {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("read {path}: {source}")]
    Read {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("parse {path}: {source}")]
    Parse {
        path: PathBuf,
        #[source]
        source: serde_yaml_ng::Error,
    },
}

/// Load and flatten every `*.yaml` task file in `dir`, sorted by filename.
pub fn load(dir: &Path) -> Result<Vec<Task>, SuiteError> {
    let mut files: Vec<PathBuf> = std::fs::read_dir(dir)
        .map_err(|source| SuiteError::Dir {
            path: dir.to_path_buf(),
            source,
        })?
        .filter_map(Result::ok)
        .map(|e| e.path())
        .filter(|p| p.extension().is_some_and(|x| x == "yaml"))
        .collect();
    files.sort();

    let mut tasks = Vec::new();
    for path in files {
        let text = std::fs::read_to_string(&path).map_err(|source| SuiteError::Read {
            path: path.clone(),
            source,
        })?;
        let file: SuiteFile =
            serde_yaml_ng::from_str(&text).map_err(|source| SuiteError::Parse {
                path: path.clone(),
                source,
            })?;
        tasks.extend(file.tasks);
    }
    Ok(tasks)
}
