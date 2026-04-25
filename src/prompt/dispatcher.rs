//! Subagent dispatch as a CLI re-entry (ARCH §3.4).
//!
//! Compaction is a dispatch — a specific implementation of the generic
//! "spawn a branch with a goal, do work, merge back" primitive (§2.5,
//! §2.7). §3.4 requires every procedure-to-procedure invocation to go
//! through the `lernie` CLI; this module is the harness side of that
//! contract for the v0.3 compactor case.
//!
//! [`SpawnDispatcher`] re-enters the binary at `lernie dispatch <role>`
//! via [`std::env::current_exe`]. The trait is `&dyn`-shaped so tests
//! supply a recording stub instead of paying the subprocess cost.

use std::io;
use std::path::{Path, PathBuf};
use std::process::Command;

/// The CLI dispatch surface the harness depends on. v0.3 has one role
/// (`compactor`); v0.4 adds verifier/worker/adversary/etc. through the
/// same primitive.
pub trait Dispatcher {
    /// Re-enter the CLI as `lernie dispatch compactor <repo> <branch>`
    /// and wait for the subprocess to terminate. Returns `Err` when the
    /// subprocess cannot start or exits non-zero.
    fn dispatch_compactor(&self, repo: &Path, branch: &str) -> io::Result<()>;
}

/// Production [`Dispatcher`] — re-enters a `lernie` binary as a
/// subprocess. §3.4 permits subprocess `exec` or in-process re-entry
/// per-procedure; v0.3 picks subprocess for clean isolation between
/// the conversation orchestrator and the compactor.
///
/// The binary path is a field so tests can pin it to `true`/`false` (or
/// a missing path) and exercise the wrapper without spawning the real
/// `lernie`. Production constructs via [`SpawnDispatcher::new`], which
/// uses [`std::env::current_exe`].
#[derive(Debug, Clone)]
pub struct SpawnDispatcher {
    exe: PathBuf,
}

impl SpawnDispatcher {
    /// Re-enter the currently running `lernie` binary. Fails when the
    /// OS cannot resolve the current executable (rare; mostly unusual
    /// platforms or `proc` mounts).
    pub fn new() -> io::Result<Self> {
        Ok(Self {
            exe: std::env::current_exe()?,
        })
    }

    /// Explicit binary path — exposed for tests and for embedded usage
    /// where the caller picks a non-default `lernie`.
    pub fn with_exe(exe: PathBuf) -> Self {
        Self { exe }
    }
}

impl Dispatcher for SpawnDispatcher {
    fn dispatch_compactor(&self, repo: &Path, branch: &str) -> io::Result<()> {
        let status = Command::new(&self.exe)
            .args(["dispatch", "compactor"])
            .arg(repo)
            .arg(branch)
            .status()?;
        if status.success() {
            Ok(())
        } else {
            Err(io::Error::other(format!(
                "lernie dispatch compactor exited with {status}"
            )))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spawn_dispatcher_returns_ok_on_zero_exit() {
        // `true` exits 0 unconditionally; the args are noise to it.
        let d = SpawnDispatcher::with_exe(PathBuf::from("true"));
        d.dispatch_compactor(Path::new("/tmp"), "conv-id-deadbeef")
            .unwrap();
    }

    #[test]
    fn spawn_dispatcher_returns_err_on_nonzero_exit() {
        // `false` exits 1 unconditionally.
        let d = SpawnDispatcher::with_exe(PathBuf::from("false"));
        let err = d
            .dispatch_compactor(Path::new("/tmp"), "conv-id-deadbeef")
            .unwrap_err();
        assert!(err.to_string().contains("dispatch compactor"), "got {err}");
    }

    #[test]
    fn spawn_dispatcher_returns_err_on_spawn_failure() {
        let d = SpawnDispatcher::with_exe(PathBuf::from("/no/such/lernie-binary"));
        let err = d
            .dispatch_compactor(Path::new("/tmp"), "conv-id-deadbeef")
            .unwrap_err();
        assert_eq!(err.kind(), io::ErrorKind::NotFound);
    }

    #[test]
    fn spawn_dispatcher_new_resolves_current_exe() {
        // Smoke-test that current_exe lookup is wired; we don't need to
        // actually invoke it — Linux test runners always have a known
        // current exe.
        let d = SpawnDispatcher::new().unwrap();
        assert!(!d.exe.as_os_str().is_empty());
    }
}
