//! Pid discovery for `lernie stop` (Linux `/proc` scan).
//!
//! Resolves "the harness driving step `<NNN>` of branch `<branch>`"
//! by scanning `/proc/<pid>/fd/*` symlinks for the absolute path of
//! `<step>/response.json`. Filters to writers via the access-mode
//! bits in `/proc/<pid>/fdinfo/<fd>` so a reader (e.g. a UI tail)
//! cannot be mistaken for the harness.
//!
//! The trait is `&dyn`-shaped so tests pass a stub and production
//! pays the directory scan only when actually called. See
//! [`super::tests::run`] for fixture usage.
//!
//! Linux only — `/proc` is not portable to Darwin or Windows. ARCH
//! §2.9 calls Linux out as the verified platform; portability deltas
//! are a v0.6+ concern.

use std::io;
use std::path::Path;

/// "Find the pgid of any writer holding `response_path` open."
/// `None` means no writer found — the harness has already exited or
/// has not yet opened the file. The pgid (== leader pid for a setpgid'd
/// harness, ARCH §2.9 cascade) is what [`super::cascade`] signals.
pub trait PgidFinder {
    fn find_writer_pgid(&self, response_path: &Path) -> io::Result<Option<i32>>;
}

/// Production [`PgidFinder`] backed by `/proc`.
#[derive(Debug, Clone)]
pub struct ProcFsFinder {
    proc_root: std::path::PathBuf,
}

impl Default for ProcFsFinder {
    fn default() -> Self {
        Self {
            proc_root: std::path::PathBuf::from("/proc"),
        }
    }
}

impl ProcFsFinder {
    /// Override the procfs root — tests point at a fixture tree.
    pub fn with_root(proc_root: std::path::PathBuf) -> Self {
        Self { proc_root }
    }
}

impl PgidFinder for ProcFsFinder {
    fn find_writer_pgid(&self, response_path: &Path) -> io::Result<Option<i32>> {
        // Canonicalize so the symlink-target compare is exact —
        // `/proc/<pid>/fd/<n>` resolves to a fully-resolved path.
        let target = match std::fs::canonicalize(response_path) {
            Ok(p) => p,
            // The file may have been removed between the harness
            // closing it and us scanning. Treat as no writer.
            Err(e) if e.kind() == io::ErrorKind::NotFound => return Ok(None),
            Err(e) => return Err(e),
        };
        for entry in std::fs::read_dir(&self.proc_root)? {
            let entry = match entry {
                Ok(e) => e,
                Err(_) => continue, // racing with kernel pid teardown
            };
            let Some(pid) = parse_pid_dir_name(&entry.file_name()) else {
                continue;
            };
            if let Some(fd) = pid_holds_writable(&entry.path(), &target)? {
                return Ok(Some(read_pgid(&self.proc_root, pid, fd)?));
            }
        }
        Ok(None)
    }
}

fn parse_pid_dir_name(name: &std::ffi::OsStr) -> Option<i32> {
    name.to_str().and_then(|s| s.parse::<i32>().ok())
}

/// Walk `<proc_pid>/fd/` for a symlink resolving to `target`. Returns
/// the fd number if found and (per `fdinfo`) opened for write.
fn pid_holds_writable(proc_pid: &Path, target: &Path) -> io::Result<Option<u32>> {
    let fd_dir = proc_pid.join("fd");
    let entries = match std::fs::read_dir(&fd_dir) {
        Ok(e) => e,
        // Most pids will refuse the read (different uid, kernel
        // thread, raced teardown). The fd-scan is opportunistic —
        // pids we can't introspect simply don't match.
        Err(_) => return Ok(None),
    };
    for entry in entries {
        let entry = match entry {
            Ok(e) => e,
            Err(_) => continue,
        };
        // `read_link` on `/proc/<pid>/fd/<n>` returns the target
        // path; `metadata`/`canonicalize` would dereference and may
        // fail for sockets, pipes, etc. — read_link side-steps that.
        let link = match std::fs::read_link(entry.path()) {
            Ok(l) => l,
            Err(_) => continue,
        };
        if link != target {
            continue;
        }
        let Some(fd_num) = entry
            .file_name()
            .to_str()
            .and_then(|s| s.parse::<u32>().ok())
        else {
            continue;
        };
        if fdinfo_is_writable(proc_pid, fd_num)? {
            return Ok(Some(fd_num));
        }
    }
    Ok(None)
}

/// Parse `flags:` from `/proc/<pid>/fdinfo/<fd>`. The low octal digit
/// of the access-mode is `0` for `O_RDONLY`, `1` for `O_WRONLY`, and
/// `2` for `O_RDWR`. Anything but `0` counts as "writer".
fn fdinfo_is_writable(proc_pid: &Path, fd: u32) -> io::Result<bool> {
    let fdinfo_path = proc_pid.join("fdinfo").join(fd.to_string());
    let contents = match std::fs::read_to_string(&fdinfo_path) {
        Ok(s) => s,
        // fdinfo may briefly be missing during fd churn; treat as
        // not-a-writer rather than fatal — false positives in the
        // pid scan must not abort `lernie stop`.
        Err(_) => return Ok(false),
    };
    for line in contents.lines() {
        if let Some(rest) = line.strip_prefix("flags:") {
            let trimmed = rest.trim();
            if trimmed.is_empty() {
                return Ok(false);
            }
            let access_mode_digit = trimmed.chars().last().expect("non-empty after trim");
            return Ok(access_mode_digit != '0');
        }
    }
    Ok(false)
}

/// Read `/proc/<pid>/stat` and return the pgid (4th field by libc
/// `proc(5)`). The stat line has the form
/// `<pid> (<comm>) <state> <ppid> <pgid> ...`; the comm field can
/// contain spaces and parens, so we split off everything up to the
/// last `)` before tokenizing.
fn read_pgid(proc_root: &Path, pid: i32, _fd: u32) -> io::Result<i32> {
    let stat_path = proc_root.join(pid.to_string()).join("stat");
    let raw = std::fs::read_to_string(&stat_path)?;
    let after_comm = raw.rsplit_once(')').map(|(_, rest)| rest).ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            format!("malformed /proc/{pid}/stat"),
        )
    })?;
    let mut fields = after_comm.split_whitespace();
    // After ')' the remaining whitespace-tokens are: state ppid pgid ...
    // We want field index 2 (pgid) of the post-comm tail.
    fields.next(); // state
    fields.next(); // ppid
    let pgid_str = fields.next().ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            format!("missing pgid field in /proc/{pid}/stat"),
        )
    })?;
    pgid_str.parse::<i32>().map_err(|e| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            format!("pgid parse: {e} in /proc/{pid}/stat"),
        )
    })
}

#[cfg(test)]
mod tests;
