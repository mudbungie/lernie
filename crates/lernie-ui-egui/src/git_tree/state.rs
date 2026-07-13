//! Agent-state classifier (ARCH §3.5 / §7.1, terminal rules §4.4).
//!
//! The four live-view states are derived from the executor lock, the
//! agent's latest-step `response.json`, and nothing stored (PRINCIPLES
//! "Single source of truth"):
//!
//! - [`AgentState::Live`] — a driver holds the agent's inbox-directory lock
//!   (§2.11): someone is stepping the branch.
//! - [`AgentState::InFlight`] — the `live` sub-state where a model call is
//!   in flight *right now*: the latest step's `response.json` fd is still
//!   open (§3.5, §4.4). The harness holds that fd across every retry
//!   attempt and backoff sleep, so a mid-retry `end` segment is still
//!   in_flight, never stopped.
//! - [`AgentState::Quiescent`] — no lock held and the latest step's
//!   `response.json` is *complete* (§4.4): last line `end`, last segment a
//!   `finish` with no `error`. A finished-for-now agent awaiting a message
//!   (§2.4).
//! - [`AgentState::Stopped`] — no lock held and the latest step is *failed*
//!   (last segment carries an `error`, §2.10) or *killed* (closed with no
//!   trailing `end`, §2.9), or no step has run. Kill, crash, and explicit
//!   stop are indistinguishable on disk (§2.9); a failed step renders here
//!   too, per §3.5.
//!
//! The two observations are deliberately not collapsed (§2.11): the lock is
//! *is-anyone-driving*; the open `response.json` fd is
//! *is-a-model-call-in-flight-right-now*.

use std::path::{Path, PathBuf};

use super::fd_probe::WriterProbe;
use super::lock_probe::LockProbe;
use super::streaming::latest_step_dir;
use super::terminal::last_segment_complete;
use super::{INBOX_DIR, STEPS_DIR};

const RESPONSE_FILE: &str = "response.json";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AgentState {
    /// The executor holds the agent's inbox-directory lock (§2.11) but no
    /// model call's `response.json` fd is open — a driver between steps or
    /// running a tool.
    Live,
    /// `live` refined: a model call is in flight right now — the latest
    /// step's `response.json` fd is still open (§3.5, §4.4).
    InFlight,
    /// No lock held; the latest step's `response.json` is a clean,
    /// complete model call (§4.4) — awaiting a message (§2.4).
    Quiescent,
    /// No lock held; the latest step failed, was killed, or never ran
    /// (§2.9, §2.10). Kill/crash/explicit stop are indistinguishable here.
    Stopped,
}

/// Classify `agent_id` in `workspace` from the two liveness observations
/// plus the latest step's terminal framing.
pub(super) fn classify(
    workspace: &Path,
    agent_id: &str,
    lock: &dyn LockProbe,
    writer: &dyn WriterProbe,
) -> AgentState {
    let inbox_dir = workspace.join(INBOX_DIR).join(agent_id);
    if lock.lock_held(&inbox_dir) {
        return live_substate(workspace, agent_id, writer);
    }
    // No lock: the terminal-only reading rules (§4.4) settle the file.
    if latest_response_complete(workspace, agent_id) {
        AgentState::Quiescent
    } else {
        AgentState::Stopped
    }
}

/// Under the lock: `InFlight` iff a writer still holds the latest step's
/// `response.json` open (a model call right now), else plain `Live`.
fn live_substate(workspace: &Path, agent_id: &str, writer: &dyn WriterProbe) -> AgentState {
    match latest_response_path(workspace, agent_id) {
        Some(path) if writer.writer_open(&path) => AgentState::InFlight,
        _ => AgentState::Live,
    }
}

fn latest_response_path(workspace: &Path, agent_id: &str) -> Option<PathBuf> {
    let steps = workspace.join(STEPS_DIR).join(agent_id);
    Some(latest_step_dir(&steps)?.join(RESPONSE_FILE))
}

