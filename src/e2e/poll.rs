//! The one polling primitive the end-to-end tests wait on, bounded by
//! **silence** rather than by wall time.
//!
//! Every e2e assertion about a detached driver (§2.11) is made against
//! disk: the test owns no handle on the process, so it watches the
//! workspace exactly as a frontend does (§3.5) until the artifact it
//! expects shows up. The pass path is satisfied by observable state
//! however slowly it arrives — so a wall-clock bound on it is not a
//! property of the code under test, it is a property of the machine. A
//! deadline measured on a loaded box reports the load, which is the same
//! call `docs/ARCHITECTURE.md` §2.9 already makes for stop's leader-pgid
//! re-read: *"a retry **count**, not a wall-clock deadline: this race only
//! appears under load, and a deadline measured under load reports the
//! load."* A 120s bound here was outrun anyway (bl-2bf0) on a box running
//! three test suites at once — the chain was slow, not stuck, and the
//! bound could not tell those apart.
//!
//! So the bound counts **consecutive probes that observed no activity in
//! the workspace**. Activity is any change in the tree — a file appearing,
//! growing, being replaced or removed — which is what a live driver emits
//! continuously (delivery commits, transcript entries, streamed
//! `response.json` event lines, worktree churn) and what a wedged or dead
//! one emits none of. An arbitrarily slow machine therefore only makes the
//! pass path slower, never redder; the only thing that fails is genuine
//! silence, which is the hang the bound was always there to diagnose.

use std::fs;
use std::hash::{DefaultHasher, Hash, Hasher};
use std::path::{Path, PathBuf};
use std::thread;
use std::time::{Duration, SystemTime};

/// Cadence between probes — backoff, never a verdict.
const PROBE_INTERVAL: Duration = Duration::from_millis(100);

/// The bound: consecutive probes that saw a wholly motionless workspace.
/// It is *not* a budget for the work — the work resets it every time it
/// touches disk — so it is sized against the longest plausible gap
/// between two writes of a chain that is still going (process spawn and
/// exec, a git object read, a model-call round trip against a local mock),
/// with orders of magnitude to spare. Nothing on the pass path approaches
/// it, so the number costs only how long a genuine hang takes to report.
const SILENT_PROBES: u32 = 600;

/// Probe until it yields a value, or the workspace falls silent for
/// [`SILENT_PROBES`] straight probes. `None` is the stall verdict — the
/// caller panics with its own diagnostics, since only the caller knows
/// what it was watching and what else is worth dumping (a child's stderr,
/// the ref list). The caller's mutable borrows end with this call, which
/// is why the diagnostics are not a second closure taken here.
pub fn until<T>(workspace: &Path, mut probe: impl FnMut() -> Option<T>) -> Option<T> {
    let mut last = activity(workspace);
    let mut silent = 0u32;
    loop {
        if let Some(value) = probe() {
            return Some(value);
        }
        if silent >= SILENT_PROBES {
            return None;
        }
        thread::sleep(PROBE_INTERVAL);
        let now = activity(workspace);
        silent = if now == last { silent + 1 } else { 0 };
        last = now;
    }
}

/// How long [`until`] stays with a motionless workspace, for diagnostics.
pub fn patience() -> Duration {
    PROBE_INTERVAL * SILENT_PROBES
}

/// A fingerprint of every path under `root` — an order-independent sum
/// over (path, length, mtime), so it moves on any create, append,
/// replace, rename or delete and holds still while nothing runs.
/// Symlinks are fingerprinted, never followed.
fn activity(root: &Path) -> u64 {
    let mut stack = vec![root.to_path_buf()];
    let mut sum = 0u64;
    while let Some(dir) = stack.pop() {
        for entry in fs::read_dir(&dir).into_iter().flatten().flatten() {
            let path = entry.path();
            let meta = entry.metadata().ok();
            let stamp = meta.map(|m| (m.is_dir(), m.len(), nanos(m.modified().ok())));
            if stamp.is_some_and(|(is_dir, ..)| is_dir) {
                stack.push(PathBuf::from(&path));
            }
            let mut hasher = DefaultHasher::new();
            (path, stamp).hash(&mut hasher);
            sum = sum.wrapping_add(hasher.finish());
        }
    }
    sum
}

fn nanos(at: Option<SystemTime>) -> Option<u128> {
    at.and_then(|t| t.duration_since(SystemTime::UNIX_EPOCH).ok())
        .map(|d| d.as_nanos())
}
