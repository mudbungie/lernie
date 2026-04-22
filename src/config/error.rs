//! Error and warning types surfaced by the configuration loaders.

use std::path::PathBuf;
use thiserror::Error;

/// Why a config file failed to load. Every variant names the file and the
/// offending key (when known) so error messages can be acted on without
/// reading the file again.
#[derive(Debug, Error)]
pub enum LoadError {
    /// Reading the file from disk failed (missing, permission denied, etc.).
    #[error("config: read {path}: {source}")]
    Io {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    /// The YAML did not parse.
    #[error("config: parse {path}: {source}")]
    Yaml {
        path: PathBuf,
        #[source]
        source: serde_yaml_ng::Error,
    },

    /// The file parsed but was structurally invalid (e.g. unknown enum value,
    /// wrong arity for an action). `key` identifies the offending location.
    #[error("config: invalid {path} at {key}: {message}")]
    Invalid {
        path: PathBuf,
        key: String,
        message: String,
    },

    /// A reference across files did not resolve (e.g. `agents.worker.model`
    /// names a model not declared in `providers.yaml`).
    #[error("config: unresolved reference at {key}: {message}")]
    UnresolvedRef { key: String, message: String },
}

/// A non-fatal observation surfaced by the loaders. The file parsed and is
/// usable, but the caller should know.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Warning {
    pub file: PathBuf,
    pub key: String,
    pub message: String,
}

impl Warning {
    pub fn new(
        file: impl Into<PathBuf>,
        key: impl Into<String>,
        message: impl Into<String>,
    ) -> Self {
        Self {
            file: file.into(),
            key: key.into(),
            message: message.into(),
        }
    }
}
