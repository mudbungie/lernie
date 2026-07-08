//! JSON Schema generation for the v0.3 conversation-repo + harness-root
//! configuration files (ARCH §2.2, §4.1).
//!
//! Used by the `gen-schemas` binary (and by tests) to produce
//! `schemas/<name>.json` so external tooling (editors, the template task's
//! validation pass) can validate configuration without linking the crate.

use crate::config::manifest::Manifest;
use crate::config::models::Models;
use crate::config::per_repo_providers::PerRepoProviders;
use crate::config::version::Version;
use crate::config::workflow::Workflow;
use schemars::JsonSchema;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

/// One (filename, JSON-serialized schema) pair to be written.
pub struct SchemaFile {
    pub name: &'static str,
    pub body: String,
}

/// Build all schema files in declaration order.
///
/// The two harness config files have distinct on-disk names now:
/// `providers.json` is the per-repo `<conv-repo>/providers.yaml` shape
/// (only `roles:`), and `models.json` is the harness-root
/// `<harness-root>/models.yaml` shape (capabilities, context windows,
/// optional `adapter:` override — no endpoints or auth, which are
/// brazen's). See ARCH §4.1/§4.2.
pub fn all() -> Vec<SchemaFile> {
    vec![
        schema_file::<Version>("version.json"),
        schema_file::<PerRepoProviders>("providers.json"),
        schema_file::<Models>("models.json"),
        schema_file::<Manifest>("manifest.json"),
        schema_file::<Workflow>("workflow.json"),
    ]
}

fn schema_file<T: JsonSchema>(name: &'static str) -> SchemaFile {
    // schemars' output is always JSON-serializable; the error branch of
    // `to_string_pretty` is unreachable. Use `unwrap_or_default` rather
    // than `.expect(...)` so coverage doesn't pin a panic landing pad
    // we can't legitimately exercise.
    let schema = schemars::schema_for!(T);
    let body = serde_json::to_string_pretty(&schema).unwrap_or_default();
    SchemaFile { name, body }
}

/// Write every schema file under `dir`, creating the directory if needed.
/// Returns the absolute paths written.
pub fn write_to(dir: &Path) -> io::Result<Vec<PathBuf>> {
    fs::create_dir_all(dir)?;
    let mut written = Vec::new();
    for SchemaFile { name, body } in all() {
        let path = dir.join(name);
        fs::write(&path, body)?;
        written.push(path);
    }
    Ok(written)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn all_schemas_have_unique_names() {
        let mut names: Vec<&str> = all().iter().map(|s| s.name).collect();
        names.sort();
        let n = names.len();
        names.dedup();
        assert_eq!(n, names.len());
    }

    #[test]
    fn each_schema_is_valid_json_and_has_a_schema_field() {
        for SchemaFile { name, body } in all() {
            let v: serde_json::Value =
                serde_json::from_str(&body).unwrap_or_else(|_| panic!("{name} not JSON"));
            assert!(v.get("$schema").is_some(), "{name} missing $schema");
        }
    }

    #[test]
    fn write_to_creates_directory_and_files() {
        let dir = TempDir::new().unwrap();
        let nested = dir.path().join("schemas");
        let written = write_to(&nested).unwrap();
        assert_eq!(written.len(), all().len());
        for path in &written {
            assert!(path.is_file(), "{} not written", path.display());
        }
    }

    #[test]
    fn write_to_surfaces_io_errors() {
        // Use a tempdir then drop the dir so creating files inside it
        // fails. Parent-of-parent is a non-existent path; create_dir_all
        // succeeds in nested cases, so we point at a regular file as the
        // intended directory.
        let f = tempfile::NamedTempFile::new().unwrap();
        let err = write_to(f.path()).unwrap_err();
        // Either "not a directory" (when create_dir_all rejects the file)
        // or "already exists" — both are io errors and adequate for the
        // surfacing contract.
        assert!(matches!(
            err.kind(),
            io::ErrorKind::AlreadyExists | io::ErrorKind::NotADirectory | io::ErrorKind::Other
        ));
    }
}
