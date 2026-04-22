//! `.agent/version` — the schema version of a conversation repo.
//!
//! Per ARCH §10, this is a bare integer. The file's content is the integer
//! and nothing else (trailing whitespace tolerated).

use crate::config::error::LoadError;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

/// The on-disk schema version. v0.1 templates start at 1.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(transparent)]
pub struct Version(pub u32);

impl Version {
    /// Read and parse `.agent/version` at `path`.
    pub fn load(path: &Path) -> Result<Self, LoadError> {
        let raw = fs::read_to_string(path).map_err(|source| LoadError::Io {
            path: path.to_path_buf(),
            source,
        })?;
        let trimmed = raw.trim();
        let parsed: u32 = trimmed.parse().map_err(|_| LoadError::Invalid {
            path: path.to_path_buf(),
            key: ".".into(),
            message: format!("expected an unsigned integer, got {trimmed:?}"),
        })?;
        Ok(Version(parsed))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    fn write_temp(contents: &str) -> NamedTempFile {
        let mut f = NamedTempFile::new().unwrap();
        f.write_all(contents.as_bytes()).unwrap();
        f
    }

    #[test]
    fn parses_a_bare_integer() {
        let f = write_temp("1\n");
        assert_eq!(Version::load(f.path()).unwrap(), Version(1));
    }

    #[test]
    fn tolerates_trailing_whitespace() {
        let f = write_temp("  42  \n\n");
        assert_eq!(Version::load(f.path()).unwrap(), Version(42));
    }

    #[test]
    fn rejects_non_integer_content() {
        let f = write_temp("v1\n");
        let err = Version::load(f.path()).unwrap_err();
        match err {
            LoadError::Invalid { message, .. } => {
                assert!(message.contains("\"v1\""), "got: {message}");
            }
            other => panic!("expected Invalid, got {other:?}"),
        }
    }

    #[test]
    fn surfaces_io_errors() {
        let path = Path::new("/nonexistent/path/version");
        let err = Version::load(path).unwrap_err();
        assert!(matches!(err, LoadError::Io { .. }));
    }
}
