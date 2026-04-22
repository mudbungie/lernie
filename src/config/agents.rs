//! `.agent/agents.yaml` — agent role definitions per ARCH §4.3.

use crate::config::error::LoadError;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

/// Top-level `agents.yaml` shape.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct Agents {
    pub agents: BTreeMap<String, AgentRole>,
}

/// One named agent role: which model to drive it with and where to find its
/// system prompt. `system_prompt` is interpreted relative to
/// `.agent/system/`.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct AgentRole {
    pub model: String,
    pub system_prompt: PathBuf,
}

impl Agents {
    /// Read and parse `agents.yaml` at `path`. Cross-file references to
    /// `models` are validated separately via [`crate::config::cross`].
    pub fn load(path: &Path) -> Result<Self, LoadError> {
        let raw = fs::read_to_string(path).map_err(|source| LoadError::Io {
            path: path.to_path_buf(),
            source,
        })?;
        let parsed: Self = serde_yaml_ng::from_str(&raw).map_err(|source| LoadError::Yaml {
            path: path.to_path_buf(),
            source,
        })?;
        parsed.validate(path)?;
        Ok(parsed)
    }

    fn validate(&self, path: &Path) -> Result<(), LoadError> {
        for (name, role) in &self.agents {
            if role.system_prompt.is_absolute() {
                return Err(LoadError::Invalid {
                    path: path.to_path_buf(),
                    key: format!("agents.{name}.system_prompt"),
                    message: format!(
                        "must be a relative path under .agent/system/, got {}",
                        role.system_prompt.display()
                    ),
                });
            }
            if role
                .system_prompt
                .components()
                .any(|c| matches!(c, std::path::Component::ParentDir))
            {
                return Err(LoadError::Invalid {
                    path: path.to_path_buf(),
                    key: format!("agents.{name}.system_prompt"),
                    message: format!("may not contain '..', got {}", role.system_prompt.display()),
                });
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    fn write_yaml(s: &str) -> NamedTempFile {
        let mut f = NamedTempFile::new().unwrap();
        f.write_all(s.as_bytes()).unwrap();
        f
    }

    #[test]
    fn parses_arch_example() {
        let f = write_yaml(
            r#"
agents:
  worker:
    model: claude-sonnet-4-7
    system_prompt: prompts/worker.md
  compactor:
    model: claude-haiku-4-5
    system_prompt: prompts/compactor.md
"#,
        );
        let a = Agents::load(f.path()).unwrap();
        assert_eq!(a.agents.len(), 2);
        assert_eq!(a.agents["worker"].model, "claude-sonnet-4-7");
        assert_eq!(
            a.agents["compactor"].system_prompt,
            PathBuf::from("prompts/compactor.md")
        );
    }

    #[test]
    fn rejects_absolute_system_prompt() {
        let f = write_yaml(
            r#"
agents:
  worker:
    model: m
    system_prompt: /etc/passwd
"#,
        );
        let err = Agents::load(f.path()).unwrap_err();
        match err {
            LoadError::Invalid { key, message, .. } => {
                assert_eq!(key, "agents.worker.system_prompt");
                assert!(message.contains("relative"), "got: {message}");
            }
            other => panic!("expected Invalid, got {other:?}"),
        }
    }

    #[test]
    fn rejects_parent_dir_traversal() {
        let f = write_yaml(
            r#"
agents:
  worker:
    model: m
    system_prompt: ../escape.md
"#,
        );
        let err = Agents::load(f.path()).unwrap_err();
        match err {
            LoadError::Invalid { message, .. } => {
                assert!(message.contains(".."), "got: {message}");
            }
            other => panic!("expected Invalid, got {other:?}"),
        }
    }

    #[test]
    fn surfaces_io_and_yaml_errors() {
        assert!(matches!(
            Agents::load(Path::new("/no/such/agents.yaml")),
            Err(LoadError::Io { .. })
        ));
        let f = write_yaml("agents: [not, a, map]");
        assert!(matches!(
            Agents::load(f.path()),
            Err(LoadError::Yaml { .. })
        ));
    }
}