/// Is the latest step's `response.json` *complete* (§4.4)? Reads the file
/// once; absence or an unreadable file is not complete.
fn latest_response_complete(workspace: &Path, agent_id: &str) -> bool {
    let Some(path) = latest_response_path(workspace, agent_id) else {
        return false;
    };
    match std::fs::read(&path) {
        Ok(bytes) => last_segment_complete(&bytes),
        Err(_) => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    /// Stub probes with fixed answers.
    struct Stub(bool);
    impl LockProbe for Stub {
        fn lock_held(&self, _dir: &Path) -> bool {
            self.0
        }
    }
    impl WriterProbe for Stub {
        fn writer_open(&self, _path: &Path) -> bool {
            self.0
        }
    }
    fn yes() -> Stub {
        Stub(true)
    }
    fn no() -> Stub {
        Stub(false)
    }

    fn write(path: &Path, contents: &[u8]) {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).unwrap();
        }
        std::fs::write(path, contents).unwrap();
    }

    fn resp(dir: &Path, agent: &str, seq: &str) -> PathBuf {
        dir.join(STEPS_DIR)
            .join(agent)
            .join(seq)
            .join(RESPONSE_FILE)
    }

    const FINISH_END: &[u8] = br#"{"type":"message_start","v":1,"role":"assistant"}
{"type":"finish","reason":"stop"}
{"type":"end"}
"#;
    const ERROR_END: &[u8] = br#"{"type":"message_start","v":1,"role":"assistant"}
{"type":"error","kind":"transport","message":"reset"}
{"type":"end"}
"#;

    #[test]
    fn lock_held_and_response_open_is_in_flight() {
        let dir = tempdir().unwrap();
        let agent = "20260427T140000Z-aaaa";
        write(&resp(dir.path(), agent, "001"), FINISH_END);
        // Lock held (yes) + writer open (yes) → InFlight.
        assert_eq!(
            classify(dir.path(), agent, &yes(), &yes()),
            AgentState::InFlight
        );
    }

    #[test]
    fn lock_held_and_response_closed_is_live() {
        let dir = tempdir().unwrap();
        let agent = "20260427T140000Z-bbbb";
        write(&resp(dir.path(), agent, "001"), FINISH_END);
        // Lock held (yes) + writer closed (no) → Live (between calls).
        assert_eq!(classify(dir.path(), agent, &yes(), &no()), AgentState::Live);
    }

    #[test]
    fn lock_held_with_no_response_is_live() {
        // A driver that acquired the lock but has not opened a
        // response.json yet (pre-first-call) is Live, not InFlight.
        let dir = tempdir().unwrap();
        let agent = "20260427T140000Z-cccc";
        std::fs::create_dir_all(dir.path().join(STEPS_DIR).join(agent)).unwrap();
        assert_eq!(
            classify(dir.path(), agent, &yes(), &yes()),
            AgentState::Live
        );
    }

    #[test]
    fn no_lock_and_complete_response_is_quiescent() {
        let dir = tempdir().unwrap();
        let agent = "20260427T140000Z-dddd";
        write(&resp(dir.path(), agent, "001"), FINISH_END);
        assert_eq!(
            classify(dir.path(), agent, &no(), &no()),
            AgentState::Quiescent
        );
    }

    #[test]
    fn no_lock_and_failed_response_is_stopped() {
        let dir = tempdir().unwrap();
        let agent = "20260427T140000Z-eeee";
        write(&resp(dir.path(), agent, "001"), ERROR_END);
        assert_eq!(
            classify(dir.path(), agent, &no(), &no()),
            AgentState::Stopped
        );
    }

    #[test]
    fn no_lock_and_no_response_is_stopped() {
        let dir = tempdir().unwrap();
        assert_eq!(
            classify(dir.path(), "no-such-agent", &no(), &no()),
            AgentState::Stopped
        );
    }

    #[test]
    fn classify_reads_latest_step_only() {
        let dir = tempdir().unwrap();
        let agent = "20260427T140000Z-ffff";
        write(&resp(dir.path(), agent, "001"), FINISH_END);
        // Latest step is mid-stream (no terminal) → not complete → Stopped
        // with no lock.
        write(
            &resp(dir.path(), agent, "002"),
            b"{\"type\":\"content_delta\",\"index\":0,\"delta\":{\"text_delta\":\"go\"}}\n",
        );
        assert_eq!(
            classify(dir.path(), agent, &no(), &no()),
            AgentState::Stopped
        );
    }
}
