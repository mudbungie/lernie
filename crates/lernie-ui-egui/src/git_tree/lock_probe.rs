//! Executor-lock probe for the §3.5 `live` classification (§2.11).
//!
//! The executor lock is the agent's inbox-directory `flock`
//! (`src/prompt/inbox/lock.rs`), acquired at step-loop start and held for
//! the whole loop. Its holder is *the* driver, so this probe answers the
//! one question `live` needs — "is anyone driving `<agent>`?" — by scanning
//! `/proc/<pid>/fd/*` for a symlink to the agent's inbox directory. This is
//! the identical scan the harness's own `stop::discover` runs to find the
//! pid to signal (`src/prompt/stop/discover`).
//!
//! **No access-mode filter.** The lock fd is opened read-only
//! (`File::open`), so a "writer" test would reject the very fd we seek; the
//! inbox directory is namespaced per agent (§2.11), so any process holding
//! it open is that agent's executor. This is exactly why the lock probe is
//! a *separate* observation from the `response.json` writer probe
//! ([`super::fd_probe`]): the lock is *is-anyone-driving*, the open
//! `response.json` is *is-a-model-call-in-flight* (§2.11 "two
//! observations", §4.4).
//!
//! Duplicated here (not shared via a crate dep) so the frontend stays
//! decoupled from the harness binary (§3.5). Linux only; `/proc` is the
//! verified platform (§2.9).

use std::path::{Path, PathBuf};

/// "Does a process hold `inbox_dir`'s fd open?" — the executor lock
/// (§2.11). Injected so the classifier is testable without a live driver.
pub(super) trait LockProbe {
    fn lock_held(&self, inbox_dir: &Path) -> bool;
}

/// Production probe backed by `/proc`.
pub(super) struct ProcFsLockProbe {
    proc_root: PathBuf,
}

impl Default for ProcFsLockProbe {
    fn default() -> Self {
        Self {
            proc_root: PathBuf::from("/proc"),
        }
    }
}

impl ProcFsLockProbe {
    #[cfg(test)]
    fn with_root(proc_root: PathBuf) -> Self {
        Self { proc_root }
    }

    fn scan(&self, inbox_dir: &Path) -> bool {
        // Canonicalize so the symlink-target compare is exact. A fresh
        // agent whose inbox dir does not exist yet canonicalizes to
        // NotFound → no holder.
        let Ok(target) = std::fs::canonicalize(inbox_dir) else {
            return false;
        };
        let Ok(entries) = std::fs::read_dir(&self.proc_root) else {
            return false;
        };
        for entry in entries.flatten() {
            // Only numeric pid dirs; skip `/proc/acpi`, `/proc/self`, etc.
            if entry
                .file_name()
                .to_str()
                .and_then(|s| s.parse::<u32>().ok())
                .is_none()
            {
                continue;
            }
            if pid_holds(&entry.path(), &target) {
                return true;
            }
        }
        false
    }
}

impl LockProbe for ProcFsLockProbe {
    fn lock_held(&self, inbox_dir: &Path) -> bool {
        self.scan(inbox_dir)
    }
}

/// Any fd under `<proc_pid>/fd/` resolving to `target`. No access-mode
/// filter — the lock fd is opened read-only, so matching a held directory
/// fd is the whole test (mirrors `stop::discover::pid_holds`). Pids we
/// cannot introspect (other uid, raced teardown) simply do not match.
fn pid_holds(proc_pid: &Path, target: &Path) -> bool {
    let Ok(entries) = std::fs::read_dir(proc_pid.join("fd")) else {
        return false;
    };
    for entry in entries.flatten() {
        match std::fs::read_link(entry.path()) {
            Ok(link) if link == *target => return true,
            _ => continue,
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    /// Lay out a fake `/proc/<pid>/fd/<n>` symlink to `target`.
    fn fake_fd(root: &Path, pid: &str, fd: &str, target: &Path) {
        let fd_dir = root.join(pid).join("fd");
        fs::create_dir_all(&fd_dir).unwrap();
        std::os::unix::fs::symlink(target, fd_dir.join(fd)).unwrap();
    }

    fn inbox(dir: &Path) -> PathBuf {
        let p = dir.join("inbox").join("agent-1");
        fs::create_dir_all(&p).unwrap();
        p
    }

    #[test]
    fn detects_a_holder_of_the_inbox_dir() {
        let dir = tempdir().unwrap();
        let target = inbox(dir.path());
        let proc = dir.path().join("proc");
        fake_fd(&proc, "42", "7", &target);
        assert!(ProcFsLockProbe::with_root(proc).lock_held(&target));
    }

    #[test]
    fn no_holder_when_no_fd_matches() {
        let dir = tempdir().unwrap();
        let target = inbox(dir.path());
        let other = dir.path().join("elsewhere");
        fs::create_dir_all(&other).unwrap();
        let proc = dir.path().join("proc");
        fake_fd(&proc, "42", "7", &other);
        assert!(!ProcFsLockProbe::with_root(proc).lock_held(&target));
    }

    #[test]
    fn missing_inbox_dir_is_no_holder() {
        let dir = tempdir().unwrap();
        let proc = dir.path().join("proc");
        fs::create_dir_all(&proc).unwrap();
        assert!(!ProcFsLockProbe::with_root(proc).lock_held(&dir.path().join("inbox/gone")));
    }

    #[test]
    fn missing_proc_root_is_no_holder() {
        let dir = tempdir().unwrap();
        let target = inbox(dir.path());
        assert!(!ProcFsLockProbe::with_root(dir.path().join("no-proc")).lock_held(&target));
    }

    #[test]
    fn non_numeric_proc_entries_and_unreadable_fd_dirs_are_skipped() {
        let dir = tempdir().unwrap();
        let target = inbox(dir.path());
        let proc = dir.path().join("proc");
        // A non-pid dir is skipped.
        fs::create_dir_all(proc.join("acpi")).unwrap();
        // A pid dir with no `fd/` subdir → read_dir fails → no match.
        fs::create_dir_all(proc.join("9")).unwrap();
        assert!(!ProcFsLockProbe::with_root(proc).lock_held(&target));
    }

    #[test]
    fn non_symlink_fd_entry_is_skipped() {
        // A regular file where an fd symlink is expected → `read_link`
        // fails and the entry is skipped (matches a racing teardown).
        let dir = tempdir().unwrap();
        let target = inbox(dir.path());
        let proc = dir.path().join("proc");
        let fd_dir = proc.join("13").join("fd");
        fs::create_dir_all(&fd_dir).unwrap();
        fs::write(fd_dir.join("3"), b"not a symlink").unwrap();
        assert!(!ProcFsLockProbe::with_root(proc).lock_held(&target));
    }
}
