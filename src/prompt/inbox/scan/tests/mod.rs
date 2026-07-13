//! Tests for the `lernie scan` workspace sweep (ARCH §2.11 *Crashes are
//! a failure class*, §8), split
//! by axis so each file stays under the repo's per-file line cap.
//!
//! - [`sweep`]: the silent-death sweep — the never-deposited-child deposit,
//!   idempotence across a double scan, the returned/driven/root exclusions,
//!   and the died-mid-work classification over `steps/`.
//! - [`flush`]: the inbox flush, error propagation across every read/probe/
//!   launch seam, and the pure filename matchers.
//!
//! Branch enumeration and transcript reads go through the scripted
//! [`StubGit`]; the sweep's deposits and the flush's launches land against
//! a real on-disk workspace (`inbox/`, `steps/`) with the launch captured.

mod flush;
mod sweep;

use super::derive::{
    died_mid_work, has_pending, is_message_from, is_pending_deposit, transcript_line_from,
};
use super::{ScanError, ScanReport, cli_run, scan};
use crate::prompt::Clock;
use crate::prompt::inbox::{INBOX_DIR, Launcher, inbox_dir, try_acquire};
use crate::prompt::step::{RESPONSE_FILE, STEPS_DIR};
use crate::template::GitRunner;
use std::cell::RefCell;
use std::collections::HashMap;
use std::io;
use tempfile::TempDir;

pub(super) struct FixedClock;
impl Clock for FixedClock {
    fn now_iso8601(&self) -> String {
        "2026-07-11T00:00:00Z".into()
    }
    fn now_compact(&self) -> String {
        unreachable!("scan never reads the compact clock")
    }
}

/// A [`GitRunner`] scripted for the scan's three reads: `branch --list`,
/// `ls-tree` (per parent branch), and `rev-parse` (per branch tip). Any
/// unscripted `ls-tree`/`rev-parse` returns empty / a default sha; an
/// optional `fail_op` makes the next matching op error.
#[derive(Default)]
pub(super) struct StubGit {
    branches: String,
    ls_tree: HashMap<String, String>,
    tips: HashMap<String, String>,
    fail_op: Option<&'static str>,
    calls: RefCell<Vec<Vec<String>>>,
}

impl StubGit {
    pub(super) fn with_branches(names: &[&str]) -> Self {
        Self {
            branches: names.join("\n"),
            ..Self::default()
        }
    }
    pub(super) fn ls_tree(mut self, branch: &str, listing: &str) -> Self {
        self.ls_tree.insert(branch.to_string(), listing.to_string());
        self
    }
    pub(super) fn tip(mut self, branch: &str, sha: &str) -> Self {
        self.tips.insert(branch.to_string(), sha.to_string());
        self
    }
    pub(super) fn failing(mut self, op: &'static str) -> Self {
        self.fail_op = Some(op);
        self
    }
}

impl GitRunner for StubGit {
    fn run(&self, _dest: &std::path::Path, _args: &[&str]) -> io::Result<()> {
        unreachable!("scan issues only run_capture")
    }
    fn run_capture(&self, _dest: &std::path::Path, args: &[&str]) -> io::Result<String> {
        self.calls
            .borrow_mut()
            .push(args.iter().map(|s| (*s).to_string()).collect());
        match args.first().copied() {
            Some("branch") => {
                if self.fail_op == Some("branch") {
                    return Err(io::Error::other("branch boom"));
                }
                Ok(self.branches.clone())
            }
            Some("ls-tree") => {
                if self.fail_op == Some("ls-tree") {
                    return Err(io::Error::other("ls-tree boom"));
                }
                // `git ls-tree -r --name-only <branch> -- messages` → [3].
                let branch = args.get(3).copied().unwrap_or("");
                Ok(self.ls_tree.get(branch).cloned().unwrap_or_default())
            }
            Some("rev-parse") => {
                if self.fail_op == Some("rev-parse") {
                    return Err(io::Error::other("rev-parse boom"));
                }
                let branch = args.get(2).copied().unwrap_or("");
                Ok(self
                    .tips
                    .get(branch)
                    .cloned()
                    .unwrap_or_else(|| "deadbeef".into()))
            }
            other => unreachable!("unexpected git op {other:?}"),
        }
    }
}

/// Capturing [`Launcher`]; records each launched agent.
#[derive(Default)]
pub(super) struct StubLauncher {
    calls: RefCell<Vec<String>>,
}
impl StubLauncher {
    pub(super) fn calls(&self) -> Vec<String> {
        self.calls.borrow().clone()
    }
}
impl Launcher for StubLauncher {
    fn launch(&self, _workspace: &std::path::Path, agent_id: &str) -> io::Result<()> {
        self.calls.borrow_mut().push(agent_id.to_string());
        Ok(())
    }
}
pub(super) struct FailLauncher;
impl Launcher for FailLauncher {
    fn launch(&self, _workspace: &std::path::Path, _agent_id: &str) -> io::Result<()> {
        Err(io::Error::other("cannot spawn"))
    }
}

/// A root id (two `<ts>-<short>` tokens) and one of its children (four).
pub(super) const PARENT: &str = "20260101-p1";
pub(super) const CHILD: &str = "20260101-p1-20260102-c1";

pub(super) fn deposit_msg(ws: &std::path::Path, agent: &str, name: &str) {
    let dir = inbox_dir(ws, agent);
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join(name), "---\nfrom: x\n---\nhi").unwrap();
}

pub(super) fn write_response(ws: &std::path::Path, branch: &str, seq: &str, body: &str) {
    let dir = ws.join(STEPS_DIR).join(branch).join(seq);
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join(RESPONSE_FILE), body).unwrap();
}

/// A complete brazen segment: a `Finish` then the terminal `End`.
pub(super) const COMPLETE: &str = "{\"type\":\"finish\",\"reason\":\"stop\"}\n{\"type\":\"end\"}\n";
/// A killed segment: content with no terminal `End` line.
pub(super) const KILLED: &str = "{\"type\":\"finish\",\"reason\":\"stop\"}\n";
