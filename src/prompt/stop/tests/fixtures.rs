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
    pub(super) merged: bool,
}
impl BranchInspector for StubInspector {
    fn exists(&self, _: &Path, _: &str, _: &dyn GitRunner) -> io::Result<bool> {
        Ok(self.exists)
    }
    fn is_merged_into_main(&self, _: &Path, _: &str, _: &dyn GitRunner) -> io::Result<bool> {
        Ok(self.merged)
    }
}

pub(super) struct ErrInspector;
impl BranchInspector for ErrInspector {
    fn exists(&self, _: &Path, _: &str, _: &dyn GitRunner) -> io::Result<bool> {
        Err(io::Error::other("rev-parse blew up"))
    }
    fn is_merged_into_main(&self, _: &Path, _: &str, _: &dyn GitRunner) -> io::Result<bool> {
        unreachable!()
    }
}

pub(super) struct ErrMergedInspector;
impl BranchInspector for ErrMergedInspector {
    fn exists(&self, _: &Path, _: &str, _: &dyn GitRunner) -> io::Result<bool> {
        Ok(true)
    }
    fn is_merged_into_main(&self, _: &Path, _: &str, _: &dyn GitRunner) -> io::Result<bool> {
        Err(io::Error::other("merge-base blew up"))
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
    fn find_writer_pgid(&self, response_path: &Path) -> io::Result<Option<i32>> {
        self.seen.lock().unwrap().push(response_path.to_owned());
        let mut q = self.returns.lock().unwrap();
        if q.is_empty() {
            Ok(None)
        } else {
            Ok(q.remove(0))
        }
    }
}

pub(super) struct ErrFinder;
impl PgidFinder for ErrFinder {
    fn find_writer_pgid(&self, _: &Path) -> io::Result<Option<i32>> {
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

pub(super) const STEPS_DIR: &str = "steps";
pub(super) const RESPONSE_FILE: &str = "response.json";

pub(super) fn touch_step_response(repo: &Path, conv_id: &str, seq: u32) -> PathBuf {
    let dir = repo.join(STEPS_DIR).join(conv_id).join(format!("{seq:03}"));
    std::fs::create_dir_all(&dir).unwrap();
    let p = dir.join(RESPONSE_FILE);
    std::fs::write(&p, "").unwrap();
    p
}
