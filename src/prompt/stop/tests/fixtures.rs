//! Stubs and helpers shared by the [`super`] orchestration tests.
//!
//! `pub(super)` so siblings (`orchestration`, `edge_cases`) reach
//! them; not `pub(crate)` because nothing outside the stop module's
//! test tree should depend on these shapes.

use crate::prompt::stop::*;
use crate::template::GitRunner;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

pub(super) struct StubInspector {
    pub(super) exists: bool,
}
impl BranchInspector for StubInspector {
    fn exists(&self, _: &Path, _: &str, _: &dyn GitRunner) -> io::Result<bool> {
        Ok(self.exists)
    }
}

pub(super) struct ErrInspector;
impl BranchInspector for ErrInspector {
    fn exists(&self, _: &Path, _: &str, _: &dyn GitRunner) -> io::Result<bool> {
        Err(io::Error::other("rev-parse blew up"))
    }
}

#[derive(Default)]
pub(super) struct StubFinder {
    /// Per-call return value, indexed by call order. Empty entries
    /// default to None.
    returns: Mutex<Vec<Option<i32>>>,
    /// Path each call was passed, for assertion.
    pub(super) seen: Mutex<Vec<PathBuf>>,
}

impl StubFinder {
    pub(super) fn with_returns(returns: Vec<Option<i32>>) -> Self {
        Self {
            returns: Mutex::new(returns),
            seen: Mutex::new(Vec::new()),
        }
    }
}

impl PgidFinder for StubFinder {
    fn find_holder_pgid(&self, inbox_dir: &Path) -> io::Result<Option<i32>> {
        self.seen.lock().unwrap().push(inbox_dir.to_owned());
        let mut q = self.returns.lock().unwrap();
        if q.is_empty() {
            Ok(None)
        } else {
            Ok(q.remove(0))
        }
    }
}

/// A [`PgidFinder`] that resolves the stop process's **own** process
/// group — the pathological reading the §2.9 self-group guard exists to
/// refuse (discovery reading a not-yet-detached executor's inherited
/// group). Read at probe time rather than baked in, so it names
/// whatever group the test binary is actually standing in.
pub(super) struct OwnGroupFinder;
impl PgidFinder for OwnGroupFinder {
    fn find_holder_pgid(&self, _: &Path) -> io::Result<Option<i32>> {
        // SAFETY: `getpgrp` takes no arguments and cannot fail.
        Ok(Some(unsafe { libc::getpgrp() }))
    }
}

pub(super) struct ErrFinder;
impl PgidFinder for ErrFinder {
    fn find_holder_pgid(&self, _: &Path) -> io::Result<Option<i32>> {
        Err(io::Error::other("/proc unreadable"))
    }
}

#[derive(Default)]
pub(super) struct NoopGit;
impl GitRunner for NoopGit {
    fn run(&self, _: &Path, _: &[&str]) -> io::Result<()> {
        Ok(())
    }
    fn run_capture(&self, _: &Path, _: &[&str]) -> io::Result<String> {
        Ok(String::new())
    }
}

pub(super) const INBOX_DIR: &str = "inbox";

/// Create the inbox directory `inbox/<agent_id>/` — the executor lock's
/// home (§2.11) and the target `super::run` scans for a live holder.
/// Returns its path.
pub(super) fn touch_inbox_dir(repo: &Path, agent_id: &str) -> PathBuf {
    let dir = repo.join(INBOX_DIR).join(agent_id);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}
