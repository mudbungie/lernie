//! Writer-fd probe for the branch-state classifier (ARCH §3.5, §4.4).
//!
//! The §3.5 completion signal is the writer closing the `response.json`
//! fd (`IN_CLOSE_WRITE`). A terminal event on disk is only authoritative
//! once that fd is closed: the harness holds ONE fd across every retry
//! attempt and the backoff sleeps between them (§4.4 "Fd held open for
//! the whole model call"), so a mid-retry `end` segment with a writer
//! still present is `in_flight`, not `stopped`. This module answers the
//! one question the classifier needs — "does a process still hold this
//! path open for *write*?" — by scanning `/proc/<pid>/fd/*`, filtered to
//! writers via `/proc/<pid>/fdinfo/<fd>` (a reader such as the UI's own
//! tail must never be mistaken for the harness).
//!
//! The scanner mirrors the harness's own `stop::discover` writer scan;
//! it is duplicated here (not shared via a crate dep) so the frontend
//! stays decoupled from the harness binary (ARCH §3.5 pluggability — a
//! frontend shares nothing but the filesystem and the CLI). Linux only;
//! `/proc` is the verified platform (§2.9).

use std::path::{Path, PathBuf};

/// "Does a process hold `path` open for write?" Injected so the
/// classifier is testable without a live writer.
pub(super) trait WriterProbe {
    fn writer_open(&self, path: &Path) -> bool;
}

/// Production probe backed by `/proc`.
pub(super) struct ProcFsProbe {
    proc_root: PathBuf,
}

impl Default for ProcFsProbe {
    fn default() -> Self {
        Self {
            proc_root: PathBuf::from("/proc"),
        }
    }
}

impl ProcFsProbe {
    #[cfg(test)]
    fn with_root(proc_root: PathBuf) -> Self {
        Self { proc_root }
    }

    fn scan(&self, path: &Path) -> bool {
        // Canonicalize so the symlink-target compare is exact. A file
        // removed between close and scan canonicalizes to NotFound →
        // no writer.
        let Ok(target) = std::fs::canonicalize(path) else {
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
            if pid_holds_writable(&entry.path(), &target) {
                return true;
            }
        }
        false
    }
}

impl WriterProbe for ProcFsProbe {
    fn writer_open(&self, path: &Path) -> bool {
        self.scan(path)
    }
}

/// Any fd under `<proc_pid>/fd/` resolving to `target` and opened for
/// write. Pids we cannot introspect (other uid, raced teardown) simply
/// do not match.
fn pid_holds_writable(proc_pid: &Path, target: &Path) -> bool {
    let Ok(entries) = std::fs::read_dir(proc_pid.join("fd")) else {
        return false;
    };
    for entry in entries.flatten() {
        let Ok(link) = std::fs::read_link(entry.path()) else {
            continue;
        };
        if link != target {
            continue;
        }
        let Some(fd) = entry
            .file_name()
            .to_str()
            .and_then(|s| s.parse::<u32>().ok())
        else {
            continue;
        };
        if fdinfo_is_writable(proc_pid, fd) {
            return true;
        }
    }
    false
}

