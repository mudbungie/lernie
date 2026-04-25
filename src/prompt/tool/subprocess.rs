//! Subprocess plumbing for the tool executor — spawn, capture, wait,
//! cascade.
//!
//! Capture is one writer thread (stdin) plus two reader threads
//! (stdout, stderr) joined after the child exits, so a tool that
//! ignores stdin or fills its stdout pipe cannot deadlock the wait
//! loop. The main thread polls `stop` every [`POLL_INTERVAL`]; on stop
//! it sends SIGTERM, waits the deadline, then SIGKILL — same pattern
//! ARCH §4.4 pins for adapters.
//!
//! Errors on the stdin writer are intentionally swallowed: the §3.3
//! contract only requires stdin be *offered* to the tool; whether the
//! tool reads it is the tool's choice.

use super::ExecError;
use std::ffi::OsString;
use std::io::{Read, Write};
use std::process::{Child, Command, ExitStatus, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::{Duration, Instant};

/// Cadence for the stop-flag polling loop. Small enough that a UI
/// cancel feels instant; large enough that an idle harness costs
/// nothing measurable.
const POLL_INTERVAL: Duration = Duration::from_millis(50);

/// Captured tool stdio + final status.
pub(super) struct Captured {
    pub(super) stdout: Vec<u8>,
    pub(super) stderr: Vec<u8>,
    pub(super) status: ExitStatus,
}

/// Spawn `binary args`, write `stdin_bytes`, capture stdio in
/// background threads, and reap the child. Polls `stop` every
/// [`POLL_INTERVAL`]; when set, sends SIGTERM and waits up to
/// `deadline` before SIGKILL.
pub(super) fn spawn_and_capture(
    binary: &OsString,
    args: &[OsString],
    stdin_bytes: &[u8],
    stop: &AtomicBool,
    deadline: Duration,
    tool_name: &str,
) -> Result<Captured, ExecError> {
    let mut child = Command::new(binary)
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|source| ExecError::Spawn {
            name: tool_name.to_string(),
            source,
        })?;

    let stdin_data = stdin_bytes.to_vec();
    let mut child_stdin = child.stdin.take().expect("stdin is piped");
    let stdin_thread = thread::spawn(move || {
        let _ = child_stdin.write_all(&stdin_data);
        drop(child_stdin);
    });

    let mut child_stdout = child.stdout.take().expect("stdout is piped");
    let mut child_stderr = child.stderr.take().expect("stderr is piped");
    let stdout_thread = thread::spawn(move || {
        let mut buf = Vec::new();
        let _ = child_stdout.read_to_end(&mut buf);
        buf
    });
    let stderr_thread = thread::spawn(move || {
        let mut buf = Vec::new();
        let _ = child_stderr.read_to_end(&mut buf);
        buf
    });

    let status = wait_with_stop(&mut child, stop, deadline);
    stdin_thread.join().expect("stdin writer did not panic");
    let stdout = stdout_thread.join().expect("stdout reader did not panic");
    let stderr = stderr_thread.join().expect("stderr reader did not panic");

    Ok(Captured {
        stdout,
        stderr,
        status,
    })
}

/// Poll `child` for completion. When `stop` flips, send SIGTERM, wait
/// up to `deadline`, then SIGKILL. Returns whatever final
/// [`ExitStatus`] the kernel reports — a tool that exits cleanly
/// inside the deadline reports its real exit code (and is therefore
/// not flagged `KilledBySignal` upstream).
fn wait_with_stop(child: &mut Child, stop: &AtomicBool, deadline: Duration) -> ExitStatus {
    loop {
        if let Some(status) = try_reap(child) {
            return status;
        }
        if stop.load(Ordering::SeqCst) {
            return cascade_terminate(child, deadline);
        }
        thread::sleep(POLL_INTERVAL);
    }
}

/// `try_wait`-as-`Option` wrapper. An I/O error reaping a child the
/// kernel still owns is treated as "not exited yet" — the next poll
/// retries.
fn try_reap(child: &mut Child) -> Option<ExitStatus> {
    child.try_wait().ok().flatten()
}

/// Send SIGTERM, wait `deadline`, then SIGKILL. SIGKILL is
/// uncatchable so the final [`Child::wait`] is bounded by kernel reap
/// latency (microseconds).
fn cascade_terminate(child: &mut Child, deadline: Duration) -> ExitStatus {
    let pid = child.id() as i32;
    // SAFETY: kill on a child PID we own; signo is a constant.
    unsafe {
        libc::kill(pid, libc::SIGTERM);
    }
    let term_until = Instant::now() + deadline;
    while Instant::now() < term_until {
        if let Some(status) = try_reap(child) {
            return status;
        }
        thread::sleep(POLL_INTERVAL);
    }
    // SAFETY: same as above; SIGKILL is uncatchable.
    unsafe {
        libc::kill(pid, libc::SIGKILL);
    }
    child.wait().expect("kernel reaps SIGKILL'd child")
}
