//! Descriptions-always producer (ARCH §3.3 *Descriptions-always
//! population*). A single step of the creation routine
//! ([`super::scaffold`]) snapshots the data-root pools into the
//! worktree's `descriptions/**` so every agent branch inherits them via
//! git (§2.2, §2.3) and context assembly intersects a role's declared
//! tools against a committed, immutable schema set rather than re-reading
//! mutable data-root state (§2.10, §5.1) — the committed form of
//! "fork is the freeze" (§2.2).
//!
//! **One mechanism over two artifact kinds, not two producers:** the
//! same pass copies every available tool's JSON schema
//! (`<data-root>/tools/<name>.json` → `descriptions/tools/<name>.json`,
//! verbatim) and every available skill's `SKILL.md` frontmatter
//! (`<data-root>/skills/<name>/SKILL.md` → `descriptions/skills/<name>.md`).
//!
//! The data-root pools are the single source of truth for *what this
//! install provides*; the committed `descriptions/**` snapshot is the
//! single source of truth for *what agents forked from this config are
//! pinned to see* — distinct facts, so the copy is a snapshot, not a
//! mirror (`docs/PRINCIPLES.md`, single source of truth). An empty (or
//! absent) pool yields an empty descriptions tree, which the composer
//! (`crate::prompt::dispatch::tools`) reads as an empty toolset.

use crate::skill;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

/// Worktree-relative root under which the snapshot lands. Kept in step
/// with the composer's `descriptions/tools` and with ARCH §2.2's layout.
pub const DESCRIPTIONS_DIR: &str = "descriptions";
/// Pool + descriptions subdir holding tool JSON schemas.
pub const TOOLS_SUBDIR: &str = "tools";
/// Pool + descriptions subdir holding skill frontmatter.
pub const SKILLS_SUBDIR: &str = "skills";
/// The frontmatter-bearing file inside each skill directory.
pub const SKILL_MANIFEST: &str = "SKILL.md";
/// Extension of a tool schema in the pool (copied verbatim).
const JSON_EXT: &str = "json";

/// Why [`snapshot`] could not complete.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("descriptions I/O at {path}: {source}")]
    Io {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
    #[error("skill {name}: {} has no YAML frontmatter block", SKILL_MANIFEST)]
    NoFrontmatter { name: String },
}

fn io_err(path: &Path, source: io::Error) -> Error {
    Error::Io {
        path: path.to_path_buf(),
        source,
    }
}

/// Snapshot the data-root tool schemas and skill frontmatter into
/// `<worktree>/descriptions/{tools,skills}/`. Idempotent overwrite; a
/// missing pool directory is not an error (empty pool → empty
/// descriptions tree, §3.3).
pub fn snapshot(data_root: &Path, worktree: &Path) -> Result<(), Error> {
    copy_tool_schemas(&data_root.join(TOOLS_SUBDIR), worktree)?;
    copy_skill_frontmatter(&data_root.join(SKILLS_SUBDIR), worktree)?;
    Ok(())
}

/// Copy every `<pool>/<name>.json` verbatim to
/// `<worktree>/descriptions/tools/<name>.json` (§3.3 point 2).
fn copy_tool_schemas(pool: &Path, worktree: &Path) -> Result<(), Error> {
    let dest = worktree.join(DESCRIPTIONS_DIR).join(TOOLS_SUBDIR);
    for entry in read_pool(pool)? {
        let path = entry.path();
        if !path.is_file() || path.extension().and_then(|e| e.to_str()) != Some(JSON_EXT) {
            continue;
        }
        ensure_dir(&dest)?;
        let target = dest.join(entry.file_name());
        fs::copy(&path, &target).map_err(|e| io_err(&target, e))?;
    }
    Ok(())
}

/// Extract each `<pool>/<name>/SKILL.md`'s frontmatter and write it to
/// `<worktree>/descriptions/skills/<name>.md` (§3.3 *Description-always*).
fn copy_skill_frontmatter(pool: &Path, worktree: &Path) -> Result<(), Error> {
    let dest = worktree.join(DESCRIPTIONS_DIR).join(SKILLS_SUBDIR);
    for entry in read_pool(pool)? {
        let dir = entry.path();
        if !dir.is_dir() {
            continue;
        }
        let manifest = dir.join(SKILL_MANIFEST);
        let raw = match fs::read_to_string(&manifest) {
            Ok(s) => s,
            // A directory with no SKILL.md is not an available skill.
            Err(e) if e.kind() == io::ErrorKind::NotFound => continue,
            Err(e) => return Err(io_err(&manifest, e)),
        };
        let name = entry.file_name().to_string_lossy().into_owned();
        let body =
            skill::frontmatter_yaml(&raw).ok_or(Error::NoFrontmatter { name: name.clone() })?;
        ensure_dir(&dest)?;
        let target = dest.join(format!("{name}.md"));
        fs::write(&target, body).map_err(|e| io_err(&target, e))?;
    }
    Ok(())
}

/// Read a pool directory into a name-sorted vec of entries; a missing
/// pool is an empty pool (§3.3), never an error. Sorting makes the
/// snapshot order deterministic. Individual entries that fail to
/// enumerate (a transient per-entry `read_dir` error) are skipped via
/// `flatten` — the snapshot is a set of independent files, so a dropped
/// entry degrades to compose dropping that tool, never a corrupt tree.
fn read_pool(pool: &Path) -> Result<Vec<fs::DirEntry>, Error> {
    let iter = match fs::read_dir(pool) {
        Ok(it) => it,
        Err(e) if e.kind() == io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(e) => return Err(io_err(pool, e)),
    };
    let mut entries: Vec<fs::DirEntry> = iter.flatten().collect();
    entries.sort_by_key(|e| e.file_name());
    Ok(entries)
}

fn ensure_dir(dir: &Path) -> Result<(), Error> {
    fs::create_dir_all(dir).map_err(|e| io_err(dir, e))
}

#[cfg(test)]
mod tests;