/// Parse `flags:` from `<proc_pid>/fdinfo/<fd>`. The low octal digit of
/// the access mode is `0` for `O_RDONLY`; anything else is a writer.
fn fdinfo_is_writable(proc_pid: &Path, fd: u32) -> bool {
    let Ok(contents) = std::fs::read_to_string(proc_pid.join("fdinfo").join(fd.to_string())) else {
        return false;
    };
    for line in contents.lines() {
        if let Some(rest) = line.strip_prefix("flags:") {
            return rest.trim().chars().last().is_some_and(|c| c != '0');
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    /// Lay out a fake `/proc/<pid>/fd/<n>` symlink to `target` plus a
    /// matching `fdinfo/<n>` with `flags`.
    fn fake_proc(root: &Path, pid: &str, fd: &str, target: &Path, flags: &str) {
        let fd_dir = root.join(pid).join("fd");
        fs::create_dir_all(&fd_dir).unwrap();
        std::os::unix::fs::symlink(target, fd_dir.join(fd)).unwrap();
        let fdinfo = root.join(pid).join("fdinfo");
        fs::create_dir_all(&fdinfo).unwrap();
        fs::write(fdinfo.join(fd), format!("pos:\t0\nflags:\t{flags}\n")).unwrap();
    }

    fn target_file(dir: &Path) -> PathBuf {
        let p = dir.join("response.json");
        fs::write(&p, b"x").unwrap();
        p
    }

    #[test]
    fn detects_a_writer_holding_the_path() {
        let dir = tempdir().unwrap();
        let target = target_file(dir.path());
        let proc = dir.path().join("proc");
        // O_WRONLY (octal ...01) → writer.
        fake_proc(&proc, "42", "3", &target, "0100001");
        assert!(ProcFsProbe::with_root(proc).writer_open(&target));
    }

    #[test]
    fn ignores_a_reader_only_handle() {
        let dir = tempdir().unwrap();
        let target = target_file(dir.path());
        let proc = dir.path().join("proc");
        // O_RDONLY (trailing 0) → not a writer (e.g. the UI's own tail).
        fake_proc(&proc, "42", "3", &target, "0100000");
        assert!(!ProcFsProbe::with_root(proc).writer_open(&target));
    }

    #[test]
    fn no_writer_when_no_fd_matches() {
        let dir = tempdir().unwrap();
        let target = target_file(dir.path());
        let other = dir.path().join("other.json");
        fs::write(&other, b"y").unwrap();
        let proc = dir.path().join("proc");
        fake_proc(&proc, "42", "3", &other, "0100001");
        assert!(!ProcFsProbe::with_root(proc).writer_open(&target));
    }

    #[test]
    fn missing_target_is_no_writer() {
        let dir = tempdir().unwrap();
        let proc = dir.path().join("proc");
        fs::create_dir_all(&proc).unwrap();
        assert!(!ProcFsProbe::with_root(proc).writer_open(&dir.path().join("gone.json")));
    }

    #[test]
    fn missing_proc_root_is_no_writer() {
        let dir = tempdir().unwrap();
        let target = target_file(dir.path());
        assert!(!ProcFsProbe::with_root(dir.path().join("no-proc")).writer_open(&target));
    }

    #[test]
    fn non_numeric_proc_entries_and_missing_fdinfo_are_skipped() {
        let dir = tempdir().unwrap();
        let target = target_file(dir.path());
        let proc = dir.path().join("proc");
        // A non-pid dir is skipped.
        fs::create_dir_all(proc.join("acpi")).unwrap();
        // A pid whose fd matches but whose fdinfo is absent → not a
        // writer (flags unknown, treated conservatively).
        let fd_dir = proc.join("7").join("fd");
        fs::create_dir_all(&fd_dir).unwrap();
        std::os::unix::fs::symlink(&target, fd_dir.join("4")).unwrap();
        assert!(!ProcFsProbe::with_root(proc).writer_open(&target));
    }

    #[test]
    fn fdinfo_without_flags_line_is_not_writable() {
        let dir = tempdir().unwrap();
        let target = target_file(dir.path());
        let proc = dir.path().join("proc");
        let fd_dir = proc.join("9").join("fd");
        fs::create_dir_all(&fd_dir).unwrap();
        std::os::unix::fs::symlink(&target, fd_dir.join("5")).unwrap();
        let fdinfo = proc.join("9").join("fdinfo");
        fs::create_dir_all(&fdinfo).unwrap();
        fs::write(fdinfo.join("5"), "pos:\t0\n").unwrap(); // no flags line
        assert!(!ProcFsProbe::with_root(proc).writer_open(&target));
    }

    #[test]
    fn non_numeric_fd_entry_is_skipped() {
        let dir = tempdir().unwrap();
        let target = target_file(dir.path());
        let proc = dir.path().join("proc");
        let fd_dir = proc.join("11").join("fd");
        fs::create_dir_all(&fd_dir).unwrap();
        // A non-numeric fd name resolving to target is skipped.
        std::os::unix::fs::symlink(&target, fd_dir.join("notanum")).unwrap();
        assert!(!ProcFsProbe::with_root(proc).writer_open(&target));
    }

    #[test]
    fn non_symlink_fd_entry_is_skipped() {
        // A regular file where an fd symlink is expected → `read_link`
        // fails and the entry is skipped (matches a racing teardown).
        let dir = tempdir().unwrap();
        let target = target_file(dir.path());
        let proc = dir.path().join("proc");
        let fd_dir = proc.join("13").join("fd");
        fs::create_dir_all(&fd_dir).unwrap();
        fs::write(fd_dir.join("3"), b"not a symlink").unwrap();
        assert!(!ProcFsProbe::with_root(proc).writer_open(&target));
    }
}
