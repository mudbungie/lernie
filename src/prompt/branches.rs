//! `.agent/state/branches.json` — runtime tracking of unmerged branches.
//!
//! Each `lernie prompt` invocation appends an entry when it spawns an
//! exchange branch; merge-back (ARCH §2.6, separate v0.2 task) removes
//! the entry; stop handling (§2.9, later) flips the status field. The
//! file is the source for the unmerged-branch-count health metric
//! (§8): count of entries == unmerged branch count, a "ballooning
//! count indicates silent failure somewhere in the merge pipeline."
//!
//! Writes are atomic (temp file in the same directory + rename) so a
//! concurrent reader never sees a half-written file.
//!
//! The file is harness-owned runtime state, not tracked in git (see
//! the template's `.gitignore`). A missing file is treated as an
//! empty map — the first spawn creates it.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::io;
use std::path::{Path, PathBuf};

/// Path of the state file relative to the conversation repo root.
pub const BRANCHES_FILE: &str = ".agent/state/branches.json";

/// Which invariant class a branch belongs to (ARCH §2.3). v0.2 only
/// spawns exchanges; `Invocation` is carried here so v0.4 slots in
/// without a schema bump.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum BranchKind {
    Exchange,
    Invocation,
}

/// Lifecycle state visible on disk. `Open` — spawned but not yet
/// merged. `Stopped` — the branch was stopped (§2.9); still
/// "unmerged" from the merge-pipeline perspective but distinguished
/// from an in-flight branch. Merged branches are *removed* from the
/// map, not re-tagged, so `entries.len()` directly reads the
/// unmerged-branch-count metric.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum BranchStatus {
    Open,
    Stopped,
}

/// Per-branch runtime entry. `stopped_at` is only set when the branch
/// moves to `Stopped`; skipped from the wire when absent so the open
/// case stays minimal.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct BranchEntry {
    pub kind: BranchKind,
    pub spawned_at: String,
    pub base_sha: String,
    pub status: BranchStatus,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stopped_at: Option<String>,
}

/// The on-disk shape of `branches.json`: a map keyed by branch name.
pub type Branches = BTreeMap<String, BranchEntry>;

/// Absolute path of the state file inside `repo`.
pub fn path(repo: &Path) -> PathBuf {
    repo.join(BRANCHES_FILE)
}

/// Read the state file, or return an empty map if it doesn't exist.
/// A malformed file surfaces as [`io::ErrorKind::InvalidData`] so
/// callers don't silently clobber operator-visible corruption.
pub fn load(repo: &Path) -> io::Result<Branches> {
    match std::fs::read(path(repo)) {
        Ok(bytes) => serde_json::from_slice(&bytes).map_err(|e| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                format!("parse branches.json: {e}"),
            )
        }),
        Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(Branches::new()),
        Err(e) => Err(e),
    }
}

/// Write `entries` to the state file atomically: write to a
/// same-directory temp file, then rename into place. A partial
/// failure (crash between write and rename) leaves the pre-call
/// file intact and a leftover `.branches.json.tmp` for the next
/// `save_atomic` to overwrite.
pub fn save_atomic(repo: &Path, entries: &Branches) -> io::Result<()> {
    let target = path(repo);
    let parent = target
        .parent()
        .expect("repo-relative path always has a parent");
    std::fs::create_dir_all(parent)?;
    let tmp = parent.join(".branches.json.tmp");
    let bytes = serde_json::to_vec_pretty(entries).expect("Branches is always serializable");
    std::fs::write(&tmp, &bytes)?;
    std::fs::rename(&tmp, &target)
}

/// Append an open entry for a freshly spawned branch, overwriting any
/// existing entry with the same id (shouldn't happen in practice but
/// makes the call idempotent under retry).
pub fn append_open(
    repo: &Path,
    id: &str,
    kind: BranchKind,
    spawned_at: String,
    base_sha: String,
) -> io::Result<()> {
    let mut entries = load(repo)?;
    entries.insert(
        id.to_string(),
        BranchEntry {
            kind,
            spawned_at,
            base_sha,
            status: BranchStatus::Open,
            stopped_at: None,
        },
    );
    save_atomic(repo, &entries)
}

/// Remove a branch entry — the merge-back path's hook into this
/// module. Consumers should call this after the `--no-ff` merge
/// commit has landed on the parent (§2.6). Idempotent: removing a
/// missing id is a no-op.
pub fn remove(repo: &Path, id: &str) -> io::Result<()> {
    let mut entries = load(repo)?;
    entries.remove(id);
    save_atomic(repo, &entries)
}

/// Number of unmerged branches (`Open` + `Stopped`). The §8 health
/// metric: a rising count with no corresponding removals indicates a
/// silent failure in the merge pipeline.
pub fn unmerged_count(repo: &Path) -> io::Result<usize> {
    Ok(load(repo)?.len())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn append(tmp: &TempDir, id: &str, base_sha: &str) {
        append_open(
            tmp.path(),
            id,
            BranchKind::Exchange,
            "iso".into(),
            base_sha.into(),
        )
        .unwrap();
    }

    #[test]
    fn load_returns_empty_when_file_missing() {
        let tmp = TempDir::new().unwrap();
        assert!(load(tmp.path()).unwrap().is_empty());
        assert_eq!(unmerged_count(tmp.path()).unwrap(), 0);
    }

    #[test]
    fn load_surfaces_parse_error_on_corrupt_file() {
        let tmp = TempDir::new().unwrap();
        let p = path(tmp.path());
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::write(&p, b"{ not json").unwrap();
        let err = load(tmp.path()).unwrap_err();
        assert_eq!(err.kind(), io::ErrorKind::InvalidData);
        assert!(err.to_string().contains("parse branches.json"));
    }

    #[test]
    fn load_surfaces_other_io_errors() {
        // A directory where a file is expected — read() returns
        // IsADirectory / Other depending on platform, but never Ok.
        let tmp = TempDir::new().unwrap();
        let p = path(tmp.path());
        std::fs::create_dir_all(&p).unwrap();
        let err = load(tmp.path()).unwrap_err();
        assert_ne!(err.kind(), io::ErrorKind::NotFound);
    }

    #[test]
    fn save_atomic_writes_and_creates_parent_dir() {
        let tmp = TempDir::new().unwrap();
        let mut entries = Branches::new();
        entries.insert(
            "ex/ts-abcd".into(),
            BranchEntry {
                kind: BranchKind::Exchange,
                spawned_at: "2026-04-22T00:00:00Z".into(),
                base_sha: "deadbeef".into(),
                status: BranchStatus::Open,
                stopped_at: None,
            },
        );
        save_atomic(tmp.path(), &entries).unwrap();
        let back = load(tmp.path()).unwrap();
        assert_eq!(back, entries);
        // No leftover temp file (successful rename consumes it).
        assert!(
            !path(tmp.path())
                .parent()
                .unwrap()
                .join(".branches.json.tmp")
                .exists()
        );
    }

    #[test]
    fn save_atomic_surfaces_rename_failure() {
        // Force the target path to be a directory so rename fails.
        let tmp = TempDir::new().unwrap();
        let p = path(tmp.path());
        std::fs::create_dir_all(&p).unwrap();
        let entries = Branches::new();
        let err = save_atomic(tmp.path(), &entries).unwrap_err();
        // Rename into a directory over an existing directory is an
        // error on every supported platform; the specific kind varies,
        // but it must not silently succeed.
        assert_ne!(err.kind(), io::ErrorKind::NotFound);
    }

    #[test]
    fn append_open_inserts_and_overwrites() {
        let tmp = TempDir::new().unwrap();
        append(&tmp, "ex/ts-a", "sha1");
        append(&tmp, "ex/ts-b", "sha2");
        assert_eq!(unmerged_count(tmp.path()).unwrap(), 2);
        // Same id overwrites, does not duplicate.
        append(&tmp, "ex/ts-a", "sha3");
        let entries = load(tmp.path()).unwrap();
        assert_eq!(entries.len(), 2);
        assert_eq!(entries["ex/ts-a"].base_sha, "sha3");
    }

    #[test]
    fn remove_is_idempotent() {
        let tmp = TempDir::new().unwrap();
        append(&tmp, "ex/ts-a", "sha");
        remove(tmp.path(), "ex/ts-a").unwrap();
        assert_eq!(unmerged_count(tmp.path()).unwrap(), 0);
        // Removing a missing id is a no-op.
        remove(tmp.path(), "ex/ts-a").unwrap();
        assert_eq!(unmerged_count(tmp.path()).unwrap(), 0);
    }

    #[test]
    fn stopped_entry_round_trips() {
        let tmp = TempDir::new().unwrap();
        let mut entries = Branches::new();
        entries.insert(
            "inv/x/y".into(),
            BranchEntry {
                kind: BranchKind::Invocation,
                spawned_at: "iso-s".into(),
                base_sha: "sha".into(),
                status: BranchStatus::Stopped,
                stopped_at: Some("iso-z".into()),
            },
        );
        save_atomic(tmp.path(), &entries).unwrap();
        let back = load(tmp.path()).unwrap();
        assert_eq!(back["inv/x/y"].status, BranchStatus::Stopped);
        assert_eq!(back["inv/x/y"].stopped_at.as_deref(), Some("iso-z"));
        // `stopped_at` omitted for Open entries: ensure wire shape.
        let mut open = Branches::new();
        open.insert(
            "ex/z".into(),
            BranchEntry {
                kind: BranchKind::Exchange,
                spawned_at: "iso".into(),
                base_sha: "sha".into(),
                status: BranchStatus::Open,
                stopped_at: None,
            },
        );
        let wire = serde_json::to_string(&open).unwrap();
        assert!(!wire.contains("stopped_at"), "wire: {wire}");
    }
}
